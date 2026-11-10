//! Portable I2C controller ("master") trait.
//!
//! I2C is a two-wire bus: one clock line (SCL) and one data line (SDA),
//! shared by every device on it. A transfer starts with the controller
//! sending a 7-bit device address and a read/write bit; the addressed
//! device acknowledges, then bytes move in the chosen direction until the
//! controller ends the transfer with a STOP. Reading a register of a
//! sensor is a *combined* transfer: write the register number, then,
//! without releasing the bus, send a repeated START (RESTART) and read the
//! value back ([`I2c::write_read`]).
//!
//! Nothing here knows what a register is; the RP2350 implementation is
//! `pico2::i2c::Rp2350I2c`. The bus terms used below are defined in the
//! RP2350 datasheet,
//! [§12.2.4 "I2C Terminology", p985–986](../../../docs/rp2350-datasheet.pdf#page=985),
//! and the transfer formats in
//! [§12.2.6 "I2C Protocols", p987–990](../../../docs/rp2350-datasheet.pdf#page=987).

use crate::common::ErrorType;

/// An I2C bus driven by this chip as the only controller.
///
/// Every method is one complete transfer: it starts with a START, ends
/// with a STOP, and returns only when the STOP has been sent or the
/// transfer has failed. `address` is the 7-bit device address, without the
/// read/write bit (`0x68` for an MPU-6050 with AD0 low, not `0xD0`).
///
/// Takes `&mut self` for the same reason as [`crate::common::Write`]: the
/// borrow checker then guarantees that two transfers can never be
/// interleaved on one bus.
pub trait I2c: ErrorType {
    /// Send `bytes` to the device at `address`.
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error>;

    /// Read `buffer.len()` bytes from the device at `address` into
    /// `buffer`.
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error>;

    /// Send `bytes`, then, with a repeated START and no STOP in between,
    /// read `buffer.len()` bytes back: the usual "read register *n*"
    /// transfer.
    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error>;
}
