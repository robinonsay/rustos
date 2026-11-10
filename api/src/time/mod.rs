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

use core::{marker::PhantomData, ops::{Add, Mul, Sub}};

pub mod civil;
pub mod discipline;

pub use civil::{Civil, Met, Utc};
pub use discipline::{
    DisciplineConfig, DisciplineState, DisciplineStatus, DisciplinedClock, Discipliner, PulseOutcome,
};

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

    /// The span from `earlier` to `self`, or `None` if `earlier` is
    /// actually later: the checked form of `self - earlier`, for when the
    /// order of two readings is not certain.
    pub const fn checked_sub(self, earlier: Self) -> Option<Duration> {
        // Integer subtraction fails exactly when the span would be negative.
        match self.to_fixed().checked_sub(earlier.to_fixed()) {
            Some(fixed) => Some(Duration { fixed }),
            None => None,
        }
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

/// The span between two times on the same clock: `later - earlier`.
///
/// The result is a [`Duration`], not a `Time`: the difference of two
/// points is a length, not another point on clock `C`. Both operands being
/// `Time<C>` means a reading from one clock cannot be subtracted from a
/// reading of another. If `rhs` is later than `self` the span would be
/// negative, which follows the integer rules: a panic in debug builds and
/// wrap-around in release builds; [`Time::checked_sub`] returns `None`
/// instead. On a clock that wraps (such as a 24-bit
/// counter), take differences of the raw counts instead.
impl<C: ClockId> Sub for Time<C>
{
    type Output = Duration;

    fn sub(self, rhs: Self) -> Duration {
        // Same 32.32 layout, so the difference is integer subtraction.
        Duration { fixed: self.to_fixed() - rhs.to_fixed() }
    }
}

/// The time `rhs` after `self`, on the same clock: a deadline is
/// `clock.now() + timeout`.
///
/// Going past 2^32 s follows the integer rules: a panic in debug builds and
/// wrap-around in release builds.
impl<C: ClockId> Add<Duration> for Time<C>
{
    type Output = Time<C>;

    fn add(self, rhs: Duration) -> Time<C> {
        // Same 32.32 layout, so this is integer addition.
        Self::from_fixed(self.to_fixed() + rhs.fixed)
    }
}

/// The time `rhs` before `self`, on the same clock.
///
/// Going back past clock `C`'s epoch follows the integer rules: a panic in
/// debug builds and wrap-around in release builds.
impl<C: ClockId> Sub<Duration> for Time<C>
{
    type Output = Time<C>;

    fn sub(self, rhs: Duration) -> Time<C> {
        // Same 32.32 layout, so this is integer subtraction.
        Self::from_fixed(self.to_fixed() - rhs.fixed)
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

/// A [`Clock`] steered to a reference clock, which can say what its
/// readings are on that reference.
///
/// The usual case is a free-running counter disciplined by a GPS
/// receiver: [`Clock::now`] gives mission elapsed time, never stepped, and
/// [`to_reference`](Self::to_reference) gives UTC once the receiver has
/// supplied one. [`DisciplinedClock`] is the implementation; who may steer
/// it is that type's business, not this trait's, so code that only reads
/// time takes a `C: Disciplined` and cannot steer.
pub trait Disciplined: Clock {
    /// The clock this one is steered to.
    type Reference: ClockId;

    /// `t`, a reading of this clock, on the reference clock; `None` until
    /// the correlation between the two is known.
    fn to_reference(&self, t: Time<Self::ThisTime>) -> Option<Time<Self::Reference>>;

    /// The reference time now, if known.
    fn reference_now(&self) -> Option<Time<Self::Reference>> {
        self.to_reference(self.now())
    }

    /// How the steering is going, for telemetry.
    fn status(&self) -> DisciplineStatus<Self::ThisTime, Self::Reference>;
}

/// A [`Clock`] that can also wait for a time on itself: what a scheduler
/// needs to pace its frames.
///
/// The LibJuno counterpart is `TIME_API_T::SleepTo`. "Sleep" here means
/// "do not return until": an implementation may busy-wait, wait for an
/// interrupt, or ask an operating system, as its platform allows.
pub trait Sleep: Clock {
    /// What [`sleep_until`](Self::sleep_until) reports when it cannot wait.
    type Error: crate::error::Error;

    /// Return once `deadline` has been reached on this clock: as soon as
    /// [`Clock::now`] would return a time not earlier than `deadline`. If
    /// `deadline` has already passed, return at once. Absolute deadlines,
    /// not relative delays, so that lateness in one frame does not push
    /// every later frame back.
    fn sleep_until(&self, deadline: Time<Self::ThisTime>) -> Result<(), Self::Error>;
}

/// A length of time, such as a timeout or a period, as 32.32 fixed-point
/// seconds.
///
/// A [`Time<C>`] is a *point* on clock `C`; a `Duration` is a *span*, and
/// is not tied to any clock: 100 ms is 100 ms whichever clock measures it.
/// It uses the same layout as `Time` (see the [module documentation](self)):
/// whole seconds in the upper 32 bits, units of 2^-32 s in the lower 32,
/// so spans up to about 136 years with about 233 ps resolution.
///
/// Conversions from a whole count of some unit ([`from_micros`] and
/// friends) truncate to whole 2^-32 s units, like [`Time::from_ticks`], and
/// saturate at [`Duration::MAX`] instead of wrapping. Conversions back to
/// a count say how they round: [`as_micros`] down, [`as_micros_ceil`] up.
/// A count converted in and back out with `_ceil` is unchanged, because
/// one microsecond is far more than one 2^-32 s unit.
///
/// [`from_micros`]: Self::from_micros
/// [`as_micros`]: Self::as_micros
/// [`as_micros_ceil`]: Self::as_micros_ceil
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration {
    /// 32.32 fixed-point seconds.
    fixed: u64,
}

impl Duration {
    /// No time at all.
    pub const ZERO: Duration = Duration { fixed: 0 };

    /// The longest span: 2^32 s less one 2^-32 s unit.
    pub const MAX: Duration = Duration { fixed: u64::MAX };

    /// One second in fixed-point units: 2^32.
    const ONE_SECOND: u64 = 1 << 32;

    /// `seconds` whole seconds plus `subseconds` units of 2^-32 s.
    pub const fn new(seconds: u32, subseconds: u32) -> Self {
        // Same packing as Time::to_fixed.
        Self { fixed: ((seconds as u64) << 32) | subseconds as u64 }
    }

    /// `ticks` periods of a `ticks_per_second` counter, truncated to whole
    /// 2^-32 s units and saturating at [`Duration::MAX`]. Panics if
    /// `ticks_per_second` is 0.
    pub const fn from_ticks(ticks: u64, ticks_per_second: u32) -> Self {
        // As u64 for the arithmetic below.
        let hz = ticks_per_second as u64;
        // Whole seconds; above u32::MAX the span does not fit.
        let seconds = ticks / hz;
        if seconds > u32::MAX as u64 {
            return Self::MAX;
        }
        // The remainder is below hz < 2^32, so the shift cannot overflow.
        let fraction = ((ticks % hz) << 32) / hz;
        // Both parts fit 32 bits.
        Self::new(seconds as u32, fraction as u32)
    }

    /// `secs` whole seconds. Always exact.
    pub const fn from_secs(secs: u32) -> Self {
        // No fraction.
        Self::new(secs, 0)
    }

    /// `millis` milliseconds, truncated to whole 2^-32 s units.
    pub const fn from_millis(millis: u64) -> Self {
        // A millisecond counter runs at 1000 Hz.
        Self::from_ticks(millis, 1_000)
    }

    /// `micros` microseconds, truncated to whole 2^-32 s units.
    pub const fn from_micros(micros: u64) -> Self {
        // A microsecond counter runs at 1 MHz.
        Self::from_ticks(micros, 1_000_000)
    }

    /// Whole seconds, the fraction discarded.
    pub const fn as_secs(&self) -> u32 {
        // The upper 32 bits.
        (self.fixed >> 32) as u32
    }

    /// This span in periods of a `ticks_per_second` counter, rounded down.
    pub const fn as_ticks(&self, ticks_per_second: u32) -> u64 {
        // 64.64 product in u128, back to whole ticks by dropping 32 bits.
        // fixed < 2^64 and the rate < 2^32, so the product is below 2^96
        // and the result below 2^64: it always fits a u64.
        ((self.fixed as u128 * ticks_per_second as u128) >> 32) as u64
    }

    /// This span in periods of a `ticks_per_second` counter, rounded up: the
    /// fewest ticks that last at least this long.
    pub const fn as_ticks_ceil(&self, ticks_per_second: u32) -> u64 {
        // Add one unit less than a tick before dropping the fraction. The
        // product is at most (2^64 − 1)(2^32 − 1), so adding 2^32 − 1 still
        // leaves it below 2^96: the result always fits a u64.
        ((self.fixed as u128 * ticks_per_second as u128 + (Self::ONE_SECOND as u128 - 1)) >> 32) as u64
    }

    /// Whole milliseconds, rounded down.
    pub const fn as_millis(&self) -> u64 {
        // A millisecond counter runs at 1000 Hz.
        self.as_ticks(1_000)
    }

    /// Whole microseconds, rounded down.
    pub const fn as_micros(&self) -> u64 {
        // A microsecond counter runs at 1 MHz.
        self.as_ticks(1_000_000)
    }

    /// Microseconds, rounded up: the fewest whole microseconds that last
    /// at least this long. What a timeout should be converted with, so it
    /// never fires early.
    pub const fn as_micros_ceil(&self) -> u64 {
        // A microsecond counter runs at 1 MHz.
        self.as_ticks_ceil(1_000_000)
    }

    /// `self + rhs`, or `None` if the sum exceeds [`Duration::MAX`].
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        // Fixed-point addition is integer addition.
        match self.fixed.checked_add(rhs.fixed) {
            Some(fixed) => Some(Self { fixed }),
            None => None,
        }
    }

    /// `self - rhs`, or `None` if `rhs` is longer than `self`.
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        // A span cannot be negative.
        match self.fixed.checked_sub(rhs.fixed) {
            Some(fixed) => Some(Self { fixed }),
            None => None,
        }
    }

    /// `self × rhs`, or `None` if the product exceeds [`Duration::MAX`].
    pub const fn checked_mul(self, rhs: u32) -> Option<Self> {
        // Scaling fixed point by an integer is integer multiplication.
        match self.fixed.checked_mul(rhs as u64) {
            Some(fixed) => Some(Self { fixed }),
            None => None,
        }
    }

    /// The raw 32.32 value, for exact arithmetic inside this crate
    /// (checking that one span is a whole multiple of another).
    pub(crate) const fn to_fixed(self) -> u64 {
        // The field itself.
        self.fixed
    }

    /// `self - rhs`, or [`Duration::ZERO`] if `rhs` is longer.
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        // Clamp at zero instead of failing.
        Self { fixed: self.fixed.saturating_sub(rhs.fixed) }
    }
}

/// The sum of two spans. Overflow past [`Duration::MAX`] follows the
/// integer rules, as for [`Time`]: a panic in debug builds, wrap-around in
/// release builds. Use [`Duration::checked_add`] where that matters.
impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Duration {
        // Integer addition of the fixed-point values.
        Duration { fixed: self.fixed + rhs.fixed }
    }
}

/// A span repeated `rhs` times. Overflow past [`Duration::MAX`] follows
/// the integer rules; use [`Duration::checked_mul`] where that matters.
impl Mul<u32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u32) -> Duration {
        // Integer multiplication of the fixed-point value.
        Duration { fixed: self.fixed * rhs as u64 }
    }
}

/// The difference of two spans. A negative result follows the integer
/// rules: a panic in debug builds, wrap-around in release builds. Use
/// [`Duration::checked_sub`] or [`Duration::saturating_sub`] where that
/// matters.
impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Duration {
        // Integer subtraction of the fixed-point values.
        Duration { fixed: self.fixed - rhs.fixed }
    }
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
    fn duration_units_convert_exactly_where_they_can() {
        assert_eq!(Duration::from_secs(3).as_micros(), 3_000_000);
        assert_eq!(Duration::from_millis(1500), Duration::new(1, HALF));
        assert_eq!(Duration::from_millis(250).as_millis(), 250);
        assert_eq!(Duration::new(2, HALF).as_secs(), 2);
    }

    #[test]
    fn duration_micros_round_trip_with_ceil() {
        // Every count in and back out unchanged, including the awkward ones.
        for us in [0, 1, 3, 999_999, 1_000_000, 16_777_215, 123_456_789] {
            assert_eq!(Duration::from_micros(us).as_micros_ceil(), us);
        }
        // Rounding down loses the last microsecond for non-exact values.
        assert_eq!(Duration::from_micros(1).as_micros(), 0);
    }

    #[test]
    fn duration_ceil_rounds_up_a_partial_unit() {
        // One 2^-32 s unit is a tiny fraction of a microsecond.
        assert_eq!(Duration::new(0, 1).as_micros_ceil(), 1);
        assert_eq!(Duration::new(0, 1).as_micros(), 0);
        assert_eq!(Duration::ZERO.as_micros_ceil(), 0);
    }

    #[test]
    fn duration_saturates_instead_of_wrapping() {
        assert_eq!(Duration::from_micros(u64::MAX), Duration::MAX);
        // The largest possible tick count fits: (2^32 − 2^-32) s × (2^32 − 1) Hz.
        assert_eq!(Duration::MAX.as_ticks_ceil(u32::MAX), 0xffff_ffff_0000_0000);
        assert_eq!(Duration::from_secs(1).saturating_sub(Duration::from_secs(2)), Duration::ZERO);
    }

    #[test]
    fn duration_times_an_integer() {
        assert_eq!(Duration::from_secs(1) * 3, Duration::from_secs(3));
        // 5 ms is not a whole number of 2^-32 s units (21 474 836.48), so
        // 200 of the truncated value fall 96 units (22 ns) short of 1 s.
        assert_eq!(Duration::from_secs(1).to_fixed() - (Duration::from_millis(5) * 200).to_fixed(), 96);
        assert_eq!(Duration::MAX.checked_mul(2), None);
        assert_eq!(Duration::from_secs(3).checked_mul(0), Some(Duration::ZERO));
    }

    #[test]
    fn duration_arithmetic() {
        let a = Duration::from_millis(1500);
        let b = Duration::from_millis(500);
        assert_eq!(a + b, Duration::from_secs(2));
        assert_eq!(a - b, Duration::from_secs(1));
        assert_eq!(b.checked_sub(a), None);
        assert_eq!(Duration::MAX.checked_add(Duration::new(0, 1)), None);
        assert!(b < a);
    }

    #[test]
    fn time_minus_time_is_the_span_between_them() {
        // 3.5 s - 1.25 s = 2.25 s.
        let span = t(3, HALF) - t(1, 1 << 30);
        assert_eq!(span, Duration::new(2, 1 << 30));
        // A time minus itself is no time at all.
        assert_eq!(t(7, 12345) - t(7, 12345), Duration::ZERO);
    }

    #[test]
    fn time_minus_time_borrows_from_the_seconds() {
        // 2.0 s - 1.5 s = 0.5 s: the fraction borrows a second.
        assert_eq!(t(2, 0) - t(1, HALF), Duration::new(0, HALF));
    }

    #[test]
    fn time_minus_duration_moves_back_along_the_clock() {
        assert_eq!(parts(t(10, 0) - Duration::from_millis(1500)), (8, HALF));
        assert_eq!(parts(t(5, 7) - Duration::ZERO), (5, 7));
    }

    #[test]
    fn time_plus_duration_moves_forward_along_the_clock() {
        // 8.5 s + 1.5 s = 10 s: the fraction carries into the seconds.
        assert_eq!(parts(t(8, HALF) + Duration::from_millis(1500)), (10, 0));
        // Adding then subtracting the same span is the identity.
        let d = Duration::from_micros(123_456);
        assert_eq!(t(4, 99) + d - d, t(4, 99));
        // And the span between a deadline and its start is the timeout.
        assert_eq!((t(4, 99) + d) - t(4, 99), d);
    }

    #[test]
    fn checked_sub_reports_the_wrong_order() {
        assert_eq!(t(3, 0).checked_sub(t(1, HALF)), Some(Duration::new(1, HALF)));
        assert_eq!(t(1, 0).checked_sub(t(1, 0)), Some(Duration::ZERO));
        assert_eq!(t(1, 0).checked_sub(t(1, 1)), None);
    }

    #[test]
    #[should_panic]
    fn subtracting_a_later_time_panics_in_debug() {
        // Debug builds check integer overflow, so a negative span panics.
        let _ = t(1, 0) - t(2, 0);
    }

    #[test]
    fn times_compare_in_time_order() {
        assert!(t(1, u32::MAX) < t(2, 0));
        assert!(t(2, 1) > t(2, 0));
    }
}
