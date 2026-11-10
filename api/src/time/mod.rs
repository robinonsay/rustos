//! # `time` — times tagged with the clock that produced them
//!
//! Different clocks on a system disagree: a GPS receiver, a free-running
//! hardware counter and mission elapsed time all count from different
//! epochs and may run at slightly different rates. A bare number of
//! seconds does not say which clock it came from, so mixing them up is
//! easy and silent.
//!
//! This module makes the clock part of the type. A [`Time<C>`] carries a
//! **type parameter** `C` — a type supplied in angle brackets, like the
//! element type of an array — naming the clock it belongs to. `Time<Gps>`
//! and `Time<Met>` are distinct types, so adding one to the other is a
//! compile error rather than a wrong answer. Converting between clocks is
//! then something done on purpose, by code written for that pair.
//!
//! ## Representation
//!
//! A `Time` is a 64-bit unsigned **fixed-point** number: a value with a
//! fixed number of fractional bits, held in an integer. The upper 32 bits
//! are whole seconds and the lower 32 bits are the fraction of a second in
//! units of 2^-32 s (about 233 ps). This is the layout of the CCSDS
//! Unsegmented Time Code with 4 coarse and 4 fine octets. Seconds wrap after
//! 2^32 s, about 136 years.

use core::{marker::PhantomData, ops::{Add, Mul}};

use crate::common::num::Number;

/// Identifies a clock, for use as the `C` in [`Time<C>`].
///
/// An implementing type is a **marker type**: a type with no fields whose
/// only job is to be named. It never exists at runtime; it only labels a
/// `Time` so the compiler can tell clocks apart. The supertraits (`Copy`,
/// `Eq`, `Ord`, `Debug`) let `#[derive]` on types that contain a `C` work
/// without extra bounds.
pub trait ClockId: Copy + Eq + Ord + core::fmt::Debug {
    /// Human-readable clock name, for logs and telemetry.
    const NAME: &'static str;
}

/// A time read from clock `C`, as 32.32 fixed-point seconds.
///
/// See the [module documentation](self) for the layout. Comparisons order
/// by seconds, then by fraction, which is time order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time<C: ClockId>
{
    /// Whole seconds since clock `C`'s epoch.
    seconds: u32,
    /// Fraction of a second, in units of 2^-32 s.
    subseconds: u32,
    /// Ties this value to clock `C`. `PhantomData<C>` is zero-sized: it
    /// tells the compiler the struct is "about" a `C` without storing one.
    _scale: PhantomData<C>
}


impl<C: ClockId> Time<C>
{
    /// Number of fractional bits in the fixed-point representation.
    const FRAC_BITS: u32 = 32;

    /// Packs the time into one `u64` of 32.32 fixed-point seconds, so
    /// arithmetic can carry between the fraction and the whole seconds.
    const fn to_fixed(&self) -> u64 {
        ((self.seconds as u64) << Self::FRAC_BITS) | self.subseconds as u64
    }

    /// Unpacks a 32.32 fixed-point value produced by [`Self::to_fixed`].
    const fn from_fixed(fixed: u64) -> Self {
        Self {
            seconds: (fixed >> Self::FRAC_BITS) as u32,
            subseconds: fixed as u32,
            _scale: PhantomData,
        }
    }

    /// One second in fixed-point units: 2^32. Dividing by it is exact in
    /// floating point because it is a power of two.
    const ONE_SECOND: u64 = 1 << Self::FRAC_BITS;

    /// Creates a time of `seconds` whole seconds plus `subseconds` units of
    /// 2^-32 s since clock `C`'s epoch.
    pub const fn new(seconds: u32, subseconds: u32) -> Self {
        Self { seconds, subseconds, _scale: PhantomData }
    }

    /// Converts a count of `ticks` from a counter running at
    /// `ticks_per_second` into a time: the usual way for a hardware
    /// [`Clock`] to build its reading.
    ///
    /// The fraction is truncated to whole 2^-32 s units. Seconds wrap
    /// after 2^32, like [`Self::add`]. Panics if `ticks_per_second` is 0.
    pub const fn from_ticks(ticks: u64, ticks_per_second: u32) -> Self {
        let hz = ticks_per_second as u64;
        // The remainder is below `hz` < 2^32, so shifting it left by 32
        // bits cannot overflow a u64.
        let fraction = ((ticks % hz) << Self::FRAC_BITS) / hz;
        Self::new((ticks / hz) as u32, fraction as u32)
    }

    /// Returns the time in seconds as an `f32`.
    ///
    /// The result is the nearest `f32` to the exact value. An `f32` holds
    /// 24 significant bits, so precision falls as the time grows: about
    /// 60 ns at 1 s, 1 ms at 10^4 s, and 256 s near the 2^32 s limit.
    /// Use [`Self::to_f64`] when that matters.
    pub fn to_f32(&self) -> f32 {
        self.to_fixed() as f32 / Self::ONE_SECOND as f32
    }

    /// Returns the time in seconds as an `f64`.
    ///
    /// The result is the nearest `f64` to the exact value. An `f64` holds
    /// 53 significant bits, so the full fraction survives below 2^21 s
    /// (about 24 days) and precision is still about 0.5 µs at the 2^32 s
    /// limit. The RP2350's floating-point unit only handles `f32`, so on
    /// that chip `f64` arithmetic runs in software and is much slower.
    pub fn to_f64(&self) -> f64 {
        self.to_fixed() as f64 / Self::ONE_SECOND as f64
    }
}

/// Adds two times on the same clock.
///
/// A carry out of the fraction rolls into the seconds. Overflow past
/// 2^32 seconds follows the integer rules: a panic in debug builds and
/// wrap-around in release builds.
impl<C: ClockId> Add for Time<C>
{
    type Output = Time<C>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_fixed(self.to_fixed() + rhs.to_fixed())
    }
}

/// Multiplies two times on the same clock.
///
/// The product of two 32.32 values is a 64.64 value; it is computed in a
/// `u128` so nothing is lost, then shifted back down to 32.32. Fractional
/// bits below 2^-32 s are truncated (rounded toward zero). A result of
/// 2^32 seconds or more follows the integer rules: a panic in debug builds
/// and wrap-around in release builds.
impl <C: ClockId> Mul for Time<C>
{
    type Output = Time<C>;

    fn mul(self, rhs: Self) -> Self::Output {
        let product = (self.to_fixed() as u128 * rhs.to_fixed() as u128) >> Self::FRAC_BITS;
        debug_assert!(product <= u64::MAX as u128, "Time multiplication overflow");
        Self::from_fixed(product as u64)
    }
}

/// A source of the current time on one particular clock.
///
/// `ThisTime` is an **associated type**: a type the implementer picks once,
/// in its `impl` block. It fixes which clock `now` reports on, so a driver
/// for a GPS receiver can only ever return `Time<Gps>`.
///
/// `now` takes `&self` so that only code holding the driver, and with it
/// proof that the hardware has been started, can read the time.
pub trait Clock
{
    /// The clock this source reports on.
    type ThisTime: ClockId;
    /// Reads the current time.
    fn now(&self) -> Time<Self::ThisTime>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Test;
    impl ClockId for Test { const NAME: &'static str = "TEST"; }

    const HALF: u32 = 1 << 31;

    fn t(seconds: u32, subseconds: u32) -> Time<Test> {
        Time::new(seconds, subseconds)
    }

    fn parts(time: Time<Test>) -> (u32, u32) {
        (time.seconds, time.subseconds)
    }

    #[test]
    fn add_whole_seconds() {
        assert_eq!(parts(t(2, 0) + t(3, 0)), (5, 0));
    }

    #[test]
    fn add_carries_fraction_into_seconds() {
        assert_eq!(parts(t(1, HALF) + t(1, HALF)), (3, 0));
        assert_eq!(parts(t(0, u32::MAX) + t(0, 1)), (1, 0));
    }

    #[test]
    fn mul_whole_seconds() {
        assert_eq!(parts(t(3, 0) * t(4, 0)), (12, 0));
    }

    #[test]
    fn mul_fractions() {
        // 1.5 * 1.5 = 2.25
        assert_eq!(parts(t(1, HALF) * t(1, HALF)), (2, 1 << 30));
        // 0.5 * 0.5 = 0.25
        assert_eq!(parts(t(0, HALF) * t(0, HALF)), (0, 1 << 30));
    }

    #[test]
    fn mul_by_one_is_identity() {
        assert_eq!(parts(t(7, 12345) * t(1, 0)), (7, 12345));
    }

    #[test]
    fn new_stores_parts() {
        assert_eq!(parts(Time::<Test>::new(42, HALF)), (42, HALF));
    }

    #[test]
    fn to_f64_is_exact_for_small_times() {
        assert_eq!(t(0, 0).to_f64(), 0.0);
        assert_eq!(t(1, HALF).to_f64(), 1.5);
        // The smallest step, 2^-32 s, survives.
        assert_eq!(t(0, 1).to_f64(), 1.0 / 4294967296.0);
        assert_eq!(t(1000, 1).to_f64(), 1000.0 + 1.0 / 4294967296.0);
    }

    #[test]
    fn to_f32_is_exact_for_representable_times() {
        assert_eq!(t(0, 0).to_f32(), 0.0);
        assert_eq!(t(1, HALF).to_f32(), 1.5);
        assert_eq!(t(3, 1 << 30).to_f32(), 3.25);
    }

    #[test]
    fn to_f32_rounds_to_nearest() {
        // 1 + 2^-32 is closer to 1.0 than to the next f32 above it.
        assert_eq!(t(1, 1).to_f32(), 1.0);
        // The largest time rounds up to 2^32.
        assert_eq!(t(u32::MAX, u32::MAX).to_f32(), 4294967296.0);
    }

    #[test]
    fn from_ticks_splits_seconds_and_fraction() {
        assert_eq!(parts(Time::<Test>::from_ticks(1_500_000, 1_000_000)), (1, HALF));
        assert_eq!(parts(Time::<Test>::from_ticks(3, 4)), (0, 3 << 30));
        assert_eq!(parts(Time::<Test>::from_ticks(0, 150_000_000)), (0, 0));
    }

    #[test]
    fn from_ticks_truncates_fraction() {
        // 1 µs is 4294.967296 units of 2^-32 s.
        assert_eq!(parts(Time::<Test>::from_ticks(1, 1_000_000)), (0, 4294));
    }

    #[test]
    fn from_ticks_at_one_tick_below_a_second() {
        let time = Time::<Test>::from_ticks(149_999_999, 150_000_000);
        assert_eq!(time.seconds, 0);
        assert!(time.to_f64() < 1.0 && time.to_f64() > 0.999_999_99);
    }

    #[test]
    fn times_compare_in_time_order() {
        assert!(t(1, u32::MAX) < t(2, 0));
        assert!(t(2, 1) > t(2, 0));
    }
}
