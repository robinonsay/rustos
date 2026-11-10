//! Portable analogue-to-digital converter (ADC) traits.
//!
//! An ADC measures a voltage and reports it as a whole number, a *code*,
//! from 0 (0 V) up to `2^bits - 1` (the converter's full-scale voltage).
//! The number of bits is its *resolution*: 12 bits is 4096 steps, about
//! 0.8 mV each on a 3.3 V scale.
//!
//! [`AdcStream`] is the continuous case: one input sampled at a fixed
//! rate, with samples buffered by the implementation until the
//! application collects them. That is what audio, vibration or any
//! signal that must be sampled evenly in time needs; on a microcontroller
//! it is usually a DMA channel filling a ring buffer, on a Linux board a
//! driver's buffered channel.
//!
//! How a stream is *started* is left to each implementation, because the
//! resources it needs are chip-specific (the RP2350 needs a DMA channel
//! and an aligned ring buffer, for example). Code that consumes samples
//! is written against [`AdcStream`] and does not care.
//!
//! ```ignore
//! use api::adc::AdcStream;
//!
//! fn level<S: AdcStream>(stream: &mut S) -> Result<Option<u32>, S::Error> {
//!     let mut block = [api::adc::Sample::INVALID; 64];
//!     let n = stream.read(&mut block)?;
//!     // Mean of the valid samples, in microvolts.
//!     let (sum, count) = block[..n].iter().filter_map(|s| s.code())
//!         .fold((0u64, 0u64), |(s, c), code| (s + code as u64, c + 1));
//!     Ok((count > 0).then(|| stream.microvolts((sum / count) as u16)))
//! }
//! ```

use crate::common::ErrorType;

/// One conversion result.
///
/// A converter can report that a particular conversion failed (the
/// RP2350's flags conversions that did not settle, for instance). Such a
/// sample keeps its place in time, so that a stream's sample count stays
/// a clock, but has no value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    /// The code, or `None` for a failed conversion.
    code: Option<u16>,
}

impl Sample {
    /// A failed conversion, also handy to fill a buffer before reading.
    pub const INVALID: Sample = Sample { code: None };

    /// A valid conversion result.
    pub const fn new(code: u16) -> Self {
        // Wrap the code as present.
        Self { code: Some(code) }
    }

    /// The code, or `None` if the conversion failed.
    pub const fn code(&self) -> Option<u16> {
        // Copy out of the field.
        self.code
    }

    /// `true` if the conversion succeeded.
    pub const fn is_valid(&self) -> bool {
        // Valid exactly when a code is present.
        self.code.is_some()
    }
}

/// Convert `code` from a converter of `bits` resolution and
/// `full_scale_microvolts` full scale into microvolts, rounding to the
/// nearest. Code `2^bits - 1` is full scale. `bits` must be 1–16.
///
/// Pure and chip-independent, so [`AdcStream::microvolts`] can share it.
pub const fn code_to_microvolts(code: u16, bits: u8, full_scale_microvolts: u32) -> u32 {
    // Largest code: 2^bits - 1, in u64 so the product cannot overflow.
    let max_code = (1u64 << bits) - 1;
    // Codes above the largest one are clamped to full scale.
    let code = if (code as u64) > max_code { max_code } else { code as u64 };
    // Scale, adding half the divisor to round to the nearest microvolt.
    ((code * full_scale_microvolts as u64 + max_code / 2) / max_code) as u32
}

/// One input sampled continuously at a fixed rate.
///
/// Samples arrive whether or not they are read; the implementation buffers
/// some, and reports an error if the application falls so far behind that
/// samples were lost.
pub trait AdcStream: ErrorType {
    /// The average sample rate actually produced, in samples per second,
    /// which may differ from the one requested by the rounding the
    /// hardware needs.
    fn sample_rate_hz(&self) -> u32;

    /// Bits per code: codes run from 0 to `2^bits - 1`.
    fn resolution_bits(&self) -> u8;

    /// The voltage of the largest code, in microvolts, as far as the
    /// implementation knows it (usually the converter's reference supply).
    fn full_scale_microvolts(&self) -> u32;

    /// Copy the samples that have arrived since the last call, oldest
    /// first, into the front of `out`, as many as fit, and return how
    /// many. Samples that do not fit stay buffered for the next call.
    /// Never waits: `Ok(0)` means nothing new.
    fn read(&mut self, out: &mut [Sample]) -> Result<usize, Self::Error>;

    /// `code` from this stream in microvolts; see [`code_to_microvolts`].
    fn microvolts(&self, code: u16) -> u32 {
        // The shared conversion with this stream's scale.
        code_to_microvolts(code, self.resolution_bits(), self.full_scale_microvolts())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_validity() {
        assert_eq!(Sample::new(123).code(), Some(123));
        assert!(Sample::new(0).is_valid());
        assert!(!Sample::INVALID.is_valid());
        assert_eq!(Sample::INVALID.code(), None);
    }

    #[test]
    fn ends_of_the_scale_are_exact() {
        // 12-bit converter on a 3.3 V reference.
        assert_eq!(code_to_microvolts(0, 12, 3_300_000), 0);
        assert_eq!(code_to_microvolts(4095, 12, 3_300_000), 3_300_000);
    }

    #[test]
    fn midscale_rounds_to_nearest() {
        // 2048 / 4095 × 3.3 V = 1 650 402.93 µV.
        assert_eq!(code_to_microvolts(2048, 12, 3_300_000), 1_650_403);
    }

    #[test]
    fn codes_above_full_scale_clamp() {
        assert_eq!(code_to_microvolts(0xffff, 12, 3_300_000), 3_300_000);
    }

    #[test]
    fn sixteen_bit_full_scale() {
        assert_eq!(code_to_microvolts(u16::MAX, 16, 5_000_000), 5_000_000);
    }
}
