//! Calendar dates and the two clocks most systems name: [`Utc`] and
//! [`Met`].
//!
//! A receiver such as GPS reports the time as a calendar date and a time of
//! day; a filesystem stamps files the same way. [`Civil`] holds that form,
//! and [`Time::<Utc>::from_civil`](Time::from_civil) and
//! [`Time::<Utc>::to_civil`](Time::to_civil) convert it to and from
//! seconds since the Unix epoch, with the day-count algorithms of Howard
//! Hinnant, "chrono-Compatible Low-Level Date Algorithms"
//! (`days_from_civil`, `civil_from_days`).

use super::{ClockId, Time};

/// Coordinated Universal Time, as seconds since 1970-01-01T00:00:00Z: the
/// POSIX convention, which does not count leap seconds (a leap second
/// repeats the second before it). The 32-bit seconds of a [`Time`] reach
/// 2106-02-07.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Utc;

impl ClockId for Utc {
    const NAME: &'static str = "UTC";
}

/// Mission elapsed time: seconds since the system started. Its epoch is
/// whatever the system calls its start; what it means in UTC is a
/// correlation someone has to measure (see
/// [`DisciplinedClock`](super::DisciplinedClock)).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Met;

impl ClockId for Met {
    const NAME: &'static str = "MET";
}

/// A UTC date and time of day, to the millisecond.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Civil {
    /// Year, 1970 to 2105.
    pub year: u16,
    /// Month, 1 to 12.
    pub month: u8,
    /// Day of the month, 1 to 31.
    pub day: u8,
    /// Hour, 0 to 23.
    pub hour: u8,
    /// Minute, 0 to 59.
    pub minute: u8,
    /// Second, 0 to 59 (a leap second, 60, is not representable; see
    /// [`Utc`]).
    pub second: u8,
    /// Millisecond, 0 to 999.
    pub millisecond: u16,
}

/// Days from 1970-01-01 to `year`-`month`-`day` in the proleptic Gregorian
/// calendar (Hinnant's `days_from_civil`).
const fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    // Count years from March, so the leap day is the last day of the year.
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    // Year of the 400-year era, 0 to 399.
    let yoe = y - era * 400;
    // Day of the March-based year, 0 to 365.
    let mp = if month > 2 { month - 3 } else { month + 9 } as i64;
    let doy = (153 * mp + 2) / 5 + day as i64 - 1;
    // Day of the era, 0 to 146 096.
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    // 719 468 days from 0000-03-01 to 1970-01-01.
    era * 146_097 + doe - 719_468
}

/// The date `days` after 1970-01-01, as (year, month, day) (Hinnant's
/// `civil_from_days`).
const fn civil_from_days(days: i64) -> (i64, u32, u32) {
    // Shift the epoch to 0000-03-01.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}

/// Days in `month` of `year`.
const fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        2 if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl Time<Utc> {
    /// The UTC time `civil` names, its millisecond rounded up to the next
    /// 2^-32 s, or `None` if any field is out of range
    /// (including dates outside 1970 to 2105, and 31 April).
    pub const fn from_civil(civil: &Civil) -> Option<Self> {
        let c = civil;
        if c.year < 1970 || c.year > 2105 || c.month < 1 || c.month > 12 || c.day < 1 {
            return None;
        }
        if c.day > days_in_month(c.year, c.month) || c.hour > 23 || c.minute > 59 || c.second > 59 {
            return None;
        }
        if c.millisecond > 999 {
            return None;
        }
        let days = days_from_civil(c.year as i64, c.month as u32, c.day as u32);
        let seconds = days * 86_400 + c.hour as i64 * 3600 + c.minute as i64 * 60 + c.second as i64;
        // 1970 to 2105 always fits 32 bits.
        // The fraction rounded up to whole 2^-32 s, so that
        // `to_civil`, which rounds down, gives the same millisecond back.
        let fraction = ((c.millisecond as u64) << 32).div_ceil(1000);
        Some(Time::new(seconds as u32, fraction as u32))
    }

    /// This time as a calendar date and time of day, the fraction truncated
    /// to whole milliseconds.
    pub const fn to_civil(&self) -> Civil {
        let seconds = self.seconds as i64;
        let (year, month, day) = civil_from_days(seconds / 86_400);
        let in_day = seconds % 86_400;
        // Units of 2^-32 s to milliseconds, rounded down.
        let millisecond = ((self.subseconds as u64 * 1000) >> 32) as u16;
        Civil {
            year: year as u16,
            month: month as u8,
            day: day as u8,
            hour: (in_day / 3600) as u8,
            minute: (in_day / 60 % 60) as u8,
            second: (in_day % 60) as u8,
            millisecond,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn civil(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8, millisecond: u16) -> Civil {
        Civil { year, month, day, hour, minute, second, millisecond }
    }

    #[test]
    fn known_unix_times() {
        let t = |c: Civil| Time::<Utc>::from_civil(&c).unwrap();
        assert_eq!(t(civil(1970, 1, 1, 0, 0, 0, 0)), Time::new(0, 0));
        // `date -u -d 2000-03-01 +%s`
        assert_eq!(t(civil(2000, 3, 1, 0, 0, 0, 0)), Time::new(951_868_800, 0));
        // `date -u -d '2026-11-10 12:34:56' +%s`
        assert_eq!(t(civil(2026, 11, 10, 12, 34, 56, 0)), Time::new(1_794_314_096, 0));
        // A leap day, and the half second.
        assert_eq!(t(civil(2024, 2, 29, 23, 59, 59, 500)), Time::new(1_709_251_199, 1 << 31));
    }

    #[test]
    fn round_trips_every_day_for_a_century() {
        // Every day from 1970 to 2105, at a time with milliseconds.
        let mut day = 0u32;
        while day < 49_673 {
            let mut c = Time::<Utc>::new(day * 86_400 + 45_296, 0).to_civil();
            assert_eq!((c.hour, c.minute, c.second, c.millisecond), (12, 34, 56, 0));
            c.millisecond = 789;
            assert_eq!(Time::<Utc>::from_civil(&c).unwrap().to_civil(), c);
            day += 1;
        }
    }

    #[test]
    fn rejects_impossible_dates() {
        let bad = [
            civil(2025, 2, 29, 0, 0, 0, 0),
            civil(2026, 4, 31, 0, 0, 0, 0),
            civil(2026, 13, 1, 0, 0, 0, 0),
            civil(2026, 1, 0, 0, 0, 0, 0),
            civil(2026, 1, 1, 24, 0, 0, 0),
            civil(2026, 1, 1, 0, 0, 60, 0),
            civil(2026, 1, 1, 0, 0, 0, 1000),
            civil(1969, 12, 31, 0, 0, 0, 0),
            civil(2106, 1, 1, 0, 0, 0, 0),
        ];
        for c in bad {
            assert_eq!(Time::<Utc>::from_civil(&c), None, "{c:?}");
        }
        // 2000 is a leap year, 1900 rules aside.
        assert!(Time::<Utc>::from_civil(&civil(2000, 2, 29, 0, 0, 0, 0)).is_some());
    }
}
