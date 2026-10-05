//! The endpoint-0 request handler for a CDC-ACM device.
//!
//! [`ControlHandler`] is the "brain" of enumeration with the hardware cut
//! away. The chip driver hands it each SETUP packet and gets back a
//! [`ControlAction`] that says which stages to run; when a request changes
//! something the hardware must act on — the bus address, which endpoints
//! exist, an endpoint's halt state — the action carries an [`Effect`].
//!
//! ```text
//!   driver                          ControlHandler
//!   ------                          --------------
//!   SETUP arrives  --- setup() --->  decode, validate, update state
//!                  <-- action ----   DataIn / DataOut / NoData(effect) / Stall
//!   DataIn:   send reply() in <=64-byte packets (+ ZLP if asked),
//!             then receive the host's zero-length OUT status
//!   DataOut:  receive `len` bytes  --- data_out() ---> NoData(effect) / Stall
//!             then send a zero-length IN status
//!   NoData:   apply the effect (SetAddress only *after* the status stage),
//!             send a zero-length IN status
//!   Stall:    STALL both directions of endpoint 0 until the next SETUP
//!   bus reset --- bus_reset() --->  back to the Default state
//! ```
//!
//! # Device states
//!
//! USB 2.0 §9.1.1 (Figure 9-1) defines the states this handler tracks:
//! **Default** after a bus reset (address 0, unconfigured), **Address** once
//! SET_ADDRESS has given it a non-zero address, and **Configured** once
//! SET_CONFIGURATION has selected configuration 1. Only in the Configured
//! state do the data endpoints exist; [`ControlHandler::is_configured`] is
//! the answer to "may bytes flow?".
//!
//! # What gets a STALL
//!
//! Anything this device does not implement is answered with a STALL, which
//! USB 2.0 §8.5.3.4 calls a protocol stall: it ends this one request with
//! "Request Error" and is cleared automatically by the next SETUP. That
//! includes vendor requests, SET_DESCRIPTOR, SYNCH_FRAME, unknown class
//! requests, requests for descriptors that do not exist (string index > 3,
//! the device qualifier, other-speed configuration), requests naming an
//! interface or endpoint that does not exist, and requests whose `wLength`
//! contradicts the request's definition.

use crate::usb::cdc_acm::{class_request, ControlLineState, LineCoding, LINE_CODING_LEN};
use crate::usb::control_in_needs_zlp;
use crate::usb::descriptor::{
    device_descriptor, string_descriptor, UsbDeviceConfig, CONFIGURATION_DESCRIPTOR,
    CONFIGURATION_VALUE, EP0_MAX_PACKET, EP_DATA_IN, EP_DATA_OUT, EP_NOTIFY_IN,
    INTERFACE_COMM, LANGUAGE_ID_DESCRIPTOR, NUM_INTERFACES, STRING_MANUFACTURER,
    STRING_PRODUCT, STRING_SERIAL,
};
use crate::usb::setup::{
    descriptor_type, feature, request, Direction, Recipient, RequestKind, SetupPacket,
};

/// Size of the reply buffer: the largest reply is a string descriptor of
/// [`MAX_STRING_UNITS`](crate::usb::descriptor::MAX_STRING_UNITS) code
/// units, 254 bytes.
pub const REPLY_CAPACITY: usize = 256;

/// USB device state (USB 2.0 §9.1.1). Attached/Powered/Suspended are bus
/// conditions the chip driver tracks itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceState {
    /// After bus reset: address 0, not configured.
    Default,
    /// SET_ADDRESS has assigned a non-zero address; not configured.
    Address,
    /// SET_CONFIGURATION(1) has completed; the data endpoints exist.
    Configured,
}

/// How the chip driver must carry out a control request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlAction {
    /// Device-to-host data stage: send the first `len` bytes of
    /// [`ControlHandler::reply`] in packets of at most
    /// [`EP0_MAX_PACKET`] bytes, starting with DATA1 and alternating; if
    /// `zlp` is true, follow them with a zero-length packet (see
    /// [`control_in_needs_zlp`]). Then receive the host's zero-length OUT
    /// status packet (DATA1).
    DataIn {
        /// Bytes to send, already clipped to `wLength`.
        len: usize,
        /// Whether a trailing zero-length packet is required.
        zlp: bool,
    },
    /// Host-to-device data stage: receive exactly `len` bytes (one packet
    /// for every request this handler accepts, starting with DATA1), pass
    /// them to [`ControlHandler::data_out`], and act on what it returns.
    DataOut {
        /// Bytes expected, equal to `wLength`.
        len: usize,
    },
    /// No data stage (or one of zero length): apply the effect as described
    /// on [`Effect`], then send a zero-length IN status packet (DATA1).
    NoData(Effect),
    /// Answer with STALL on both directions of endpoint 0 (on the RP2350:
    /// `EP_STALL_ARM` plus the STALL bit in both buffer controls). The next
    /// SETUP clears it.
    Stall,
}

/// A change the hardware must make as a result of a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Nothing beyond the status stage.
    None,
    /// Program this bus address — but only **after** the status stage has
    /// completed. USB 2.0 §9.4.6 and §9.2.6.3: the status packet of
    /// SET_ADDRESS is still exchanged at the old address (0); answering it
    /// at the new one would make the host see a timeout. The host then
    /// allows the device 2 ms before it uses the new address.
    SetAddress(u8),
    /// The configuration changed from `from` to `to` (each 0 or
    /// [`CONFIGURATION_VALUE`]); apply immediately. Going to 1: enable the
    /// data endpoints with both data toggles at DATA0 (USB 2.0 §9.4.5
    /// note on SetConfiguration). Going to 0: disable them. 1 to 1 is a
    /// re-configuration and must also reset the toggles.
    ConfigurationChanged {
        /// Previous configuration value.
        from: u8,
        /// New configuration value.
        to: u8,
    },
    /// SET_FEATURE / CLEAR_FEATURE(ENDPOINT_HALT) on a data or notification
    /// endpoint; apply immediately. `halted: true`: answer every
    /// transaction on that endpoint with STALL. `halted: false`: stop
    /// stalling **and reset the endpoint's data toggle to DATA0** — USB 2.0
    /// §9.4.5 requires the toggle reset even if the endpoint was not
    /// halted, which hosts rely on to resynchronise.
    EndpointHalt {
        /// Endpoint address, direction in bit 7.
        endpoint: u8,
        /// New halt state.
        halted: bool,
    },
    /// SET_INTERFACE(alternate 0) on this interface: every interface here
    /// has only alternate setting 0, so the only effect is that the
    /// interface's endpoints restart with DATA0 toggles (USB 2.0 §9.4.10).
    InterfaceReset(u8),
}

/// Which request an expected OUT data stage belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingOut {
    None,
    LineCoding,
}

/// The endpoint-0 state machine for the CDC-ACM function described in
/// [`crate::usb::descriptor`].
#[derive(Clone, Debug)]
pub struct ControlHandler {
    config: UsbDeviceConfig,
    state: DeviceState,
    configuration: u8,
    /// Halt flags, one bit per non-control endpoint: see `halt_bit`.
    halted: u8,
    line_coding: LineCoding,
    line_state: ControlLineState,
    pending_out: PendingOut,
    reply: [u8; REPLY_CAPACITY],
    reply_len: usize,
}

/// Bit in `ControlHandler::halted` for one of this device's non-control
/// endpoints, or `None` if the address is not one of them.
const fn halt_bit(endpoint: u8) -> Option<u8> {
    match endpoint {
        EP_NOTIFY_IN => Some(1 << 0),
        EP_DATA_OUT => Some(1 << 1),
        EP_DATA_IN => Some(1 << 2),
        _ => None,
    }
}

impl ControlHandler {
    /// A handler in the Default state, line coding
    /// [`LineCoding::DEFAULT`], DTR and RTS low.
    pub const fn new(config: UsbDeviceConfig) -> Self {
        Self {
            config,
            state: DeviceState::Default,
            configuration: 0,
            halted: 0,
            line_coding: LineCoding::DEFAULT,
            line_state: ControlLineState { dtr: false, rts: false },
            pending_out: PendingOut::None,
            reply: [0; REPLY_CAPACITY],
            reply_len: 0,
        }
    }

    /// The descriptor configuration this handler answers with.
    pub const fn config(&self) -> &UsbDeviceConfig {
        &self.config
    }

    /// Current device state.
    pub const fn state(&self) -> DeviceState {
        self.state
    }

    /// Current configuration value: 0, or [`CONFIGURATION_VALUE`].
    pub const fn configuration(&self) -> u8 {
        self.configuration
    }

    /// True in the Configured state — the data endpoints exist.
    pub const fn is_configured(&self) -> bool {
        matches!(self.state, DeviceState::Configured)
    }

    /// Whether the host has halted `endpoint` (address with direction bit)
    /// with SET_FEATURE(ENDPOINT_HALT). Always false for endpoint 0 and for
    /// addresses that are not this device's.
    pub const fn is_halted(&self, endpoint: u8) -> bool {
        match halt_bit(endpoint) {
            Some(bit) => self.halted & bit != 0,
            None => false,
        }
    }

    /// The line coding most recently set by the host.
    pub const fn line_coding(&self) -> LineCoding {
        self.line_coding
    }

    /// DTR and RTS as most recently set by the host.
    pub const fn control_line_state(&self) -> ControlLineState {
        self.line_state
    }

    /// The reply prepared by the last [`setup`](Self::setup) that returned
    /// [`ControlAction::DataIn`]; exactly `len` bytes long.
    pub fn reply(&self) -> &[u8] {
        &self.reply[..self.reply_len]
    }

    /// Return to the Default state after a USB bus reset (or a
    /// disconnect): address 0, unconfigured, no halts, line coding and
    /// control lines back to their defaults, any half-finished request
    /// forgotten. USB 2.0 §9.1.1.3: a bus reset returns a device to the
    /// Default state from any other state.
    pub fn bus_reset(&mut self) {
        self.state = DeviceState::Default;
        self.configuration = 0;
        self.halted = 0;
        self.line_coding = LineCoding::DEFAULT;
        self.line_state = ControlLineState::default();
        self.pending_out = PendingOut::None;
        self.reply_len = 0;
    }

    /// Decide how to answer a SETUP packet.
    ///
    /// A new SETUP always aborts whatever control transfer was in
    /// progress (USB 2.0 §8.5.3), so any pending OUT data stage is
    /// forgotten first.
    pub fn setup(&mut self, pkt: &SetupPacket) -> ControlAction {
        self.pending_out = PendingOut::None;
        self.reply_len = 0;
        match pkt.kind() {
            RequestKind::Standard => self.standard(pkt),
            RequestKind::Class => self.class(pkt),
            RequestKind::Vendor | RequestKind::Reserved => ControlAction::Stall,
        }
    }

    /// Hand over the bytes of an OUT data stage that
    /// [`setup`](Self::setup) asked for with [`ControlAction::DataOut`].
    /// Returns [`ControlAction::NoData`] (send the IN status) or
    /// [`ControlAction::Stall`] if the data is invalid or nothing was
    /// expected.
    pub fn data_out(&mut self, data: &[u8]) -> ControlAction {
        let pending = self.pending_out;
        self.pending_out = PendingOut::None;
        match pending {
            PendingOut::LineCoding => match LineCoding::from_bytes(data) {
                Some(lc) => {
                    self.line_coding = lc;
                    ControlAction::NoData(Effect::None)
                }
                None => ControlAction::Stall,
            },
            PendingOut::None => ControlAction::Stall,
        }
    }

    /// Copy `bytes` into the reply buffer, clipped to `wLength`, and build
    /// the matching [`ControlAction::DataIn`].
    fn reply_with(&mut self, pkt: &SetupPacket, bytes: &[u8]) -> ControlAction {
        let len = bytes.len().min(REPLY_CAPACITY);
        self.reply[..len].copy_from_slice(&bytes[..len]);
        self.reply_len = len;
        self.finish_reply(pkt)
    }

    /// Clip the already-filled reply buffer to `wLength` and build the
    /// action. A device-to-host request with `wLength == 0` has no data
    /// stage, so its status is a zero-length IN packet like any no-data
    /// request (USB 2.0 §8.5.3).
    fn finish_reply(&mut self, pkt: &SetupPacket) -> ControlAction {
        let w_length = pkt.length as usize;
        if w_length == 0 {
            self.reply_len = 0;
            return ControlAction::NoData(Effect::None);
        }
        self.reply_len = self.reply_len.min(w_length);
        ControlAction::DataIn {
            len: self.reply_len,
            zlp: control_in_needs_zlp(self.reply_len, w_length, EP0_MAX_PACKET),
        }
    }

    /// Standard requests, USB 2.0 §9.4.
    fn standard(&mut self, pkt: &SetupPacket) -> ControlAction {
        use Direction::{DeviceToHost as In, HostToDevice as Out};
        match (pkt.direction(), pkt.recipient(), pkt.request) {
            (In, Recipient::Device, request::GET_DESCRIPTOR) => self.get_descriptor(pkt),
            (In, _, request::GET_STATUS) => self.get_status(pkt),
            (In, Recipient::Device, request::GET_CONFIGURATION) => {
                let v = [self.configuration];
                self.reply_with(pkt, &v)
            }
            (In, Recipient::Interface, request::GET_INTERFACE) => {
                if self.is_configured() && pkt.index < NUM_INTERFACES as u16 {
                    self.reply_with(pkt, &[0])
                } else {
                    ControlAction::Stall
                }
            }
            (Out, _, _) if pkt.length != 0 => ControlAction::Stall,
            (Out, Recipient::Device, request::SET_ADDRESS) => self.set_address(pkt),
            (Out, Recipient::Device, request::SET_CONFIGURATION) => self.set_configuration(pkt),
            (Out, Recipient::Interface, request::SET_INTERFACE) => {
                if self.is_configured() && pkt.index < NUM_INTERFACES as u16 && pkt.value == 0 {
                    ControlAction::NoData(Effect::InterfaceReset(pkt.index as u8))
                } else {
                    ControlAction::Stall
                }
            }
            (Out, recipient, request::CLEAR_FEATURE) => self.feature(pkt, recipient, false),
            (Out, recipient, request::SET_FEATURE) => self.feature(pkt, recipient, true),
            _ => ControlAction::Stall,
        }
    }

    /// GET_DESCRIPTOR, USB 2.0 §9.4.3.
    fn get_descriptor(&mut self, pkt: &SetupPacket) -> ControlAction {
        match (pkt.descriptor_type(), pkt.descriptor_index()) {
            (descriptor_type::DEVICE, 0) => {
                let d = device_descriptor(&self.config);
                self.reply_with(pkt, &d)
            }
            (descriptor_type::CONFIGURATION, 0) => self.reply_with(pkt, &CONFIGURATION_DESCRIPTOR),
            (descriptor_type::STRING, 0) => self.reply_with(pkt, &LANGUAGE_ID_DESCRIPTOR),
            (descriptor_type::STRING, index) => {
                // wIndex carries the language ID; there is only one
                // language, so any value gets the same strings.
                let s = match index {
                    STRING_MANUFACTURER => self.config.manufacturer,
                    STRING_PRODUCT => self.config.product,
                    STRING_SERIAL => self.config.serial_number,
                    _ => return ControlAction::Stall,
                };
                self.reply_len = string_descriptor(s, &mut self.reply);
                self.finish_reply(pkt)
            }
            // DEVICE_QUALIFIER and OTHER_SPEED_CONFIGURATION: USB 2.0 §9.6.2
            // says a full-speed-only device must answer with Request Error.
            // Everything else (BOS, HID, debug, unknown) likewise.
            _ => ControlAction::Stall,
        }
    }

    /// GET_STATUS, USB 2.0 §9.4.5 (Figures 9-4, 9-5, 9-6).
    fn get_status(&mut self, pkt: &SetupPacket) -> ControlAction {
        match pkt.recipient() {
            // Bit 0 self-powered = 0 (bmAttributes says bus powered), bit 1
            // remote wakeup = 0 (not supported).
            Recipient::Device => self.reply_with(pkt, &[0, 0]),
            Recipient::Interface => {
                if self.is_configured() && pkt.index < NUM_INTERFACES as u16 {
                    self.reply_with(pkt, &[0, 0])
                } else {
                    ControlAction::Stall
                }
            }
            Recipient::Endpoint => {
                let ep = pkt.index as u8;
                if pkt.index > 0xff {
                    ControlAction::Stall
                } else if ep & 0x7f == 0 {
                    self.reply_with(pkt, &[0, 0])
                } else if self.is_configured() && halt_bit(ep).is_some() {
                    let halted = self.is_halted(ep) as u8;
                    self.reply_with(pkt, &[halted, 0])
                } else {
                    ControlAction::Stall
                }
            }
            _ => ControlAction::Stall,
        }
    }

    /// SET_ADDRESS, USB 2.0 §9.4.6. Valid in the Default and Address
    /// states; the address takes effect after the status stage.
    fn set_address(&mut self, pkt: &SetupPacket) -> ControlAction {
        if pkt.value > 127 || pkt.index != 0 || self.is_configured() {
            return ControlAction::Stall;
        }
        let address = pkt.value as u8;
        self.state = if address == 0 { DeviceState::Default } else { DeviceState::Address };
        ControlAction::NoData(Effect::SetAddress(address))
    }

    /// SET_CONFIGURATION, USB 2.0 §9.4.7. Valid once the device has an
    /// address; value 0 deconfigures, [`CONFIGURATION_VALUE`] configures.
    fn set_configuration(&mut self, pkt: &SetupPacket) -> ControlAction {
        let to = pkt.value;
        if pkt.index != 0
            || matches!(self.state, DeviceState::Default)
            || (to != 0 && to != CONFIGURATION_VALUE as u16)
        {
            return ControlAction::Stall;
        }
        let to = to as u8;
        let from = self.configuration;
        self.configuration = to;
        self.halted = 0;
        self.state = if to == 0 { DeviceState::Address } else { DeviceState::Configured };
        ControlAction::NoData(Effect::ConfigurationChanged { from, to })
    }

    /// SET_FEATURE / CLEAR_FEATURE, USB 2.0 §9.4.9 / §9.4.1.
    fn feature(&mut self, pkt: &SetupPacket, recipient: Recipient, set: bool) -> ControlAction {
        match recipient {
            Recipient::Endpoint if pkt.value == feature::ENDPOINT_HALT && pkt.index <= 0xff => {
                let ep = pkt.index as u8;
                if ep & 0x7f == 0 {
                    // Halting the default control pipe is "neither required
                    // nor recommended" (USB 2.0 §9.4.5); accept and ignore.
                    return ControlAction::NoData(Effect::None);
                }
                match halt_bit(ep) {
                    Some(bit) if self.is_configured() => {
                        if set {
                            self.halted |= bit;
                        } else {
                            self.halted &= !bit;
                        }
                        ControlAction::NoData(Effect::EndpointHalt { endpoint: ep, halted: set })
                    }
                    _ => ControlAction::Stall,
                }
            }
            // DEVICE_REMOTE_WAKEUP is not advertised in bmAttributes and
            // TEST_MODE is high-speed only: both are request errors here.
            _ => ControlAction::Stall,
        }
    }

    /// CDC class requests addressed to the communication interface,
    /// PSTN 1.2 §6.3.
    fn class(&mut self, pkt: &SetupPacket) -> ControlAction {
        if pkt.recipient() != Recipient::Interface || pkt.index != INTERFACE_COMM as u16 {
            return ControlAction::Stall;
        }
        match (pkt.direction(), pkt.request) {
            (Direction::HostToDevice, class_request::SET_LINE_CODING) => {
                if pkt.length as usize != LINE_CODING_LEN {
                    return ControlAction::Stall;
                }
                self.pending_out = PendingOut::LineCoding;
                ControlAction::DataOut { len: LINE_CODING_LEN }
            }
            (Direction::DeviceToHost, class_request::GET_LINE_CODING) => {
                let lc = self.line_coding.to_bytes();
                self.reply_with(pkt, &lc)
            }
            (Direction::HostToDevice, class_request::SET_CONTROL_LINE_STATE) if pkt.length == 0 => {
                self.line_state = ControlLineState::from_value(pkt.value);
                ControlAction::NoData(Effect::None)
            }
            (Direction::HostToDevice, class_request::SEND_BREAK) if pkt.length == 0 => {
                ControlAction::NoData(Effect::None)
            }
            _ => ControlAction::Stall,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usb::cdc_acm::{Parity, StopBits};
    use crate::usb::descriptor::{CONFIGURATION_DESCRIPTOR_LEN, DEVICE_DESCRIPTOR_LEN};

    const CFG: UsbDeviceConfig = UsbDeviceConfig {
        vendor_id: 0x2E8A,
        product_id: 0x000A,
        device_release: 0x0100,
        manufacturer: "M",
        product: "Prod",
        serial_number: "S1",
    };

    fn pkt(b: [u8; 8]) -> SetupPacket {
        SetupPacket::from_bytes(b)
    }

    /// Run the usual Linux/Windows enumeration prefix and leave the handler
    /// configured.
    fn configured() -> ControlHandler {
        let mut h = ControlHandler::new(CFG);
        assert_eq!(
            h.setup(&pkt([0x00, 0x05, 0x07, 0x00, 0, 0, 0, 0])),
            ControlAction::NoData(Effect::SetAddress(7))
        );
        assert_eq!(
            h.setup(&pkt([0x00, 0x09, 0x01, 0x00, 0, 0, 0, 0])),
            ControlAction::NoData(Effect::ConfigurationChanged { from: 0, to: 1 })
        );
        assert!(h.is_configured());
        h
    }

    #[test]
    fn get_device_descriptor_clipped_to_wlength() {
        let mut h = ControlHandler::new(CFG);
        // wLength 64: the whole 18 bytes, short packet, no ZLP.
        let a = h.setup(&pkt([0x80, 0x06, 0x00, 0x01, 0, 0, 64, 0]));
        assert_eq!(a, ControlAction::DataIn { len: DEVICE_DESCRIPTOR_LEN, zlp: false });
        assert_eq!(h.reply(), &device_descriptor(&CFG));
        // Old Windows asks for 8 bytes first.
        let a = h.setup(&pkt([0x80, 0x06, 0x00, 0x01, 0, 0, 8, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 8, zlp: false });
        assert_eq!(h.reply(), &device_descriptor(&CFG)[..8]);
    }

    #[test]
    fn get_configuration_descriptor_header_then_whole() {
        let mut h = ControlHandler::new(CFG);
        let a = h.setup(&pkt([0x80, 0x06, 0x00, 0x02, 0, 0, 9, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 9, zlp: false });
        assert_eq!(h.reply(), &CONFIGURATION_DESCRIPTOR[..9]);
        let a = h.setup(&pkt([0x80, 0x06, 0x00, 0x02, 0, 0, 0xff, 0]));
        assert_eq!(a, ControlAction::DataIn { len: CONFIGURATION_DESCRIPTOR_LEN, zlp: false });
        assert_eq!(h.reply(), &CONFIGURATION_DESCRIPTOR[..]);
        // Index 1 does not exist.
        assert_eq!(h.setup(&pkt([0x80, 0x06, 0x01, 0x02, 0, 0, 0xff, 0])), ControlAction::Stall);
    }

    #[test]
    fn get_string_descriptors() {
        let mut h = ControlHandler::new(CFG);
        let a = h.setup(&pkt([0x80, 0x06, 0x00, 0x03, 0, 0, 0xff, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 4, zlp: false });
        assert_eq!(h.reply(), &[4, 3, 0x09, 0x04]);
        let a = h.setup(&pkt([0x80, 0x06, 0x02, 0x03, 0x09, 0x04, 0xff, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 10, zlp: false });
        assert_eq!(h.reply(), &[10, 3, b'P', 0, b'r', 0, b'o', 0, b'd', 0]);
        let a = h.setup(&pkt([0x80, 0x06, 0x01, 0x03, 0x09, 0x04, 0xff, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 4, zlp: false });
        assert_eq!(h.reply(), &[4, 3, b'M', 0]);
        let a = h.setup(&pkt([0x80, 0x06, 0x03, 0x03, 0x09, 0x04, 0xff, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 6, zlp: false });
        assert_eq!(h.reply(), &[6, 3, b'S', 0, b'1', 0]);
        // Header-only read of a string (some hosts read bLength first).
        let a = h.setup(&pkt([0x80, 0x06, 0x02, 0x03, 0x09, 0x04, 2, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 2, zlp: false });
        assert_eq!(h.reply(), &[10, 3]);
        // No such string; Microsoft OS string (0xEE) too.
        assert_eq!(h.setup(&pkt([0x80, 0x06, 0x04, 0x03, 0x09, 0x04, 0xff, 0])), ControlAction::Stall);
        assert_eq!(h.setup(&pkt([0x80, 0x06, 0xEE, 0x03, 0, 0, 0x12, 0])), ControlAction::Stall);
    }

    #[test]
    fn string_reply_needing_zlp() {
        // 31 characters -> 64-byte descriptor: exactly one full packet,
        // host asked for 255, so a ZLP must follow.
        const CFG64: UsbDeviceConfig = UsbDeviceConfig {
            product: "0123456789012345678901234567890",
            ..CFG
        };
        let mut h = ControlHandler::new(CFG64);
        let a = h.setup(&pkt([0x80, 0x06, 0x02, 0x03, 0x09, 0x04, 0xff, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 64, zlp: true });
    }

    #[test]
    fn qualifier_and_other_speed_stall() {
        let mut h = ControlHandler::new(CFG);
        assert_eq!(h.setup(&pkt([0x80, 0x06, 0x00, 0x06, 0, 0, 10, 0])), ControlAction::Stall);
        assert_eq!(h.setup(&pkt([0x80, 0x06, 0x00, 0x07, 0, 0, 9, 0])), ControlAction::Stall);
        // BOS (type 15) too.
        assert_eq!(h.setup(&pkt([0x80, 0x06, 0x00, 0x0F, 0, 0, 5, 0])), ControlAction::Stall);
        // GET_DESCRIPTOR addressed to an interface (HID report etc.).
        assert_eq!(h.setup(&pkt([0x81, 0x06, 0x00, 0x22, 0, 0, 0x40, 0])), ControlAction::Stall);
    }

    #[test]
    fn set_address_rules() {
        let mut h = ControlHandler::new(CFG);
        assert_eq!(h.state(), DeviceState::Default);
        assert_eq!(
            h.setup(&pkt([0x00, 0x05, 0x2A, 0x00, 0, 0, 0, 0])),
            ControlAction::NoData(Effect::SetAddress(42))
        );
        assert_eq!(h.state(), DeviceState::Address);
        // Address 0 goes back to Default.
        assert_eq!(
            h.setup(&pkt([0x00, 0x05, 0x00, 0x00, 0, 0, 0, 0])),
            ControlAction::NoData(Effect::SetAddress(0))
        );
        assert_eq!(h.state(), DeviceState::Default);
        // Out of range, or with a data stage.
        assert_eq!(h.setup(&pkt([0x00, 0x05, 0x80, 0x00, 0, 0, 0, 0])), ControlAction::Stall);
        assert_eq!(h.setup(&pkt([0x00, 0x05, 0x01, 0x00, 0, 0, 1, 0])), ControlAction::Stall);
        // Not while configured.
        let mut h = configured();
        assert_eq!(h.setup(&pkt([0x00, 0x05, 0x03, 0x00, 0, 0, 0, 0])), ControlAction::Stall);
    }

    #[test]
    fn set_and_get_configuration() {
        let mut h = ControlHandler::new(CFG);
        // Not in the Default state.
        assert_eq!(h.setup(&pkt([0x00, 0x09, 0x01, 0x00, 0, 0, 0, 0])), ControlAction::Stall);
        let mut h = configured();
        let a = h.setup(&pkt([0x80, 0x08, 0, 0, 0, 0, 1, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 1, zlp: false });
        assert_eq!(h.reply(), &[1]);
        // Re-configuration resets halts and is reported 1 -> 1.
        h.setup(&pkt([0x02, 0x03, 0x00, 0x00, 0x82, 0x00, 0, 0]));
        assert!(h.is_halted(EP_DATA_IN));
        assert_eq!(
            h.setup(&pkt([0x00, 0x09, 0x01, 0x00, 0, 0, 0, 0])),
            ControlAction::NoData(Effect::ConfigurationChanged { from: 1, to: 1 })
        );
        assert!(!h.is_halted(EP_DATA_IN));
        // Deconfigure.
        assert_eq!(
            h.setup(&pkt([0x00, 0x09, 0x00, 0x00, 0, 0, 0, 0])),
            ControlAction::NoData(Effect::ConfigurationChanged { from: 1, to: 0 })
        );
        assert_eq!(h.state(), DeviceState::Address);
        assert!(!h.is_configured());
        // Unknown configuration value.
        assert_eq!(h.setup(&pkt([0x00, 0x09, 0x02, 0x00, 0, 0, 0, 0])), ControlAction::Stall);
    }

    #[test]
    fn get_status_variants() {
        let mut h = ControlHandler::new(CFG);
        let a = h.setup(&pkt([0x80, 0x00, 0, 0, 0, 0, 2, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 2, zlp: false });
        assert_eq!(h.reply(), &[0, 0]);
        // Interface and data endpoint status need the Configured state.
        assert_eq!(h.setup(&pkt([0x81, 0x00, 0, 0, 0, 0, 2, 0])), ControlAction::Stall);
        assert_eq!(h.setup(&pkt([0x82, 0x00, 0, 0, 0x82, 0, 2, 0])), ControlAction::Stall);
        // Endpoint 0 status is always available.
        let a = h.setup(&pkt([0x82, 0x00, 0, 0, 0x80, 0, 2, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 2, zlp: false });
        let mut h = configured();
        let a = h.setup(&pkt([0x81, 0x00, 0, 0, 1, 0, 2, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 2, zlp: false });
        assert_eq!(h.setup(&pkt([0x81, 0x00, 0, 0, 2, 0, 2, 0])), ControlAction::Stall);
        let a = h.setup(&pkt([0x82, 0x00, 0, 0, 0x02, 0, 2, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 2, zlp: false });
        assert_eq!(h.reply(), &[0, 0]);
        assert_eq!(h.setup(&pkt([0x82, 0x00, 0, 0, 0x03, 0, 2, 0])), ControlAction::Stall);
    }

    #[test]
    fn endpoint_halt_set_status_clear() {
        let mut h = configured();
        assert_eq!(
            h.setup(&pkt([0x02, 0x03, 0x00, 0x00, 0x02, 0x00, 0, 0])),
            ControlAction::NoData(Effect::EndpointHalt { endpoint: EP_DATA_OUT, halted: true })
        );
        assert!(h.is_halted(EP_DATA_OUT));
        h.setup(&pkt([0x82, 0x00, 0, 0, 0x02, 0, 2, 0]));
        assert_eq!(h.reply(), &[1, 0]);
        // CLEAR_FEATURE always reports the toggle reset, halted or not.
        assert_eq!(
            h.setup(&pkt([0x02, 0x01, 0x00, 0x00, 0x02, 0x00, 0, 0])),
            ControlAction::NoData(Effect::EndpointHalt { endpoint: EP_DATA_OUT, halted: false })
        );
        assert_eq!(
            h.setup(&pkt([0x02, 0x01, 0x00, 0x00, 0x82, 0x00, 0, 0])),
            ControlAction::NoData(Effect::EndpointHalt { endpoint: EP_DATA_IN, halted: false })
        );
        assert!(!h.is_halted(EP_DATA_OUT));
        // Endpoint 0: accepted, no effect.
        assert_eq!(
            h.setup(&pkt([0x02, 0x03, 0x00, 0x00, 0x00, 0x00, 0, 0])),
            ControlAction::NoData(Effect::None)
        );
        // Non-existent endpoint, wrong feature, device features.
        assert_eq!(h.setup(&pkt([0x02, 0x03, 0x00, 0x00, 0x05, 0x00, 0, 0])), ControlAction::Stall);
        assert_eq!(h.setup(&pkt([0x02, 0x03, 0x01, 0x00, 0x02, 0x00, 0, 0])), ControlAction::Stall);
        assert_eq!(h.setup(&pkt([0x00, 0x03, 0x01, 0x00, 0, 0, 0, 0])), ControlAction::Stall);
        assert_eq!(h.setup(&pkt([0x00, 0x03, 0x02, 0x00, 0, 0x01, 0, 0])), ControlAction::Stall);
        // Before configuration the data endpoints do not exist.
        let mut h = ControlHandler::new(CFG);
        assert_eq!(h.setup(&pkt([0x02, 0x01, 0x00, 0x00, 0x82, 0x00, 0, 0])), ControlAction::Stall);
    }

    #[test]
    fn interface_requests() {
        let mut h = configured();
        let a = h.setup(&pkt([0x81, 0x0A, 0, 0, 1, 0, 1, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 1, zlp: false });
        assert_eq!(h.reply(), &[0]);
        assert_eq!(
            h.setup(&pkt([0x01, 0x0B, 0, 0, 1, 0, 0, 0])),
            ControlAction::NoData(Effect::InterfaceReset(1))
        );
        assert_eq!(h.setup(&pkt([0x01, 0x0B, 1, 0, 1, 0, 0, 0])), ControlAction::Stall);
        assert_eq!(h.setup(&pkt([0x01, 0x0B, 0, 0, 2, 0, 0, 0])), ControlAction::Stall);
    }

    #[test]
    fn line_coding_and_control_lines() {
        let mut h = configured();
        // GET_LINE_CODING before any SET returns the default.
        let a = h.setup(&pkt([0xA1, 0x21, 0, 0, 0, 0, 7, 0]));
        assert_eq!(a, ControlAction::DataIn { len: 7, zlp: false });
        assert_eq!(h.reply(), &LineCoding::DEFAULT.to_bytes());
        // SET_LINE_CODING 9600 8N1.
        let a = h.setup(&pkt([0x21, 0x20, 0, 0, 0, 0, 7, 0]));
        assert_eq!(a, ControlAction::DataOut { len: 7 });
        assert_eq!(h.data_out(&[0x80, 0x25, 0, 0, 0, 0, 8]), ControlAction::NoData(Effect::None));
        let lc = h.line_coding();
        assert_eq!((lc.baud, lc.stop_bits, lc.parity, lc.data_bits), (9600, StopBits::One, Parity::None, 8));
        // Invalid data stalls and leaves the old value.
        h.setup(&pkt([0x21, 0x20, 0, 0, 0, 0, 7, 0]));
        assert_eq!(h.data_out(&[0, 0, 0, 0, 9, 0, 8]), ControlAction::Stall);
        assert_eq!(h.line_coding().baud, 9600);
        // Data with nothing pending stalls.
        assert_eq!(h.data_out(&[0; 7]), ControlAction::Stall);
        // Wrong length.
        assert_eq!(h.setup(&pkt([0x21, 0x20, 0, 0, 0, 0, 8, 0])), ControlAction::Stall);
        // DTR on, then DTR+RTS, then off.
        assert_eq!(h.setup(&pkt([0x21, 0x22, 1, 0, 0, 0, 0, 0])), ControlAction::NoData(Effect::None));
        assert!(h.control_line_state().dtr && !h.control_line_state().rts);
        h.setup(&pkt([0x21, 0x22, 3, 0, 0, 0, 0, 0]));
        assert!(h.control_line_state().dtr && h.control_line_state().rts);
        h.setup(&pkt([0x21, 0x22, 0, 0, 0, 0, 0, 0]));
        assert_eq!(h.control_line_state(), ControlLineState::default());
        // SEND_BREAK accepted.
        assert_eq!(h.setup(&pkt([0x21, 0x23, 0xff, 0xff, 0, 0, 0, 0])), ControlAction::NoData(Effect::None));
        // Class requests to the data interface, or unknown ones, stall.
        assert_eq!(h.setup(&pkt([0x21, 0x22, 1, 0, 1, 0, 0, 0])), ControlAction::Stall);
        assert_eq!(h.setup(&pkt([0x21, 0x00, 0, 0, 0, 0, 4, 0])), ControlAction::Stall);
        assert_eq!(h.setup(&pkt([0xA1, 0x01, 0, 0, 0, 0, 4, 0])), ControlAction::Stall);
    }

    #[test]
    fn a_new_setup_cancels_pending_out() {
        let mut h = configured();
        h.setup(&pkt([0x21, 0x20, 0, 0, 0, 0, 7, 0]));
        h.setup(&pkt([0x80, 0x06, 0x00, 0x01, 0, 0, 18, 0]));
        assert_eq!(h.data_out(&[0x80, 0x25, 0, 0, 0, 0, 8]), ControlAction::Stall);
        assert_eq!(h.line_coding(), LineCoding::DEFAULT);
    }

    #[test]
    fn vendor_and_unknown_requests_stall() {
        let mut h = configured();
        assert_eq!(h.setup(&pkt([0xC0, 0x01, 0, 0, 0, 0, 4, 0])), ControlAction::Stall);
        assert_eq!(h.setup(&pkt([0x40, 0x01, 0, 0, 0, 0, 0, 0])), ControlAction::Stall);
        assert_eq!(h.setup(&pkt([0x00, 0x07, 0, 0x01, 0, 0, 18, 0])), ControlAction::Stall);
        assert_eq!(h.setup(&pkt([0x82, 0x0C, 0, 0, 0x82, 0, 2, 0])), ControlAction::Stall);
        assert_eq!(h.setup(&pkt([0x60, 0x00, 0, 0, 0, 0, 0, 0])), ControlAction::Stall);
    }

    #[test]
    fn device_to_host_with_zero_wlength_is_no_data() {
        let mut h = ControlHandler::new(CFG);
        assert_eq!(
            h.setup(&pkt([0x80, 0x06, 0x00, 0x01, 0, 0, 0, 0])),
            ControlAction::NoData(Effect::None)
        );
        assert_eq!(h.reply(), &[] as &[u8]);
    }

    #[test]
    fn bus_reset_returns_to_default() {
        let mut h = configured();
        h.setup(&pkt([0x21, 0x22, 1, 0, 0, 0, 0, 0]));
        h.setup(&pkt([0x02, 0x03, 0x00, 0x00, 0x02, 0x00, 0, 0]));
        h.bus_reset();
        assert_eq!(h.state(), DeviceState::Default);
        assert_eq!(h.configuration(), 0);
        assert!(!h.is_halted(EP_DATA_OUT));
        assert!(!h.control_line_state().dtr);
        assert_eq!(h.config(), &CFG);
    }
}
