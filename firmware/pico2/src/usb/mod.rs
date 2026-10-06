//! A polled USB CDC-ACM device ("USB serial port") on the RP2350's USB
//! controller.
//!
//! The protocol logic — descriptors, the endpoint-0 request state machine,
//! line coding, the ZLP rules — is chip-independent and host-tested in
//! [`api::usb`]. This module is the hardware half: it moves packets
//! between the controller's dual-port RAM and that logic, and applies the
//! [`Effect`]s it asks for.
//!
//! # Using it
//!
//! ```ignore
//! let clocks = Rp2350Clocks::new(board.clocks);
//! let timer = Rp2350Timer::new(board.timer, &clocks);
//! let mut usb = Rp2350Usb::new(board.usb, &clocks, &timer, UsbDeviceConfig {
//!     vendor_id: 0x2e8a,
//!     product_id: 0x000a,
//!     device_release: 0x0100,
//!     manufacturer: "Example",
//!     product: "Example serial",
//!     serial_number: "0001",
//! });
//! let mut seen = usb.link_epoch();
//! let mut buf = [0u8; 64];
//! loop {
//!     usb.poll();                       // at least once per millisecond
//!     if usb.link_epoch() != seen {     // bus reset, suspend, unplug, deconfigure
//!         seen = usb.link_epoch();
//!         // drop any session state here
//!     }
//!     let n = usb.read(&mut buf);
//!     let _sent = usb.write(&buf[..n]); // may accept fewer bytes than offered
//! }
//! ```
//!
//! # Bring-up requirements
//!
//! * **Reset.** The controller is `USBCTRL`, bit 28 of `RESETS.RESET`
//!   (Table 534, p504). [`Rp2350Usb::new`] puts it through a full reset
//!   cycle, so it starts from reset values even after a debugger warm
//!   reset.
//! * **`clk_usb` at exactly 48 MHz** (§12.7.3.1, p1142), from `PLL_USB`,
//!   which [`crate::clocks`] sets up. The constructor takes
//!   `&Rp2350Clocks` as proof.
//! * **`clk_sys` at least 10 % faster than `clk_usb`.** Erratum RP2350-E12
//!   (p1366–1367): several controller event flags, `SETUP_REC` among them,
//!   cross from `clk_usb` to `clk_sys` without proper synchronisation and
//!   can be lost when `clk_sys` is not faster. 150 MHz against 48 MHz
//!   satisfies this; a compile-time assertion below keeps it that way.
//! * **A running timer**, for the 10 ms disconnect pulse in `new`.
//!
//! So the order is: clocks, timer, USB. The watchdog may come before or
//! after, but `new` blocks for about 10 ms, so a watchdog that is already
//! running needs a longer timeout than that.
//!
//! `ACCESSCTRL` lets Secure code on either core reach `USBCTRL` out of
//! reset (Table 929, p838), which is where this image runs.
//!
//! # Controller set-up, step by step ([`Rp2350Usb::new`])
//!
//! 1. Reset cycle `USBCTRL`. The DPRAM may only be accessed once the
//!    controller is out of reset (§12.7.1.1.3, p1140).
//! 2. Zero all 4 kB of DPRAM with 32-bit writes. Reset does not clear it,
//!    and stale endpoint or buffer control words from a previous run must
//!    not survive.
//! 3. `USB_MUXING = TO_PHY | SOFTCON`: connect the controller to the
//!    on-chip PHY (as in the device example, §12.7.4.2.1, p1154).
//! 4. `USB_PWR`: force VBUS detect — first the value, then the override
//!    enable, the order Table 1211's description (p1170) gives. The Pico 2
//!    senses VBUS on GPIO24, but GPIO24 has no "USB VBUS DET" function
//!    (its F10 is "USB OVCUR DET", §9.4, p591), so the controller cannot
//!    see it. Forcing detect is the documented alternative: "VBUS detect
//!    can be forced in USB_PWR" (§12.7.3.10, p1153).
//! 5. `MAIN_CTRL = CONTROLLER_EN`: enable the controller in device mode
//!    and, in the same write, clear `PHY_ISO`. The PHY comes out of reset
//!    isolated, and RP2040 code that forgets to clear this bit does not
//!    work on RP2350 (§12.7.2, p1140; Table 1197, p1160).
//! 6. `SIE_CTRL = EP0_INT_1BUF`: report every completed EP0 buffer in
//!    `BUFF_STATUS`. Writing the whole register also clears `PULLDOWN_EN`,
//!    which resets to 1 on RP2350 to match the PHY's isolation latches
//!    (§12.7.2.2.1, p1141) but is a host-mode setting. `PULLUP_EN` stays 0.
//! 7. `INTE = 0` and `ADDR_ENDP = 0`.
//! 8. Wait 10 ms with the pull-up off. Until step 5 the isolated PHY may
//!    still be holding a previous run's pull-up; with it now released,
//!    this gap makes the host see a disconnect, so it re-enumerates from
//!    scratch instead of talking to a device that has forgotten its
//!    address. The length is a choice — USB detects a disconnect after
//!    about 2.5 µs of SE0 — and is unverified on hardware.
//! 9. Set `SIE_CTRL.PULLUP_EN` through the atomic set alias. The host now
//!    sees a full-speed device and starts with a bus reset.
//!
//! `LINESTATE_TUNING` is left at its reset value as §12.7.2 (p1140)
//! recommends. That value enables every RP2350 device-side fix, including
//! the RP2040-E15 bit-stuff hang fix (§12.7.2.2.4, p1141).
//!
//! # Endpoints
//!
//! | Endpoint | Type | DPRAM buffer | Use |
//! |----------|------|--------------|-----|
//! | EP0 IN/OUT | control | `0x100` (shared) | enumeration, CDC requests |
//! | EP1 IN (`0x81`) | interrupt | `0x180` | CDC notifications: never armed, so it always NAKs |
//! | EP2 OUT (`0x02`) | bulk | `0x1c0` | bytes from the host |
//! | EP2 IN (`0x82`) | bulk | `0x200` | bytes to the host |
//!
//! All are single-buffered. A buffer is handed to the controller by
//! writing its buffer control word with `AVAILABLE` set. When the
//! transaction is done the controller clears `AVAILABLE` and sets the
//! endpoint's bit in `BUFF_STATUS`.
//!
//! **One ordering rule matters for polling.** §12.7.3.8.2 (p1149) lists the
//! completion steps in this order: step 3 sets the `BUFF_STATUS` bit, then
//! step 4 writes the length, PID and cleared `AVAILABLE` back into the
//! buffer control word. A fast processor can therefore see the
//! `BUFF_STATUS` bit before the write-back has happened. This driver
//! handles a completion only once `AVAILABLE` reads back 0. Until then it
//! leaves the `BUFF_STATUS` bit set and looks again on the next poll.
//!
//! **Revoking a buffer.** To take back a buffer that is still `AVAILABLE`
//! — the host halts an endpoint, clears a halt, or reconfigures — the
//! driver uses `EP_ABORT` (Table 1205, p1166): the controller NAKs the
//! endpoint, sets `EP_ABORT_DONE` (Table 1206, p1167) once no transaction
//! is in flight, and the buffer control word can then be rewritten
//! safely. The RP2040 erratum that left aborts stuck (RP2040-E2) is fixed
//! on RP2350 (§12.7.2.1, p1141). The wait for `EP_ABORT_DONE` is bounded
//! (about 0.15–0.5 ms); if it runs out the driver carries on regardless,
//! which is unverified on hardware.
//!
//! # Data toggles
//!
//! The controller takes each packet's PID from the buffer control word and
//! does not track toggles itself, so this driver does:
//!
//! * EP0: the first data packet after SETUP is DATA1, alternating after
//!   that, and every status packet is DATA1.
//! * EP2 IN/OUT: DATA0 after SET_CONFIGURATION, after
//!   CLEAR_FEATURE(ENDPOINT_HALT) and after SET_INTERFACE, alternating per
//!   completed packet. An EP2 IN packet that was armed but not yet sent
//!   when the toggle is reset is re-armed as DATA0 rather than lost.
//!
//! An OUT packet whose PID does not match the buffer control word is
//! rejected by the controller (`SIE_STATUS.DATA_SEQ_ERROR`, §12.7.3.8.3,
//! p1149–1150).
//!
//! # Endpoint 0 and the status stage
//!
//! For a control read (data stage IN), the OUT status buffer is armed
//! together with the *last* IN packet, not after it completes. That allows
//! one round trip fewer per request under polling. It also means a host
//! that moves to the status stage early — for example because the ACK of
//! the last data packet was lost — is answered rather than NAKed until it
//! gives up. If the status OUT completes while an IN packet is still
//! armed, that IN buffer is revoked.
//!
//! **Residual hazard, inherent to polling, unverified on hardware.** A
//! stale IN buffer that is still armed when a new SETUP arrives can be
//! sent as the first data packet of the new request before the next poll
//! revokes it. The flow above only leaves one armed in that way if the
//! host abandons a transfer mid-way.
//!
//! STALL on EP0 needs both `EP_STALL_ARM` and the buffer control STALL bit.
//! The controller clears `EP_STALL_ARM` on the next SETUP, as USB requires
//! (§12.7.3.8.1, p1148–1149; Table 1207, p1168).
//!
//! # Link state
//!
//! [`Rp2350Usb::link_epoch`] counts link-lost events
//! ([`LinkEvent`]): bus reset, suspend, disconnect and deconfiguration. An
//! application compares it with the value it saw last to learn that
//! whatever was on the other end may have gone away.
//!
//! * **Suspend** is `SIE_STATUS.SUSPENDED`. To catch a whole suspend and
//!   resume that happens between two polls, the driver also watches
//!   `INTR.DEV_SUSPEND`, a latch that sets on any change of the suspend
//!   state. It clears the latch by writing `SIE_STATUS.SUSPENDED` (Table
//!   1216, p1174). It infers a missed event only on a *new* rising edge of
//!   the latch, so if the clear did not work on real hardware the cost is
//!   at most one extra event, not a stream of them.
//! * **Disconnect.** Without VBUS detection, unplugging the cable looks
//!   exactly like a suspend: "Without VBUS detection, it is impossible to
//!   tell the difference between being disconnected and suspended"
//!   (§12.7.3.8.4, p1150). On this board an unplug is therefore reported
//!   as [`LinkEvent::Suspended`]. [`LinkEvent::Disconnected`] is reported
//!   if `SIE_STATUS.CONNECTED` falls (or its `INTR.DEV_CONN_DIS` latch
//!   shows it fell and rose between polls). The datasheet describes that
//!   bit only as "Device: connected", so whether it ever falls with VBUS
//!   forced is unverified. A board powered through VSYS that needs to tell
//!   the two apart can read `board.pins.vbus_sense` (GPIO24) with the GPIO
//!   driver. A USB-powered board loses power when unplugged, so the
//!   question never arises.
//! * **Bus reset** is `SIE_STATUS.BUS_RESET`. It returns the device to
//!   address 0 and unconfigured, and clears both byte buffers.
//! * **Deconfiguration** is SET_CONFIGURATION(0), or SET_CONFIGURATION(1)
//!   while already configured (a fresh session). The byte buffers are
//!   cleared.
//!
//! # What polling costs, and what it requires
//!
//! [`Rp2350Usb::poll`] must run at least once per millisecond:
//!
//! * After SET_ADDRESS the host waits 2 ms before using the new address
//!   (USB 2.0 §9.2.6.3). The address can only be programmed after the
//!   status stage completes, and only `poll` notices that.
//! * Every SETUP is answered by `poll`, and the host's control-transfer
//!   timeouts assume a responsive device.
//! * Each direction of EP2 moves at most one 64-byte packet per poll, so
//!   one poll per millisecond gives at least 64 kB/s each way.
//!
//! Slower polling makes the host retry (NAK) rather than lose data, but
//! enumeration may fail.
//!
//! Nothing in `poll`, [`read`](Rp2350Usb::read) or
//! [`write`](Rp2350Usb::write) waits for the host. The only wait is the
//! bounded `EP_ABORT_DONE` spin, which runs only while handling
//! SET_CONFIGURATION, halt or SET_INTERFACE requests.
//!
//! The controller's interrupt line is never used (`INTE` = 0, no NVIC
//! enable), so this driver needs no handler and cannot interrupt the
//! application. The device does not reduce its current draw while
//! suspended, which a USB-compliant bus-powered device would.
//!
//! Page numbers in this crate are PDF page indices of the RP2350 datasheet
//! (one more than the number printed in the page footer).

mod hw;

use api::common::{ErrorType, Read, Write};
use api::device::DeviceHandle;
use api::usb::bulk_in_needs_zlp;
use api::usb::cdc_acm::LineCoding;
use api::usb::control::{ControlAction, ControlHandler, Effect};
use api::usb::descriptor::{
    BULK_MAX_PACKET, EP0_MAX_PACKET, EP_DATA_IN, EP_DATA_OUT, EP_NOTIFY_IN, INTERFACE_COMM,
    INTERFACE_DATA, NOTIFY_MAX_PACKET,
};
use api::usb::ring::ByteRing;
use api::usb::setup::SetupPacket;

/// The device identity and strings, given to [`Rp2350Usb::new`].
/// Re-exported from [`api::usb::descriptor`].
pub use api::usb::descriptor::UsbDeviceConfig;

use crate::clocks::{Rp2350Clocks, CLK_SYS_HZ, CLK_USB_HZ};
use crate::common::reset::cycle_reset;
use crate::timer::Rp2350Timer;

use hw::*;

/// Bytes buffered from the host before the driver stops accepting more.
/// Once fewer than 64 bytes are free, EP2 OUT is left unarmed and the
/// controller NAKs the host, so nothing is ever dropped.
pub const RX_BUFFER_SIZE: usize = 512;

/// Bytes [`Rp2350Usb::write`] can queue for the host.
pub const TX_BUFFER_SIZE: usize = 512;

/// `USBCTRL` — bit 28 of `RESETS.RESET` (Table 534, p504).
const RESET_USBCTRL: u32 = 1 << 28;

/// How long the pull-up stays off in [`Rp2350Usb::new`].
const DISCONNECT_MS: u32 = 10;

/// Iterations of the `EP_ABORT_DONE` wait. Each iteration is at least one
/// register read on the AHB bus plus a compare and branch: at least 3
/// cycles and in practice under 10. So this is roughly 0.15–0.5 ms at
/// 150 MHz. That is several times the longest full-speed transaction (a
/// 64-byte packet is about 50 µs on the wire).
const ABORT_SPIN_LIMIT: u32 = CLK_SYS_HZ / 20_000;

// RP2350-E12 (p1366–1367): clk_sys must be at least 10 % faster than
// clk_usb while the controller is in use.
const _: () = assert!(CLK_SYS_HZ as u64 * 10 >= CLK_USB_HZ as u64 * 11);
const _: () = assert!(CLK_USB_HZ == 48_000_000);
// Every packet this driver handles fits one 64-byte buffer.
const _: () = assert!(EP0_MAX_PACKET == 64 && BULK_MAX_PACKET == 64);
const _: () = assert!(NOTIFY_MAX_PACKET <= 64);
const _: () = assert!(AVAILABLE_DELAY_NOPS >= 1);

/// A reason the link to the host was lost. See the
/// [module documentation](self#link-state).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkEvent {
    /// The host reset the bus: the device is back at address 0 and
    /// unconfigured. Hosts do this on attach, on driver reload and to
    /// recover from errors.
    BusReset,
    /// The bus went into suspend (no start-of-frame for 3 ms): the host
    /// went to sleep, or the cable was pulled.
    Suspended,
    /// `SIE_STATUS.CONNECTED` fell.
    Disconnected,
    /// The host deconfigured the device, or configured it again while it
    /// was already configured.
    Deconfigured,
}

/// Why a byte-at-a-time [`Read`] or [`Write`] did not complete.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsbError {
    /// No byte to read, or no room to queue one: try again after
    /// [`Rp2350Usb::poll`].
    WouldBlock,
    /// The host has not configured the device, so no bytes can flow.
    NotConfigured,
}

/// Where endpoint 0 is in a control transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ep0Stage {
    /// Nothing in progress, or the last request was stalled.
    Idle,
    /// Sending the reply in [`ControlHandler::reply`]: `sent` of `len`
    /// bytes have been armed, a zero-length packet is still owed after
    /// them if `zlp`, and `data1` is the PID of the next packet. One IN
    /// packet is armed.
    DataIn { sent: usize, len: usize, zlp: bool, data1: bool },
    /// The last data packet and the OUT status packet are both armed;
    /// `in_done` once the IN packet has completed.
    StatusOut { in_done: bool },
    /// The OUT data stage is armed, `len` bytes expected.
    DataOut { len: usize },
    /// The IN status packet is armed; apply the effect once it completes.
    StatusIn(Effect),
}

/// The USB CDC-ACM device. See the [module documentation](self).
pub struct Rp2350Usb {
    control: ControlHandler,
    ep0: Ep0Stage,
    rx: ByteRing<RX_BUFFER_SIZE>,
    tx: ByteRing<TX_BUFFER_SIZE>,
    /// EP2 OUT is armed.
    out_armed: bool,
    /// PID the next EP2 OUT packet must carry.
    out_data1: bool,
    /// Length of the packet sitting in the EP2 IN buffer that the host has
    /// not yet acknowledged (0 for a ZLP), if any.
    in_packet: Option<usize>,
    /// That packet is currently handed to the controller.
    in_armed: bool,
    /// PID of the next EP2 IN packet.
    in_data1: bool,
    /// `SIE_STATUS.SUSPENDED` at the last poll.
    suspended: bool,
    /// `SIE_STATUS.CONNECTED` at the last poll.
    connected: bool,
    /// `INTR` change latches seen at the last poll.
    latches: u32,
    epoch: u32,
    last_event: Option<LinkEvent>,
}

impl Rp2350Usb {
    /// Bring up the controller and attach to the bus.
    ///
    /// Runs the sequence in the [module documentation](self) and returns
    /// with the pull-up on, so the host starts enumerating. Enumeration
    /// itself only happens while [`poll`](Self::poll) is called. Blocks for
    /// about 10 ms (the disconnect pulse).
    ///
    /// `&Rp2350Clocks` proves `clk_usb` runs at 48 MHz; `timer` provides
    /// the delay. `config` supplies the vendor and product IDs and the
    /// three strings the host displays.
    pub fn new(
        _handle: DeviceHandle<Rp2350Usb>,
        _clocks: &Rp2350Clocks,
        timer: &Rp2350Timer,
        config: UsbDeviceConfig,
    ) -> Self {
        unsafe {
            let regs = regs();
            // 1. Reset.
            cycle_reset(RESET_USBCTRL);

            // 2. Clear the DPRAM, 32 bits at a time.
            let words = dpram() as *mut u32;
            for i in 0..core::mem::size_of::<Dpram>() / 4 {
                words.add(i).write_volatile(0);
            }

            // 3. Controller to the on-chip PHY.
            (&raw mut (*regs).usb_muxing).write_volatile(MUXING_TO_PHY | MUXING_SOFTCON);

            // 4. Force VBUS detect: value first, then the override enable.
            (&raw mut (*regs).usb_pwr).write_volatile(PWR_VBUS_DETECT);
            (&raw mut (*regs).usb_pwr)
                .write_volatile(PWR_VBUS_DETECT | PWR_VBUS_DETECT_OVERRIDE_EN);

            // 5. Enable in device mode (HOST_NDEVICE = 0), PHY not isolated.
            (&raw mut (*regs).main_ctrl)
                .write_volatile(MAIN_CTRL_CONTROLLER_EN & !MAIN_CTRL_PHY_ISO);

            // 6. EP0 completions in BUFF_STATUS; no pull-down, no pull-up yet.
            (&raw mut (*regs).sie_ctrl).write_volatile(
                SIE_CTRL_EP0_INT_1BUF & !(SIE_CTRL_PULLDOWN_EN | SIE_CTRL_PULLUP_EN),
            );

            // 7. Polled: no interrupts. Address 0.
            (&raw mut (*regs).inte).write_volatile(0);
            (&raw mut (*regs).addr_endp).write_volatile(0);
        }

        // 8. Let the host see a disconnect.
        timer.delay_ms(DISCONNECT_MS);

        // 9. Attach.
        unsafe { (&raw mut (*regs_set()).sie_ctrl).write_volatile(SIE_CTRL_PULLUP_EN) };

        Self {
            control: ControlHandler::new(config),
            ep0: Ep0Stage::Idle,
            rx: ByteRing::new(),
            tx: ByteRing::new(),
            out_armed: false,
            out_data1: false,
            in_packet: None,
            in_armed: false,
            in_data1: false,
            suspended: false,
            connected: false,
            latches: 0,
            epoch: 0,
            last_event: None,
        }
    }

    // --- Application API ---------------------------------------------------

    /// Service the controller: track the bus state, handle a bus reset,
    /// collect completed packets, answer a SETUP, and keep the data
    /// endpoints armed.
    ///
    /// Call at least once per millisecond. Never waits for the host; see
    /// the [module documentation](self#what-polling-costs-and-what-it-requires).
    pub fn poll(&mut self) {
        self.track_bus_state();
        let status = unsafe { (&raw const (*regs()).sie_status).read_volatile() };
        if status & SIE_BUS_RESET != 0 {
            self.bus_reset();
        }
        // Completions before the SETUP: they belong to the transfer the
        // SETUP may be about to replace.
        self.handle_buffers();
        let status = unsafe { (&raw const (*regs()).sie_status).read_volatile() };
        if status & SIE_SETUP_REC != 0 {
            self.handle_setup();
        }
        self.service_rx();
        self.service_tx();
    }

    /// Move up to `buf.len()` received bytes into `buf`; returns how many.
    /// 0 means nothing is waiting. Never blocks.
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let n = self.rx.pop_slice(buf);
        if n > 0 {
            self.service_rx();
        }
        n
    }

    /// Queue bytes for the host; returns how many were accepted, which may
    /// be fewer than `data.len()` (or 0) when the transmit buffer is full.
    /// Never blocks.
    ///
    /// Returns 0 and queues nothing while the device is not configured:
    /// there is no session to deliver the bytes to, and the buffers are
    /// cleared when one starts.
    ///
    /// Bytes go out in 64-byte packets as the host asks for them. When the
    /// queue runs empty right after a full 64-byte packet, a zero-length
    /// packet follows, so the host's read completes immediately
    /// ([`bulk_in_needs_zlp`]).
    pub fn write(&mut self, data: &[u8]) -> usize {
        if !self.control.is_configured() {
            return 0;
        }
        let n = self.tx.push_slice(data);
        self.service_tx();
        n
    }

    /// True once the host has selected the configuration
    /// (SET_CONFIGURATION(1)): the serial port exists and bytes can flow.
    /// Stays true while the bus is suspended, as USB says it should.
    pub fn is_configured(&self) -> bool {
        self.control.is_configured()
    }

    /// True while the bus is suspended (no start-of-frame packets for
    /// 3 ms): the host is asleep or the cable is unplugged.
    pub fn is_suspended(&self) -> bool {
        self.suspended
    }

    /// Configured and not suspended: a host is there and talking to the
    /// device.
    pub fn link_up(&self) -> bool {
        self.is_configured() && !self.suspended
    }

    /// Count of link-lost events since [`new`](Self::new), wrapping.
    ///
    /// If it differs from the value seen before the last
    /// [`poll`](Self::poll), the link went down at least once in between —
    /// even if it is back up now. The first enumeration advances it too:
    /// every attach begins with a bus reset.
    pub fn link_epoch(&self) -> u32 {
        self.epoch
    }

    /// The most recent link-lost event, if any.
    pub fn last_link_event(&self) -> Option<LinkEvent> {
        self.last_event
    }

    /// The line coding (baud rate, framing) last set by the host. This
    /// driver does nothing with it: there is no real UART behind the port.
    pub fn line_coding(&self) -> LineCoding {
        self.control.line_coding()
    }

    /// DTR as last set by the host. Most hosts raise it when a program
    /// opens the port. Information only: the driver never acts on it.
    pub fn dtr(&self) -> bool {
        self.control.control_line_state().dtr
    }

    /// RTS as last set by the host. Information only.
    pub fn rts(&self) -> bool {
        self.control.control_line_state().rts
    }

    // --- Link tracking -----------------------------------------------------

    fn link_event(&mut self, event: LinkEvent) {
        self.epoch = self.epoch.wrapping_add(1);
        self.last_event = Some(event);
    }

    /// Follow `SIE_STATUS.SUSPENDED` and `CONNECTED`, using the `INTR`
    /// change latches to catch transitions that came and went between
    /// polls.
    fn track_bus_state(&mut self) {
        let (latched, status) = unsafe {
            let regs = regs();
            let latched = (&raw const (*regs).intr).read_volatile()
                & (INTR_DEV_SUSPEND | INTR_DEV_CONN_DIS);
            // Acknowledge the latches *before* sampling the levels, so a
            // change after the sample sets them again for the next poll.
            let mut ack = 0;
            if latched & INTR_DEV_SUSPEND != 0 {
                ack |= SIE_SUSPENDED;
            }
            if latched & INTR_DEV_CONN_DIS != 0 {
                ack |= SIE_CONNECTED;
            }
            if ack != 0 {
                (&raw mut (*regs_clr()).sie_status).write_volatile(ack);
            }
            (latched, (&raw const (*regs).sie_status).read_volatile())
        };
        // Only a latch that was clear at the previous poll counts as news;
        // see "Link state" in the module docs.
        let fresh = latched & !self.latches;
        self.latches = latched;

        let suspended = status & SIE_SUSPENDED != 0;
        if suspended && !self.suspended {
            self.link_event(LinkEvent::Suspended);
        } else if !suspended && !self.suspended && fresh & INTR_DEV_SUSPEND != 0 {
            // Suspended and resumed again since the last poll.
            self.link_event(LinkEvent::Suspended);
        }
        self.suspended = suspended;

        let connected = status & SIE_CONNECTED != 0;
        if !connected && self.connected {
            self.link_event(LinkEvent::Disconnected);
        } else if connected && self.connected && fresh & INTR_DEV_CONN_DIS != 0 {
            // Disconnected and connected again since the last poll.
            self.link_event(LinkEvent::Disconnected);
        }
        self.connected = connected;
    }

    /// The host reset the bus: back to address 0 with only EP0, as after
    /// [`new`](Self::new).
    ///
    /// No `EP_ABORT` is needed here: while the host is resetting the bus
    /// there are no transactions to race with.
    fn bus_reset(&mut self) {
        unsafe {
            let regs = regs();
            let dp = dpram();
            (&raw mut (*regs_clr()).sie_status).write_volatile(SIE_BUS_RESET);
            (&raw mut (*regs).addr_endp).write_volatile(0);
            (&raw mut (*regs_clr()).ep_stall_arm).write_volatile(STALL_ARM_EP0);
            for word in [
                &raw mut (*dp).ep_ctrl[0].dir_in,
                &raw mut (*dp).ep_ctrl[1].dir_out,
                &raw mut (*dp).ep_ctrl[1].dir_in,
                &raw mut (*dp).buf_ctrl[0].dir_in,
                &raw mut (*dp).buf_ctrl[0].dir_out,
                &raw mut (*dp).buf_ctrl[1].dir_in,
                &raw mut (*dp).buf_ctrl[2].dir_out,
                &raw mut (*dp).buf_ctrl[2].dir_in,
            ] {
                word.write_volatile(0);
            }
            (&raw mut (*regs_clr()).ep_abort).write_volatile(u32::MAX);
            (&raw mut (*regs_clr()).ep_abort_done).write_volatile(u32::MAX);
            (&raw mut (*regs_clr()).buff_status).write_volatile(u32::MAX);
        }
        self.control.bus_reset();
        self.ep0 = Ep0Stage::Idle;
        self.forget_data_endpoints();
        self.link_event(LinkEvent::BusReset);
    }

    /// Reset the software side of EP1/EP2 and empty both byte buffers.
    fn forget_data_endpoints(&mut self) {
        self.rx.clear();
        self.tx.clear();
        self.out_armed = false;
        self.out_data1 = false;
        self.in_packet = None;
        self.in_armed = false;
        self.in_data1 = false;
    }

    // --- Completed buffers -------------------------------------------------

    /// If `BUFF_STATUS` reports a completion for this endpoint direction
    /// and its write-back has landed, clear the `BUFF_STATUS` bit and
    /// return the buffer control word — but only if `armed` (this driver
    /// gave the buffer to the controller). A bit for a buffer that was not
    /// armed is stale and is just cleared.
    fn take_completion(status: u32, bit: u32, bc: *const u32, armed: bool) -> Option<u32> {
        if status & bit == 0 {
            return None;
        }
        unsafe {
            let word = bc.read_volatile();
            if armed && word & BC_AVAILABLE != 0 {
                // The write-back has not happened yet (§12.7.3.8.2, steps
                // 3 then 4, p1149). Look again next poll.
                return None;
            }
            (&raw mut (*regs_clr()).buff_status).write_volatile(bit);
            if armed { Some(word) } else { None }
        }
    }

    fn handle_buffers(&mut self) {
        let status = unsafe { (&raw const (*regs()).buff_status).read_volatile() };
        if status == 0 {
            return;
        }
        let (ep0_in, ep0_out, ep1_in, ep2_out, ep2_in) = unsafe {
            let dp = dpram();
            (
                &raw const (*dp).buf_ctrl[0].dir_in,
                &raw const (*dp).buf_ctrl[0].dir_out,
                &raw const (*dp).buf_ctrl[1].dir_in,
                &raw const (*dp).buf_ctrl[2].dir_out,
                &raw const (*dp).buf_ctrl[2].dir_in,
            )
        };
        // EP0 IN before EP0 OUT: when both the last data packet and the
        // status packet complete between two polls, they did so in that
        // order. What is armed is re-read after each step, because
        // handling one completion can arm the next buffer.
        if Self::take_completion(status, buff_bit(0, true), ep0_in, self.ep0_armed().0).is_some() {
            self.ep0_in_complete();
        }
        if let Some(word) = Self::take_completion(status, buff_bit(0, false), ep0_out, self.ep0_armed().1) {
            self.ep0_out_complete(word);
        }
        // EP1 IN is never armed; a bit here could only be stale.
        let _ = Self::take_completion(status, buff_bit(1, true), ep1_in, false);
        if let Some(word) = Self::take_completion(status, buff_bit(2, false), ep2_out, self.out_armed) {
            self.ep2_out_complete(word);
        }
        if Self::take_completion(status, buff_bit(2, true), ep2_in, self.in_armed).is_some() {
            self.ep2_in_complete();
        }
    }

    /// Which EP0 buffers (IN, OUT) the current stage has armed.
    fn ep0_armed(&self) -> (bool, bool) {
        match self.ep0 {
            Ep0Stage::Idle => (false, false),
            Ep0Stage::DataIn { .. } => (true, false),
            Ep0Stage::StatusOut { in_done } => (!in_done, true),
            Ep0Stage::DataOut { .. } => (false, true),
            Ep0Stage::StatusIn(_) => (true, false),
        }
    }

    /// Received data from the host on EP2 OUT.
    fn ep2_out_complete(&mut self, word: u32) {
        self.out_armed = false;
        self.out_data1 = !self.out_data1;
        let len = ((word & BC_LEN_MASK) as usize).min(BULK_MAX_PACKET);
        let mut packet = [0u8; BULK_MAX_PACKET];
        unsafe { copy_from_dpram(&mut packet[..len], (&raw const (*dpram()).ep2_out_buf).cast()) };
        // Cannot fall short: OUT is only armed with a full packet of room.
        self.rx.push_slice(&packet[..len]);
    }

    /// The host acknowledged the packet in the EP2 IN buffer.
    fn ep2_in_complete(&mut self) {
        let len = self.in_packet.take().unwrap_or(0);
        self.in_armed = false;
        self.in_data1 = !self.in_data1;
        if bulk_in_needs_zlp(len, BULK_MAX_PACKET, !self.tx.is_empty()) {
            self.in_packet = Some(0);
        }
    }

    // --- Data endpoints ----------------------------------------------------

    /// Arm EP2 OUT if the device is configured, the endpoint is not halted
    /// and a whole packet fits in the receive buffer.
    fn service_rx(&mut self) {
        if self.out_armed
            || !self.control.is_configured()
            || self.control.is_halted(EP_DATA_OUT)
            || self.rx.free() < BULK_MAX_PACKET
        {
            return;
        }
        let pid = if self.out_data1 { BC_DATA1 } else { 0 };
        unsafe {
            write_buffer_control(
                &raw mut (*dpram()).buf_ctrl[2].dir_out,
                pid | BC_AVAILABLE | BULK_MAX_PACKET as u32,
            );
        }
        self.out_armed = true;
    }

    /// Arm EP2 IN with the pending packet, or with the next one from the
    /// transmit buffer.
    fn service_tx(&mut self) {
        if self.in_armed || !self.control.is_configured() || self.control.is_halted(EP_DATA_IN) {
            return;
        }
        let dp = dpram();
        let len = match self.in_packet {
            Some(len) => len,
            None => {
                if self.tx.is_empty() {
                    return;
                }
                let mut packet = [0u8; BULK_MAX_PACKET];
                let len = self.tx.pop_slice(&mut packet);
                unsafe { copy_to_dpram((&raw mut (*dp).ep2_in_buf).cast(), &packet[..len]) };
                self.in_packet = Some(len);
                len
            }
        };
        let pid = if self.in_data1 { BC_DATA1 } else { 0 };
        unsafe {
            write_buffer_control(
                &raw mut (*dp).buf_ctrl[2].dir_in,
                BC_FULL | pid | BC_AVAILABLE | len as u32,
            );
        }
        self.in_armed = true;
    }

    /// Make an endpoint's endpoint and buffer control words safe to
    /// rewrite: abort it and wait (bounded) for `EP_ABORT_DONE`, then take
    /// in any transaction that completed before the abort.
    fn abort_begin(&mut self, mask: u32) {
        unsafe {
            let regs = regs();
            (&raw mut (*regs_set()).ep_abort).write_volatile(mask);
            let done = &raw const (*regs).ep_abort_done;
            let mut spins = ABORT_SPIN_LIMIT;
            while done.read_volatile() & mask != mask && spins > 0 {
                spins -= 1;
            }
        }
        // A transaction that finished before the abort took effect is
        // real: the host saw it acknowledged. Account for it now, or it
        // would be lost when the buffer control word is rewritten.
        self.handle_buffers();
    }

    /// Hand aborted endpoints back to the controller. `BUFF_STATUS` is
    /// cleared for them first, so nothing from before the abort is
    /// mistaken for a completion afterwards.
    fn abort_end(mask: u32) {
        unsafe {
            (&raw mut (*regs_clr()).buff_status).write_volatile(mask);
            (&raw mut (*regs_clr()).ep_abort).write_volatile(mask);
            (&raw mut (*regs_clr()).ep_abort_done).write_volatile(mask);
        }
    }

    /// Enable EP1 IN, EP2 OUT and EP2 IN with DATA0 toggles. Their buffer
    /// control words are already 0 (from `new`, bus reset or
    /// [`disable_data_endpoints`](Self::disable_data_endpoints)); EP2 OUT
    /// is armed by the next [`service_rx`](Self::service_rx).
    fn enable_data_endpoints(&mut self) {
        let ep = |kind: u32, buf: usize| {
            EP_CTRL_ENABLE | EP_CTRL_INT_PER_BUFFER | (kind << EP_CTRL_TYPE_SHIFT) | buf as u32
        };
        unsafe {
            let dp = dpram();
            (&raw mut (*dp).ep_ctrl[0].dir_in)
                .write_volatile(ep(EP_TYPE_INTERRUPT, core::mem::offset_of!(Dpram, ep1_in_buf)));
            (&raw mut (*dp).ep_ctrl[1].dir_out)
                .write_volatile(ep(EP_TYPE_BULK, core::mem::offset_of!(Dpram, ep2_out_buf)));
            (&raw mut (*dp).ep_ctrl[1].dir_in)
                .write_volatile(ep(EP_TYPE_BULK, core::mem::offset_of!(Dpram, ep2_in_buf)));
        }
        self.forget_data_endpoints();
    }

    /// Disable EP1 IN, EP2 OUT and EP2 IN and revoke their buffers.
    fn disable_data_endpoints(&mut self) {
        let mask = buff_bit(1, true) | buff_bit(2, false) | buff_bit(2, true);
        self.abort_begin(mask);
        unsafe {
            let dp = dpram();
            for word in [
                &raw mut (*dp).ep_ctrl[0].dir_in,
                &raw mut (*dp).ep_ctrl[1].dir_out,
                &raw mut (*dp).ep_ctrl[1].dir_in,
                &raw mut (*dp).buf_ctrl[1].dir_in,
                &raw mut (*dp).buf_ctrl[2].dir_out,
                &raw mut (*dp).buf_ctrl[2].dir_in,
            ] {
                word.write_volatile(0);
            }
        }
        Self::abort_end(mask);
        self.forget_data_endpoints();
    }

    /// Reset one data endpoint's toggle to DATA0 and apply its current
    /// halt state: SET/CLEAR_FEATURE(ENDPOINT_HALT) and SET_INTERFACE.
    ///
    /// A halted endpoint gets the STALL bit in its buffer control word.
    /// `EP_STALL_ARM` is only for EP0 (Table 1207, p1168). An EP2 IN packet
    /// that was armed but not yet sent stays in the DPRAM buffer and is
    /// re-armed as DATA0 by [`service_tx`](Self::service_tx) once the
    /// endpoint is not halted.
    fn reset_endpoint(&mut self, endpoint: u8) {
        let (n, is_in) = match endpoint {
            EP_NOTIFY_IN => (1, true),
            EP_DATA_OUT => (2, false),
            EP_DATA_IN => (2, true),
            _ => return,
        };
        let mask = buff_bit(n, is_in);
        self.abort_begin(mask);
        let halted = self.control.is_halted(endpoint);
        unsafe {
            let dp = dpram();
            let word = if is_in {
                &raw mut (*dp).buf_ctrl[n].dir_in
            } else {
                &raw mut (*dp).buf_ctrl[n].dir_out
            };
            write_buffer_control(word, if halted { BC_STALL } else { 0 });
        }
        Self::abort_end(mask);
        match endpoint {
            EP_DATA_OUT => {
                self.out_armed = false;
                self.out_data1 = false;
            }
            EP_DATA_IN => {
                self.in_armed = false;
                self.in_data1 = false;
            }
            _ => {}
        }
    }

    // --- Endpoint 0 --------------------------------------------------------

    /// A SETUP packet has arrived: drop whatever EP0 was doing and answer
    /// the new request.
    fn handle_setup(&mut self) {
        let mut bytes = [0u8; 8];
        unsafe {
            let dp = dpram();
            // Clear SETUP_REC before reading the packet: a SETUP arriving
            // during the read sets it again and is handled next poll.
            // Clearing after the read could swallow that newer SETUP.
            (&raw mut (*regs_clr()).sie_status).write_volatile(SIE_SETUP_REC);
            copy_from_dpram(&mut bytes, (&raw const (*dp).setup_packet).cast());
            // Revoke both EP0 buffers and forget their completions: a new
            // SETUP aborts any control transfer in progress (USB 2.0
            // §8.5.3).
            (&raw mut (*dp).buf_ctrl[0].dir_in).write_volatile(0);
            (&raw mut (*dp).buf_ctrl[0].dir_out).write_volatile(0);
            (&raw mut (*regs_clr()).buff_status).write_volatile(buff_bit(0, true) | buff_bit(0, false));
        }
        self.ep0 = Ep0Stage::Idle;
        let action = self.control.setup(&SetupPacket::from_bytes(bytes));
        self.run_action(action);
    }

    /// Carry out what the request handler decided.
    fn run_action(&mut self, action: ControlAction) {
        match action {
            ControlAction::DataIn { len, zlp } => {
                self.ep0 = Ep0Stage::DataIn { sent: 0, len, zlp, data1: true };
                self.ep0_send_next();
            }
            ControlAction::DataOut { len } if len <= EP0_MAX_PACKET => {
                // The handler only asks for single-packet OUT data stages
                // (SET_LINE_CODING, 7 bytes). The first data packet after
                // SETUP is DATA1.
                unsafe {
                    write_buffer_control(
                        &raw mut (*dpram()).buf_ctrl[0].dir_out,
                        BC_DATA1 | BC_AVAILABLE | EP0_MAX_PACKET as u32,
                    );
                }
                self.ep0 = Ep0Stage::DataOut { len };
            }
            ControlAction::NoData(effect) => {
                self.apply_effect(effect);
                self.ep0_arm_status_in(effect);
            }
            ControlAction::DataOut { .. } | ControlAction::Stall => self.ep0_stall(),
        }
    }

    /// Arm the next IN data packet of a control read. If it is the last,
    /// arm the OUT status packet with it.
    fn ep0_send_next(&mut self) {
        let Ep0Stage::DataIn { sent, len, zlp, data1 } = self.ep0 else {
            return;
        };
        let (chunk, zlp) = if sent < len {
            ((len - sent).min(EP0_MAX_PACKET), zlp)
        } else {
            (0, false) // the owed zero-length packet
        };
        let sent = sent + chunk;
        let last = sent == len && !zlp;
        unsafe {
            let dp = dpram();
            copy_to_dpram((&raw mut (*dp).ep0_buf).cast(), &self.control.reply()[sent - chunk..sent]);
            if last {
                // Status stage: zero-length OUT, DATA1.
                write_buffer_control(&raw mut (*dp).buf_ctrl[0].dir_out, BC_DATA1 | BC_AVAILABLE);
            }
            let pid = if data1 { BC_DATA1 } else { 0 };
            write_buffer_control(
                &raw mut (*dp).buf_ctrl[0].dir_in,
                BC_FULL | pid | BC_AVAILABLE | chunk as u32,
            );
        }
        self.ep0 = if last {
            Ep0Stage::StatusOut { in_done: false }
        } else {
            Ep0Stage::DataIn { sent, len, zlp, data1: !data1 }
        };
    }

    /// Arm the zero-length IN status packet (DATA1) and remember the
    /// effect to apply once it completes.
    fn ep0_arm_status_in(&mut self, effect: Effect) {
        unsafe {
            write_buffer_control(
                &raw mut (*dpram()).buf_ctrl[0].dir_in,
                BC_FULL | BC_DATA1 | BC_AVAILABLE,
            );
        }
        self.ep0 = Ep0Stage::StatusIn(effect);
    }

    /// STALL both directions of EP0 until the next SETUP.
    fn ep0_stall(&mut self) {
        unsafe {
            let dp = dpram();
            (&raw mut (*regs_set()).ep_stall_arm).write_volatile(STALL_ARM_EP0);
            write_buffer_control(&raw mut (*dp).buf_ctrl[0].dir_in, BC_STALL);
            write_buffer_control(&raw mut (*dp).buf_ctrl[0].dir_out, BC_STALL);
        }
        self.ep0 = Ep0Stage::Idle;
    }

    fn ep0_in_complete(&mut self) {
        match self.ep0 {
            Ep0Stage::DataIn { .. } => self.ep0_send_next(),
            Ep0Stage::StatusOut { in_done: false } => {
                self.ep0 = Ep0Stage::StatusOut { in_done: true };
            }
            Ep0Stage::StatusIn(effect) => {
                if let Effect::SetAddress(address) = effect {
                    // Only now: the status stage had to complete at the old
                    // address (USB 2.0 §9.4.6; ADDR_ENDP, Table 1195,
                    // p1159).
                    unsafe { (&raw mut (*regs()).addr_endp).write_volatile(address as u32) };
                }
                self.ep0 = Ep0Stage::Idle;
            }
            _ => {}
        }
    }

    fn ep0_out_complete(&mut self, word: u32) {
        match self.ep0 {
            Ep0Stage::StatusOut { in_done } => {
                if !in_done {
                    // The host went to the status stage with an IN packet
                    // still armed: take it back.
                    unsafe { (&raw mut (*dpram()).buf_ctrl[0].dir_in).write_volatile(0) };
                    unsafe { (&raw mut (*regs_clr()).buff_status).write_volatile(buff_bit(0, true)) };
                }
                self.ep0 = Ep0Stage::Idle;
            }
            Ep0Stage::DataOut { len } => {
                let received = ((word & BC_LEN_MASK) as usize).min(len);
                let mut data = [0u8; EP0_MAX_PACKET];
                unsafe { copy_from_dpram(&mut data[..received], (&raw const (*dpram()).ep0_buf).cast()) };
                let action = self.control.data_out(&data[..received]);
                self.run_action(action);
            }
            _ => {}
        }
    }

    /// Apply the immediate part of an [`Effect`]. `SetAddress` waits for
    /// the status stage; see [`ep0_in_complete`](Self::ep0_in_complete).
    fn apply_effect(&mut self, effect: Effect) {
        match effect {
            Effect::None | Effect::SetAddress(_) => {}
            Effect::ConfigurationChanged { from, to } => {
                if from != 0 {
                    self.disable_data_endpoints();
                    self.link_event(LinkEvent::Deconfigured);
                }
                if to != 0 {
                    self.enable_data_endpoints();
                }
            }
            Effect::EndpointHalt { endpoint, .. } => self.reset_endpoint(endpoint),
            Effect::InterfaceReset(interface) => {
                if interface == INTERFACE_COMM {
                    self.reset_endpoint(EP_NOTIFY_IN);
                } else if interface == INTERFACE_DATA {
                    self.reset_endpoint(EP_DATA_OUT);
                    self.reset_endpoint(EP_DATA_IN);
                }
            }
        }
    }
}

/// Byte-at-a-time access through the portable traits, for code written
/// against [`api::common`]. Note that `usb.read(&mut buf)` and
/// `usb.write(&data)` name the inherent slice methods above. The trait
/// methods are reached as `Read::read(&mut usb)` and
/// `Write::write(&mut usb, byte)`, or through a generic bound.
impl ErrorType for Rp2350Usb {
    type Error = UsbError;
}

impl Read<u8> for Rp2350Usb {
    /// One received byte, [`UsbError::WouldBlock`] if none is waiting, or
    /// [`UsbError::NotConfigured`].
    fn read(&mut self) -> Result<u8, UsbError> {
        let mut byte = [0u8; 1];
        if Rp2350Usb::read(self, &mut byte) == 1 {
            Ok(byte[0])
        } else if !self.is_configured() {
            Err(UsbError::NotConfigured)
        } else {
            Err(UsbError::WouldBlock)
        }
    }
}

impl Write<u8> for Rp2350Usb {
    /// Queue one byte; [`UsbError::WouldBlock`] if the transmit buffer is
    /// full, or [`UsbError::NotConfigured`].
    fn write(&mut self, byte: u8) -> Result<(), UsbError> {
        if !self.is_configured() {
            Err(UsbError::NotConfigured)
        } else if Rp2350Usb::write(self, &[byte]) == 1 {
            Ok(())
        } else {
            Err(UsbError::WouldBlock)
        }
    }
}
