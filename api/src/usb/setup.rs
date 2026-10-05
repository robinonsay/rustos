//! The 8-byte SETUP packet that opens every control transfer, and the
//! standard codes found in it.
//!
//! USB 2.0 §9.3, Table 9-2 defines the layout. All multi-byte fields are
//! little-endian:
//!
//! | Offset | Field | Size | Meaning |
//! |--------|-------|------|---------|
//! | 0 | `bmRequestType` | 1 | bit 7 direction, bits 6:5 type, bits 4:0 recipient |
//! | 1 | `bRequest` | 1 | which request (meaning depends on the type) |
//! | 2 | `wValue` | 2 | request-specific argument |
//! | 4 | `wIndex` | 2 | request-specific; usually an interface or endpoint number |
//! | 6 | `wLength` | 2 | number of bytes in the data stage (0 = no data stage) |

/// One decoded SETUP packet. Plain data: build it with
/// [`SetupPacket::from_bytes`] from the 8 bytes the hardware received.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetupPacket {
    /// `bmRequestType`: direction, type and recipient packed into one byte.
    /// Use [`direction`](Self::direction), [`kind`](Self::kind) and
    /// [`recipient`](Self::recipient) rather than masking by hand.
    pub request_type: u8,
    /// `bRequest`: the request code. For standard requests see [`request`];
    /// for CDC class requests see [`crate::usb::cdc_acm::class_request`].
    pub request: u8,
    /// `wValue`.
    pub value: u16,
    /// `wIndex`.
    pub index: u16,
    /// `wLength`: the maximum number of bytes the data stage may carry. For
    /// a device-to-host request the device may send *fewer* (USB 2.0
    /// §9.3.5), never more.
    pub length: u16,
}

/// Bit 7 of `bmRequestType`: which way the data stage flows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// OUT: host to device. Also used by requests with no data stage.
    HostToDevice,
    /// IN: device to host.
    DeviceToHost,
}

/// Bits 6:5 of `bmRequestType`: who defines the request's meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestKind {
    /// Defined by USB 2.0 chapter 9; every device must handle these.
    Standard,
    /// Defined by the class specification, here CDC/PSTN.
    Class,
    /// Defined by the vendor. This device defines none.
    Vendor,
    /// Value 3, reserved by USB 2.0.
    Reserved,
}

/// Bits 4:0 of `bmRequestType`: what the request is addressed to. When
/// the recipient is an interface or an endpoint, `wIndex` says which one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recipient {
    /// The device as a whole.
    Device,
    /// An interface; `wIndex` low byte is its number.
    Interface,
    /// An endpoint; `wIndex` low byte is its address, direction in bit 7.
    Endpoint,
    /// "Other" (value 3), unused by this device.
    Other,
    /// Values 4–31, reserved.
    Reserved(u8),
}

impl SetupPacket {
    /// Decode the 8 bytes exactly as the hardware stored them.
    pub const fn from_bytes(b: [u8; 8]) -> Self {
        Self {
            request_type: b[0],
            request: b[1],
            value: u16::from_le_bytes([b[2], b[3]]),
            index: u16::from_le_bytes([b[4], b[5]]),
            length: u16::from_le_bytes([b[6], b[7]]),
        }
    }

    /// Direction of the data stage (bit 7 of `bmRequestType`).
    pub const fn direction(&self) -> Direction {
        if self.request_type & 0x80 != 0 {
            Direction::DeviceToHost
        } else {
            Direction::HostToDevice
        }
    }

    /// Request type (bits 6:5 of `bmRequestType`).
    pub const fn kind(&self) -> RequestKind {
        match (self.request_type >> 5) & 0x3 {
            0 => RequestKind::Standard,
            1 => RequestKind::Class,
            2 => RequestKind::Vendor,
            _ => RequestKind::Reserved,
        }
    }

    /// Recipient (bits 4:0 of `bmRequestType`).
    pub const fn recipient(&self) -> Recipient {
        match self.request_type & 0x1f {
            0 => Recipient::Device,
            1 => Recipient::Interface,
            2 => Recipient::Endpoint,
            3 => Recipient::Other,
            r => Recipient::Reserved(r),
        }
    }

    /// For GET_DESCRIPTOR: the descriptor type, the high byte of `wValue`
    /// (USB 2.0 §9.4.3).
    pub const fn descriptor_type(&self) -> u8 {
        (self.value >> 8) as u8
    }

    /// For GET_DESCRIPTOR: the descriptor index, the low byte of `wValue`.
    pub const fn descriptor_index(&self) -> u8 {
        (self.value & 0xff) as u8
    }
}

/// Standard request codes, `bRequest` when the type is
/// [`RequestKind::Standard`]. USB 2.0 §9.4, Table 9-4.
pub mod request {
    /// Two status bytes for the device, an interface or an endpoint (§9.4.5).
    pub const GET_STATUS: u8 = 0;
    /// Clear a feature; for this device only ENDPOINT_HALT (§9.4.1).
    pub const CLEAR_FEATURE: u8 = 1;
    /// Set a feature; for this device only ENDPOINT_HALT (§9.4.9).
    pub const SET_FEATURE: u8 = 3;
    /// Assign the device its bus address, `wValue` 0–127 (§9.4.6).
    pub const SET_ADDRESS: u8 = 5;
    /// Read a descriptor (§9.4.3).
    pub const GET_DESCRIPTOR: u8 = 6;
    /// Write a descriptor. Optional; this device stalls it (§9.4.8).
    pub const SET_DESCRIPTOR: u8 = 7;
    /// One byte: the current configuration value, 0 if unconfigured (§9.4.2).
    pub const GET_CONFIGURATION: u8 = 8;
    /// Select a configuration by value, or 0 to deconfigure (§9.4.7).
    pub const SET_CONFIGURATION: u8 = 9;
    /// One byte: an interface's current alternate setting (§9.4.4).
    pub const GET_INTERFACE: u8 = 10;
    /// Select an interface's alternate setting (§9.4.10).
    pub const SET_INTERFACE: u8 = 11;
    /// Isochronous only; this device stalls it (§9.4.11).
    pub const SYNCH_FRAME: u8 = 12;
}

/// Descriptor type codes: the high byte of `wValue` in GET_DESCRIPTOR and
/// the second byte of every descriptor. USB 2.0 §9.4, Table 9-5, plus the
/// IAD ECN and CDC 1.2 §5.2.3.
pub mod descriptor_type {
    /// Device descriptor (§9.6.1).
    pub const DEVICE: u8 = 1;
    /// Configuration descriptor, returned together with all the interface,
    /// endpoint and class descriptors that follow it (§9.6.3).
    pub const CONFIGURATION: u8 = 2;
    /// String descriptor (§9.6.7).
    pub const STRING: u8 = 3;
    /// Interface descriptor (§9.6.5).
    pub const INTERFACE: u8 = 4;
    /// Endpoint descriptor (§9.6.6).
    pub const ENDPOINT: u8 = 5;
    /// Device qualifier: only for devices that can also run at high speed
    /// (§9.6.2). A full-speed-only device answers with a STALL.
    pub const DEVICE_QUALIFIER: u8 = 6;
    /// Other-speed configuration: likewise high-speed-capable only (§9.6.4).
    pub const OTHER_SPEED_CONFIGURATION: u8 = 7;
    /// Interface association descriptor (IAD ECN).
    pub const INTERFACE_ASSOCIATION: u8 = 11;
    /// Class-specific interface descriptor (CDC 1.2 §5.2.3).
    pub const CS_INTERFACE: u8 = 0x24;
}

/// Standard feature selectors, `wValue` of SET_FEATURE / CLEAR_FEATURE.
/// USB 2.0 §9.4, Table 9-6.
pub mod feature {
    /// Recipient endpoint: halt (STALL) the endpoint.
    pub const ENDPOINT_HALT: u16 = 0;
    /// Recipient device: allow the device to wake a suspended host.
    pub const DEVICE_REMOTE_WAKEUP: u16 = 1;
    /// Recipient device: electrical test modes, high-speed only.
    pub const TEST_MODE: u16 = 2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_get_device_descriptor() {
        // What every host sends first: GET_DESCRIPTOR(DEVICE), wLength 64.
        let p = SetupPacket::from_bytes([0x80, 0x06, 0x00, 0x01, 0x00, 0x00, 0x40, 0x00]);
        assert_eq!(p.direction(), Direction::DeviceToHost);
        assert_eq!(p.kind(), RequestKind::Standard);
        assert_eq!(p.recipient(), Recipient::Device);
        assert_eq!(p.request, request::GET_DESCRIPTOR);
        assert_eq!(p.descriptor_type(), descriptor_type::DEVICE);
        assert_eq!(p.descriptor_index(), 0);
        assert_eq!(p.index, 0);
        assert_eq!(p.length, 64);
    }

    #[test]
    fn decodes_string_request_with_langid() {
        // GET_DESCRIPTOR(STRING, index 2, LANGID 0x0409), wLength 255.
        let p = SetupPacket::from_bytes([0x80, 0x06, 0x02, 0x03, 0x09, 0x04, 0xff, 0x00]);
        assert_eq!(p.descriptor_type(), descriptor_type::STRING);
        assert_eq!(p.descriptor_index(), 2);
        assert_eq!(p.index, 0x0409);
        assert_eq!(p.length, 255);
    }

    #[test]
    fn decodes_class_interface_out() {
        // SET_CONTROL_LINE_STATE(DTR|RTS) to interface 0.
        let p = SetupPacket::from_bytes([0x21, 0x22, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(p.direction(), Direction::HostToDevice);
        assert_eq!(p.kind(), RequestKind::Class);
        assert_eq!(p.recipient(), Recipient::Interface);
        assert_eq!(p.value, 3);
    }

    #[test]
    fn decodes_endpoint_recipient_and_reserved() {
        let p = SetupPacket::from_bytes([0x02, 0x01, 0x00, 0x00, 0x82, 0x00, 0x00, 0x00]);
        assert_eq!(p.recipient(), Recipient::Endpoint);
        assert_eq!(p.index, 0x82);
        let p = SetupPacket::from_bytes([0x65, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(p.kind(), RequestKind::Reserved);
        assert_eq!(p.recipient(), Recipient::Reserved(5));
        let p = SetupPacket::from_bytes([0x43, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(p.kind(), RequestKind::Vendor);
        assert_eq!(p.recipient(), Recipient::Other);
    }
}
