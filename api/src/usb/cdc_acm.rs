//! CDC-ACM class requests and the state they carry.
//!
//! An ACM serial port still has the knobs of an RS-232 port — baud rate,
//! framing, the DTR and RTS lines — but on USB they are just values the
//! host sends over endpoint 0. Nothing here changes how bytes move on the
//! bulk endpoints: a USB serial port runs at USB speed whatever baud rate
//! the host "sets". The values are stored so the host can read them back,
//! and so an application can use DTR as a "terminal is open" hint if it
//! wishes.
//!
//! # DTR is reported, not required
//!
//! Many terminal programs assert DTR when they open a port, but not all do
//! (some scripting libraries open with DTR and RTS deliberately low, e.g. to
//! avoid resetting Arduino-style boards). The driver therefore never gates
//! data on DTR: bytes flow as soon as the host has configured the device.

/// CDC class request codes used by ACM, `bRequest` when the request type is
/// Class and the recipient the communication interface. PSTN 1.2 §6.3 lists
/// the full set.
pub mod class_request {
    /// Send an encapsulated (AT) command. Not supported: protocol 0.
    pub const SEND_ENCAPSULATED_COMMAND: u8 = 0x00;
    /// Read an encapsulated response. Not supported: protocol 0.
    pub const GET_ENCAPSULATED_RESPONSE: u8 = 0x01;
    /// Host to device, 7 bytes of [`LineCoding`](super::LineCoding)
    /// (PSTN 1.2 §6.3.10).
    pub const SET_LINE_CODING: u8 = 0x20;
    /// Device to host, 7 bytes of [`LineCoding`](super::LineCoding)
    /// (PSTN 1.2 §6.3.11).
    pub const GET_LINE_CODING: u8 = 0x21;
    /// No data stage; DTR and RTS in `wValue` bits 0 and 1
    /// (PSTN 1.2 §6.3.12).
    pub const SET_CONTROL_LINE_STATE: u8 = 0x22;
    /// No data stage; break duration in ms in `wValue` (PSTN 1.2 §6.3.13).
    pub const SEND_BREAK: u8 = 0x23;
}

/// Number of stop bits, `bCharFormat` of the line coding structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum StopBits {
    /// 1 stop bit.
    One = 0,
    /// 1.5 stop bits.
    OnePointFive = 1,
    /// 2 stop bits.
    Two = 2,
}

/// Parity, `bParityType` of the line coding structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Parity {
    /// No parity bit.
    None = 0,
    /// Odd parity.
    Odd = 1,
    /// Even parity.
    Even = 2,
    /// Parity bit always 1.
    Mark = 3,
    /// Parity bit always 0.
    Space = 4,
}

/// The 7-byte line coding structure of SET_LINE_CODING / GET_LINE_CODING
/// (PSTN 1.2 §6.3.11):
///
/// | Offset | Field | Size | Meaning |
/// |--------|-------|------|---------|
/// | 0 | `dwDTERate` | 4 | baud rate, little-endian |
/// | 4 | `bCharFormat` | 1 | [`StopBits`] |
/// | 5 | `bParityType` | 1 | [`Parity`] |
/// | 6 | `bDataBits` | 1 | 5, 6, 7, 8 or 16 |
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineCoding {
    /// Baud rate the host asked for. Has no effect on USB throughput.
    pub baud: u32,
    /// Stop bits.
    pub stop_bits: StopBits,
    /// Parity.
    pub parity: Parity,
    /// Data bits: 5, 6, 7, 8 or 16.
    pub data_bits: u8,
}

/// Length of the line coding structure on the wire.
pub const LINE_CODING_LEN: usize = 7;

impl LineCoding {
    /// 115200 baud, 8 data bits, no parity, 1 stop bit — what GET_LINE_CODING
    /// reports until the host sets something else.
    pub const DEFAULT: LineCoding = LineCoding {
        baud: 115_200,
        stop_bits: StopBits::One,
        parity: Parity::None,
        data_bits: 8,
    };

    /// Decode the 7 bytes of a SET_LINE_CODING data stage. Returns `None`
    /// if the slice is not exactly 7 bytes or any enumerated field holds a
    /// value PSTN 1.2 does not define; the caller answers that with a STALL.
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() != LINE_CODING_LEN {
            return None;
        }
        let stop_bits = match b[4] {
            0 => StopBits::One,
            1 => StopBits::OnePointFive,
            2 => StopBits::Two,
            _ => return None,
        };
        let parity = match b[5] {
            0 => Parity::None,
            1 => Parity::Odd,
            2 => Parity::Even,
            3 => Parity::Mark,
            4 => Parity::Space,
            _ => return None,
        };
        let data_bits = match b[6] {
            5 | 6 | 7 | 8 | 16 => b[6],
            _ => return None,
        };
        Some(Self {
            baud: u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            stop_bits,
            parity,
            data_bits,
        })
    }

    /// Encode for GET_LINE_CODING.
    pub const fn to_bytes(&self) -> [u8; LINE_CODING_LEN] {
        let r = self.baud.to_le_bytes();
        [r[0], r[1], r[2], r[3], self.stop_bits as u8, self.parity as u8, self.data_bits]
    }
}

impl Default for LineCoding {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The two modem-control outputs a host drives with SET_CONTROL_LINE_STATE
/// (PSTN 1.2 §6.3.12): `wValue` bit 0 is DTR, bit 1 is RTS (named
/// "carrier control" in the spec). Both start low.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlLineState {
    /// Data Terminal Ready: conventionally "a terminal has the port open".
    pub dtr: bool,
    /// Request To Send.
    pub rts: bool,
}

impl ControlLineState {
    /// Decode `wValue` of SET_CONTROL_LINE_STATE. Bits above 1 are reserved
    /// and ignored.
    pub const fn from_value(value: u16) -> Self {
        Self {
            dtr: value & 0x1 != 0,
            rts: value & 0x2 != 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_coding_round_trip() {
        // 9600 7E2 as a host would send it.
        let wire = [0x80, 0x25, 0x00, 0x00, 2, 2, 7];
        let lc = LineCoding::from_bytes(&wire).unwrap();
        assert_eq!(lc.baud, 9600);
        assert_eq!(lc.stop_bits, StopBits::Two);
        assert_eq!(lc.parity, Parity::Even);
        assert_eq!(lc.data_bits, 7);
        assert_eq!(lc.to_bytes(), wire);
    }

    #[test]
    fn default_line_coding_bytes() {
        assert_eq!(LineCoding::DEFAULT.to_bytes(), [0x00, 0xC2, 0x01, 0x00, 0, 0, 8]);
        assert_eq!(LineCoding::default(), LineCoding::DEFAULT);
    }

    #[test]
    fn line_coding_rejects_bad_values() {
        assert_eq!(LineCoding::from_bytes(&[0, 0, 0, 0, 0, 0]), None);
        assert_eq!(LineCoding::from_bytes(&[0, 0, 0, 0, 0, 0, 8, 0]), None);
        assert_eq!(LineCoding::from_bytes(&[0, 0, 0, 0, 3, 0, 8]), None);
        assert_eq!(LineCoding::from_bytes(&[0, 0, 0, 0, 0, 5, 8]), None);
        assert_eq!(LineCoding::from_bytes(&[0, 0, 0, 0, 0, 0, 9]), None);
        assert!(LineCoding::from_bytes(&[0, 0, 0, 0, 1, 4, 16]).is_some());
    }

    #[test]
    fn control_line_state_bits() {
        assert_eq!(ControlLineState::from_value(0), ControlLineState { dtr: false, rts: false });
        assert_eq!(ControlLineState::from_value(1), ControlLineState { dtr: true, rts: false });
        assert_eq!(ControlLineState::from_value(2), ControlLineState { dtr: false, rts: true });
        assert_eq!(ControlLineState::from_value(0xff03), ControlLineState { dtr: true, rts: true });
    }
}
