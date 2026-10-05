//! Descriptors: the self-description a USB device hands the host during
//! enumeration.
//!
//! Three kinds matter here:
//!
//! * the **device descriptor** — who made the device (VID/PID) and which
//!   class driver should look at it. Built at run time from a
//!   [`UsbDeviceConfig`] by [`device_descriptor`].
//! * the **configuration descriptor** — the interfaces and endpoints. The
//!   CDC-ACM layout is fixed, so it is the constant
//!   [`CONFIGURATION_DESCRIPTOR`].
//! * **string descriptors** — human-readable names, encoded as UTF-16LE by
//!   [`string_descriptor`]; string 0 is the list of supported languages,
//!   [`LANGUAGE_ID_DESCRIPTOR`].
//!
//! # Why an Interface Association Descriptor
//!
//! A CDC-ACM function is *two* interfaces (communication + data). A host
//! binding class drivers per interface needs to be told that the two belong
//! together, or Windows in particular will try to drive them separately.
//! The IAD ECN solves this: the device descriptor declares class `0xEF`
//! (Miscellaneous), subclass `0x02`, protocol `0x01` ("this device uses
//! IADs"), and an IAD placed before the first interface groups interfaces 0
//! and 1 into one function of class CDC/ACM. With that, Windows 10 and later
//! load the in-box `usbser.sys` through the composite driver with no INF,
//! and Linux and macOS, which handle CDC-ACM without the IAD anyway, are
//! unaffected.

use crate::usb::setup::descriptor_type;

/// Endpoint 0's maximum packet size. 64 is the largest allowed at full
/// speed and what every modern host expects (USB 2.0 §5.5.3).
pub const EP0_MAX_PACKET: usize = 64;

/// Maximum packet size of the two bulk data endpoints: 64, the full-speed
/// maximum for bulk (USB 2.0 §5.8.3).
pub const BULK_MAX_PACKET: usize = 64;

/// Maximum packet size of the notification endpoint. A CDC SERIAL_STATE
/// notification is 10 bytes (PSTN 1.2 §6.5.4), so 16 holds one whole.
/// This device never sends a notification; the endpoint exists because ACM
/// requires it and simply NAKs every poll.
pub const NOTIFY_MAX_PACKET: usize = 16;

/// Endpoint 1 IN, interrupt: CDC notifications (never armed, see above).
pub const EP_NOTIFY_IN: u8 = 0x81;
/// Endpoint 2 OUT, bulk: bytes from the host.
pub const EP_DATA_OUT: u8 = 0x02;
/// Endpoint 2 IN, bulk: bytes to the host.
pub const EP_DATA_IN: u8 = 0x82;

/// Interface number of the CDC communication interface.
pub const INTERFACE_COMM: u8 = 0;
/// Interface number of the CDC data interface.
pub const INTERFACE_DATA: u8 = 1;
/// Number of interfaces in the configuration.
pub const NUM_INTERFACES: u8 = 2;

/// `bConfigurationValue` of the one configuration. SET_CONFIGURATION with
/// this value configures the device; with 0 it deconfigures it.
pub const CONFIGURATION_VALUE: u8 = 1;

/// String descriptor index of the manufacturer string.
pub const STRING_MANUFACTURER: u8 = 1;
/// String descriptor index of the product string.
pub const STRING_PRODUCT: u8 = 2;
/// String descriptor index of the serial-number string.
pub const STRING_SERIAL: u8 = 3;

/// The longest string, in UTF-16 code units, a string descriptor can hold:
/// `bLength` is one byte, so 2 header bytes + 2 × 126 = 254 bytes.
pub const MAX_STRING_UNITS: usize = 126;

/// The parts of the descriptors an application chooses.
///
/// # Choosing a VID/PID
///
/// The pair identifies the product to every host it is ever plugged into, so
/// it should be one you are entitled to use: your own USB-IF vendor ID, a
/// PID allocated from a vendor's community scheme (Raspberry Pi allocates
/// PIDs under its VID `0x2E8A` to RP2040/RP2350 projects on request), or a
/// test pair used only on your own bench. Hosts cache driver choices per
/// VID/PID, which is why changing one makes a device "new" to the OS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UsbDeviceConfig {
    /// `idVendor`.
    pub vendor_id: u16,
    /// `idProduct`.
    pub product_id: u16,
    /// `bcdDevice`, the device release number in binary-coded decimal
    /// (`0x0100` = 1.00).
    pub device_release: u16,
    /// String 1. Truncated to [`MAX_STRING_UNITS`] UTF-16 code units.
    pub manufacturer: &'static str,
    /// String 2, the name hosts show the user. Same limit.
    pub product: &'static str,
    /// String 3. Hosts use it to tell two identical devices apart and to
    /// give the same one a stable name (e.g. `/dev/serial/by-id/...` on
    /// Linux, a stable COM number on Windows). Same limit.
    pub serial_number: &'static str,
}

/// Length of the device descriptor (USB 2.0 Table 9-8).
pub const DEVICE_DESCRIPTOR_LEN: usize = 18;

/// Build the 18-byte device descriptor (USB 2.0 §9.6.1, Table 9-8).
///
/// | Offset | Field | Value | Why |
/// |--------|-------|-------|-----|
/// | 0 | `bLength` | 18 | |
/// | 1 | `bDescriptorType` | 1 (DEVICE) | |
/// | 2 | `bcdUSB` | `0x0200` | USB 2.0. Not 2.01: that would make Windows ask for a BOS descriptor. |
/// | 4 | `bDeviceClass` | `0xEF` | Miscellaneous: "see the IAD" (IAD ECN) |
/// | 5 | `bDeviceSubClass` | `0x02` | Common Class |
/// | 6 | `bDeviceProtocol` | `0x01` | Interface Association Descriptor |
/// | 7 | `bMaxPacketSize0` | 64 | [`EP0_MAX_PACKET`] |
/// | 8 | `idVendor` | config | |
/// | 10 | `idProduct` | config | |
/// | 12 | `bcdDevice` | config | |
/// | 14 | `iManufacturer` | 1 | [`STRING_MANUFACTURER`] |
/// | 15 | `iProduct` | 2 | [`STRING_PRODUCT`] |
/// | 16 | `iSerialNumber` | 3 | [`STRING_SERIAL`] |
/// | 17 | `bNumConfigurations` | 1 | |
pub const fn device_descriptor(cfg: &UsbDeviceConfig) -> [u8; DEVICE_DESCRIPTOR_LEN] {
    let vid = cfg.vendor_id.to_le_bytes();
    let pid = cfg.product_id.to_le_bytes();
    let rel = cfg.device_release.to_le_bytes();
    [
        DEVICE_DESCRIPTOR_LEN as u8,
        descriptor_type::DEVICE,
        0x00, 0x02,            // bcdUSB 2.00
        0xEF, 0x02, 0x01,      // class/subclass/protocol: IAD
        EP0_MAX_PACKET as u8,
        vid[0], vid[1],
        pid[0], pid[1],
        rel[0], rel[1],
        STRING_MANUFACTURER,
        STRING_PRODUCT,
        STRING_SERIAL,
        1,                     // bNumConfigurations
    ]
}

/// Total length of [`CONFIGURATION_DESCRIPTOR`].
pub const CONFIGURATION_DESCRIPTOR_LEN: usize = 75;

/// The complete CDC-ACM configuration descriptor, 75 bytes.
///
/// GET_DESCRIPTOR(CONFIGURATION) returns the configuration descriptor
/// followed by every descriptor that belongs to it, in one blob whose total
/// length is in bytes 2–3 (USB 2.0 §9.4.3, §9.6.3). Hosts typically ask for
/// 9 bytes first to learn that total, then for the whole thing.
///
/// ```text
/// off len  descriptor
///   0   9  Configuration      wTotalLength 75, 2 interfaces, value 1,
///                             bmAttributes 0x80 (bus powered, no remote
///                             wakeup), bMaxPower 50 (x 2 mA = 100 mA)
///   9   8  Interface Assoc.   interfaces 0..=1, class 02/02/00 (CDC ACM)
///  17   9  Interface 0        1 endpoint, class 02 (CDC comm), subclass 02
///                             (ACM), protocol 00 (no AT commands)
///  26   5   CDC Header        bcdCDC 1.10              (CDC 1.2 §5.2.3.1)
///  31   5   CDC Call Mgmt     caps 0, data interface 1 (PSTN 1.2 §5.3.1)
///  36   4   CDC ACM           caps 0x02                (PSTN 1.2 §5.3.2)
///  40   5   CDC Union         master 0, slave 1        (CDC 1.2 §5.2.3.2)
///  45   7   Endpoint 0x81     interrupt, 16 bytes, every 16 ms
///  52   9  Interface 1        2 endpoints, class 0A (CDC data)
///  61   7   Endpoint 0x02     bulk OUT, 64 bytes
///  68   7   Endpoint 0x82     bulk IN, 64 bytes
/// ```
///
/// ACM capability `0x02` (bit D1) advertises SET_LINE_CODING,
/// GET_LINE_CODING, SET_CONTROL_LINE_STATE and the SERIAL_STATE
/// notification; bit D2 (SEND_BREAK) is left clear, although SEND_BREAK is
/// still accepted. Call-management capabilities 0 mean the device does no
/// call management of its own, which is the norm for a virtual serial
/// port.
pub const CONFIGURATION_DESCRIPTOR: [u8; CONFIGURATION_DESCRIPTOR_LEN] = [
    // Configuration (USB 2.0 Table 9-10)
    9, descriptor_type::CONFIGURATION,
    CONFIGURATION_DESCRIPTOR_LEN as u8, 0,
    NUM_INTERFACES, CONFIGURATION_VALUE, 0, 0x80, 50,
    // Interface Association (IAD ECN)
    8, descriptor_type::INTERFACE_ASSOCIATION,
    INTERFACE_COMM, 2, 0x02, 0x02, 0x00, 0,
    // Interface 0: CDC communication (USB 2.0 Table 9-12; CDC 1.2 §4.2-§4.4)
    9, descriptor_type::INTERFACE,
    INTERFACE_COMM, 0, 1, 0x02, 0x02, 0x00, 0,
    // CDC Header functional descriptor: bcdCDC = 1.10
    5, descriptor_type::CS_INTERFACE, 0x00, 0x10, 0x01,
    // CDC Call Management functional descriptor
    5, descriptor_type::CS_INTERFACE, 0x01, 0x00, INTERFACE_DATA,
    // CDC Abstract Control Management functional descriptor
    4, descriptor_type::CS_INTERFACE, 0x02, 0x02,
    // CDC Union functional descriptor
    5, descriptor_type::CS_INTERFACE, 0x06, INTERFACE_COMM, INTERFACE_DATA,
    // Endpoint 0x81: interrupt IN, notifications (USB 2.0 Table 9-13)
    7, descriptor_type::ENDPOINT,
    EP_NOTIFY_IN, 0x03, NOTIFY_MAX_PACKET as u8, 0, 16,
    // Interface 1: CDC data (CDC 1.2 §4.5)
    9, descriptor_type::INTERFACE,
    INTERFACE_DATA, 0, 2, 0x0A, 0x00, 0x00, 0,
    // Endpoint 0x02: bulk OUT
    7, descriptor_type::ENDPOINT,
    EP_DATA_OUT, 0x02, BULK_MAX_PACKET as u8, 0, 0,
    // Endpoint 0x82: bulk IN
    7, descriptor_type::ENDPOINT,
    EP_DATA_IN, 0x02, BULK_MAX_PACKET as u8, 0, 0,
];

/// String descriptor 0: the list of supported language IDs (USB 2.0
/// §9.6.7, Table 9-15). One entry, `0x0409`, US English, which is what
/// every host asks for.
pub const LANGUAGE_ID_DESCRIPTOR: [u8; 4] = [4, descriptor_type::STRING, 0x09, 0x04];

/// Length in bytes of the string descriptor [`string_descriptor`] would
/// build for `s` into an unlimited buffer.
pub fn string_descriptor_len(s: &str) -> usize {
    2 + 2 * s.encode_utf16().count().min(MAX_STRING_UNITS)
}

/// Encode `s` as a string descriptor (USB 2.0 §9.6.7, Table 9-16) into
/// `out`, returning the number of bytes written.
///
/// The layout is `bLength`, `bDescriptorType` = 3, then the string as
/// UTF-16 little-endian code units with no terminator. Characters outside
/// the Basic Multilingual Plane become surrogate pairs, as UTF-16 requires.
/// The string is truncated to [`MAX_STRING_UNITS`] code units, and further
/// to whatever fits in `out`; a truncation never splits a code unit, though
/// it can split a surrogate pair. `out` shorter than 2 bytes produces 0.
pub fn string_descriptor(s: &str, out: &mut [u8]) -> usize {
    if out.len() < 2 {
        return 0;
    }
    let mut n = 2;
    for (i, u) in s.encode_utf16().enumerate() {
        if i == MAX_STRING_UNITS || n + 2 > out.len() {
            break;
        }
        let b = u.to_le_bytes();
        out[n] = b[0];
        out[n + 1] = b[1];
        n += 2;
    }
    out[0] = n as u8;
    out[1] = descriptor_type::STRING;
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    const CFG: UsbDeviceConfig = UsbDeviceConfig {
        vendor_id: 0x2E8A,
        product_id: 0x1234,
        device_release: 0x0102,
        manufacturer: "Maker",
        product: "Keyer",
        serial_number: "0001",
    };

    #[test]
    fn device_descriptor_bytes() {
        assert_eq!(
            device_descriptor(&CFG),
            [
                18, 1, 0x00, 0x02, 0xEF, 0x02, 0x01, 64,
                0x8A, 0x2E, 0x34, 0x12, 0x02, 0x01,
                1, 2, 3, 1,
            ]
        );
    }

    #[test]
    fn configuration_descriptor_bytes() {
        #[rustfmt::skip]
        let expected: [u8; 75] = [
            0x09, 0x02, 0x4B, 0x00, 0x02, 0x01, 0x00, 0x80, 0x32,
            0x08, 0x0B, 0x00, 0x02, 0x02, 0x02, 0x00, 0x00,
            0x09, 0x04, 0x00, 0x00, 0x01, 0x02, 0x02, 0x00, 0x00,
            0x05, 0x24, 0x00, 0x10, 0x01,
            0x05, 0x24, 0x01, 0x00, 0x01,
            0x04, 0x24, 0x02, 0x02,
            0x05, 0x24, 0x06, 0x00, 0x01,
            0x07, 0x05, 0x81, 0x03, 0x10, 0x00, 0x10,
            0x09, 0x04, 0x01, 0x00, 0x02, 0x0A, 0x00, 0x00, 0x00,
            0x07, 0x05, 0x02, 0x02, 0x40, 0x00, 0x00,
            0x07, 0x05, 0x82, 0x02, 0x40, 0x00, 0x00,
        ];
        assert_eq!(CONFIGURATION_DESCRIPTOR, expected);
    }

    #[test]
    fn configuration_descriptor_is_self_consistent() {
        // Walk the chain of bLength fields: they must tile the blob exactly,
        // and wTotalLength must equal the blob length.
        let d = &CONFIGURATION_DESCRIPTOR;
        assert_eq!(u16::from_le_bytes([d[2], d[3]]) as usize, d.len());
        let mut off = 0;
        let mut count = 0;
        while off < d.len() {
            let len = d[off] as usize;
            assert!(len >= 2);
            off += len;
            count += 1;
        }
        assert_eq!(off, d.len());
        assert_eq!(count, 11);
    }

    #[test]
    fn language_id_bytes() {
        assert_eq!(LANGUAGE_ID_DESCRIPTOR, [0x04, 0x03, 0x09, 0x04]);
    }

    #[test]
    fn ascii_string_descriptor() {
        let mut buf = [0u8; 64];
        let n = string_descriptor("Pico", &mut buf);
        assert_eq!(n, 10);
        assert_eq!(&buf[..n], &[10, 3, b'P', 0, b'i', 0, b'c', 0, b'o', 0]);
        assert_eq!(string_descriptor_len("Pico"), 10);
    }

    #[test]
    fn non_ascii_string_descriptor() {
        let mut buf = [0u8; 16];
        // U+00E9 is one unit; U+1F4FB (radio emoji) is a surrogate pair.
        let n = string_descriptor("\u{e9}\u{1F4FB}", &mut buf);
        assert_eq!(&buf[..n], &[8, 3, 0xE9, 0x00, 0x3D, 0xD8, 0xFB, 0xDC]);
    }

    #[test]
    fn empty_string_descriptor() {
        let mut buf = [0u8; 4];
        assert_eq!(string_descriptor("", &mut buf), 2);
        assert_eq!(&buf[..2], &[2, 3]);
    }

    #[test]
    fn string_descriptor_truncates() {
        // 200 ASCII characters: capped at 126 units = 254 bytes.
        let mut buf = [0u8; 300];
        let n = string_descriptor(LONG, &mut buf);
        assert_eq!(n, 254);
        assert_eq!(buf[0], 254);
        assert_eq!(string_descriptor_len(LONG), 254);
        // A buffer too small for the whole string: whole units only.
        let mut small = [0u8; 7];
        assert_eq!(string_descriptor("abcdef", &mut small), 6);
        assert_eq!(small[0], 6);
        // Too small for even the header.
        let mut tiny = [0u8; 1];
        assert_eq!(string_descriptor("a", &mut tiny), 0);
    }

    /// 200 ASCII characters, built without `alloc`.
    const LONG: &str = concat!(
        "abcdefghijabcdefghijabcdefghijabcdefghijabcdefghij",
        "abcdefghijabcdefghijabcdefghijabcdefghijabcdefghij",
        "abcdefghijabcdefghijabcdefghijabcdefghijabcdefghij",
        "abcdefghijabcdefghijabcdefghijabcdefghijabcdefghij",
    );
}
