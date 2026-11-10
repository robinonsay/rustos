//! Portable SPI controller ("master") trait.
//!
//! SPI moves data in both directions at once: on every clock edge pair the
//! controller shifts one bit out on its TX line (MOSI) and one bit in on
//! its RX line (MISO). Sending a byte therefore always receives a byte,
//! and reading a byte means sending one (usually `0xFF`, the idle level
//! many devices expect). Which device takes part is chosen by a separate
//! chip-select line, which is not part of this trait: drive it with a GPIO
//! output around each group of transfers, because devices such as SD cards
//! need it held low across several transfers.
//!
//! The four clock "modes" are the combinations of clock polarity and
//! phase; the RP2350 datasheet defines them in
//! [§12.3.4.9 "Motorola SPI frame format", p1051–1054](../../../docs/rp2350-datasheet.pdf#page=1051).
//! The RP2350 implementation is `pico2::spi::Rp2350Spi`.

use crate::common::ErrorType;

/// Clock polarity and phase: which level the clock idles at and which edge
/// samples the data.
///
/// Named after the two register bits that select them on Arm PrimeCell
/// controllers, `SPO` (polarity) and `SPH` (phase):
/// [§12.3.4.9.1–12.3.4.9.2, p1051](../../../docs/rp2350-datasheet.pdf#page=1051).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Clock idles low; data is captured on the first (rising) edge. SD
    /// cards in SPI mode use this.
    Mode0,
    /// Clock idles low; data is captured on the second (falling) edge.
    Mode1,
    /// Clock idles high; data is captured on the first (falling) edge.
    Mode2,
    /// Clock idles high; data is captured on the second (rising) edge.
    Mode3,
}

impl Mode {
    /// `true` if the clock idles high (`SPO` = 1).
    pub const fn idle_high(self) -> bool {
        // Modes 2 and 3 are the two with polarity 1.
        matches!(self, Mode::Mode2 | Mode::Mode3)
    }

    /// `true` if data is captured on the second clock edge (`SPH` = 1).
    pub const fn capture_on_second_edge(self) -> bool {
        // Modes 1 and 3 are the two with phase 1.
        matches!(self, Mode::Mode1 | Mode::Mode3)
    }
}

/// An SPI bus driven by this chip as controller, moving 8-bit words.
///
/// Every method returns once the last bit has been clocked, so the caller
/// may raise chip-select as soon as it returns.
pub trait SpiBus: ErrorType {
    /// Send `write` and store the bytes received at the same time in
    /// `read`. If the two differ in length, the shorter one is padded:
    /// extra bytes sent are `0xFF`, extra bytes received are discarded.
    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error>;

    /// Send each byte of `buffer` and replace it with the byte received.
    fn transfer_in_place(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error>;

    /// Send `bytes`, discarding what comes back.
    fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;

    /// Receive `buffer.len()` bytes, sending `0xFF` for each.
    fn read(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_bits_match_the_four_combinations() {
        // (polarity, phase) for modes 0..3, as in §12.3.4.10–12.3.4.13.
        assert_eq!((Mode::Mode0.idle_high(), Mode::Mode0.capture_on_second_edge()), (false, false));
        assert_eq!((Mode::Mode1.idle_high(), Mode::Mode1.capture_on_second_edge()), (false, true));
        assert_eq!((Mode::Mode2.idle_high(), Mode::Mode2.capture_on_second_edge()), (true, false));
        assert_eq!((Mode::Mode3.idle_high(), Mode::Mode3.capture_on_second_edge()), (true, true));
    }
}
