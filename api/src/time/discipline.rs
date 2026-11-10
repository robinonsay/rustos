//! [`DisciplinedClock`]: a free-running counter steered to a reference
//! clock, such as a GPS receiver's pulse per second.
//!
//! # The model
//!
//! Three clocks are involved:
//!
//! * the **source** `S`, a hardware counter that never stops or jumps but
//!   runs at its crystal's rate, a few tens of parts per million off;
//! * the **output** `M`, what [`Clock::now`] returns: the source, rescaled.
//!   It starts at the source's reading and is never stepped, so its
//!   readings only ever increase, and timestamps taken from it can be
//!   subtracted and sorted. Its rate is steered so that its seconds are
//!   the reference's seconds;
//! * the **reference** `R`, the clock the system trusts (UTC from GPS),
//!   seen only at instants a receiver marks: "at this source reading, the
//!   reference read this".
//!
//! The output and the reference differ by an offset (the *correlation*),
//! measured at the first reference mark: `reference = output + offset`.
//! The output is then steered so the offset stays right. This is the
//! spacecraft-clock arrangement of mission elapsed time plus a time
//! correlation factor: on-board timestamps are MET, never stepped, and UTC
//! is MET plus a correlation the ground (here, the receiver) supplies.
//!
//! # Steering
//!
//! Each mark ([`Discipliner::pulse`]) gives the phase error: the reference
//! time the receiver reported minus the one the output and the offset
//! predicted. A phase error larger than
//! [`DisciplineConfig::step_threshold`] is corrected at once by replacing
//! the offset, a *step*, which moves no output timestamp. A smaller one is
//! removed by a proportional-integral loop with time constant τ
//! ([`DisciplineConfig::time_constant_s`]): the output runs at
//! `1 + frequency + error / τ` times the source's rate until the next
//! mark, and the frequency term accumulates `error / 4τ²` each mark, so it
//! learns the crystal's offset and holds it when marks stop. These are the
//! gains of a critically damped second-order loop with marks a second
//! apart.
//!
//! Every change of rate is anchored at the moment it is made: the output
//! at that source reading is the same under the old rate and the new, so
//! the output never jumps.
//!
//! # Sharing
//!
//! A `DisciplinedClock` is meant to be one `static` read by every part of
//! the system, on any core, through `&self`. Its state is a few words
//! behind a sequence lock: the one writer (the [`Discipliner`]) bumps a
//! counter to odd, writes, and bumps it to even; a reader retries if the
//! counter was odd or changed while it read. Readers never block the
//! writer, and nothing is locked. It needs only 32-bit atomics, which
//! every Armv8-M core has.
//!
//! Only one `Discipliner` is ever handed out
//! ([`DisciplinedClock::discipliner`]), so exactly one part of the system
//! can steer the clock, and the compiler enforces it: everything else
//! holds the clock and can only read it.

use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering, fence};

use super::{Clock, ClockId, Disciplined, Duration, Time};

/// How a [`DisciplinedClock`] responds to its marks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisciplineConfig {
    /// τ, in seconds: how quickly a phase error is steered out. Longer
    /// averages more noise in the marks away; shorter follows faster. 0 is
    /// taken as 1.
    pub time_constant_s: u32,
    /// A phase error larger than this replaces the correlation instead of
    /// being steered out: the reference jumped, or the mark was paired with
    /// the wrong reference time.
    pub step_threshold: Duration,
    /// The largest rate correction, in parts per billion, either way. A
    /// crystal is a few tens of ppm off; this bounds what a bad mark can do.
    pub max_rate_ppb: u32,
    /// With no mark for this long, a synchronized clock reports
    /// [`DisciplineState::Holdover`].
    pub holdover_after: Duration,
}

impl DisciplineConfig {
    /// τ = 16 s, steps above 100 ms, rate within ±500 ppm, holdover after
    /// 3 s without a mark: for one mark a second with up to a millisecond of
    /// noise.
    pub const DEFAULT: Self = Self {
        time_constant_s: 16,
        step_threshold: Duration::from_millis(100),
        max_rate_ppb: 500_000,
        holdover_after: Duration::from_secs(3),
    };
}

impl Default for DisciplineConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Whether a [`DisciplinedClock`] knows the reference time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DisciplineState {
    /// No mark yet: the output runs at the source's rate and the reference
    /// time is unknown.
    Free = 0,
    /// Marks are arriving: the output is being steered.
    Locked = 1,
    /// Marks stopped arriving: the correlation and the learned frequency
    /// still hold, and the error grows with the crystal's drift.
    Holdover = 2,
}

/// How a [`DisciplinedClock`] is doing, for telemetry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisciplineStatus<M: ClockId, R: ClockId> {
    /// See [`DisciplineState`].
    pub state: DisciplineState,
    /// The reference time at the output's epoch: `reference = output +
    /// epoch`. `None` while [`Free`](DisciplineState::Free).
    pub epoch: Option<Time<R>>,
    /// The learned frequency correction, parts per billion: how much
    /// faster than the source the output runs, steady-state.
    pub freq_ppb: i32,
    /// The rate correction in force now, parts per billion: the frequency
    /// plus the phase term.
    pub rate_ppb: i32,
    /// The phase error at the last mark, nanoseconds: reference minus
    /// prediction, positive when the output was behind. Saturates at
    /// ±2.1 s; 0 for a mark that stepped.
    pub phase_error_ns: i32,
    /// Marks taken.
    pub pulses: u32,
    /// Marks that replaced the correlation instead of being steered out,
    /// the first included.
    pub steps: u32,
    /// The output time of the last mark. `None` before the first.
    pub last_pulse: Option<Time<M>>,
}

/// What [`Discipliner::pulse`] did with a mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PulseOutcome {
    /// The correlation was set: the first mark, or one whose error was over
    /// the step threshold.
    Stepped,
    /// The error was steered.
    Slewed {
        /// The phase error, nanoseconds, as in
        /// [`DisciplineStatus::phase_error_ns`].
        phase_error_ns: i32,
    },
}

/// The shared state, as plain values. Times are raw 32.32 fixed point;
/// `rate` and `freq` are dimensionless, in units of 2^-32.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct State {
    anchor_src: u64,
    anchor_out: u64,
    rate: i32,
    freq: i32,
    offset: u64,
    synced: bool,
    phase_error_ns: i32,
    pulses: u32,
    steps: u32,
    last_pulse: u64,
}

impl State {
    /// The output time at source time `src`: the anchor plus the elapsed
    /// source time, rescaled by `1 + rate`. `src` a little before the
    /// anchor (read just before another core moved it) still works.
    fn out(&self, src: u64) -> u64 {
        let d = src.wrapping_sub(self.anchor_src) as i64;
        let adjust = ((d as i128 * self.rate as i128) >> 32) as i64;
        self.anchor_out.wrapping_add(d.wrapping_add(adjust) as u64)
    }
}

/// Words of shared state, after the sequence counter.
const WORDS: usize = 14;

/// Parts per billion in units of 2^-32, rounded toward zero.
const fn ppb_to_q32(ppb: i64) -> i64 {
    (ppb << 32) / 1_000_000_000
}

/// Units of 2^-32 in parts per billion, rounded toward zero.
const fn q32_to_ppb(q: i64) -> i32 {
    ((q * 1_000_000_000) >> 32) as i32
}

/// A fixed-point span, signed, in nanoseconds, saturating at the `i32`
/// limits.
fn to_ns(fixed: i64) -> i32 {
    let ns = (fixed as i128 * 1_000_000_000) >> 32;
    ns.clamp(i32::MIN as i128, i32::MAX as i128) as i32
}

/// A source clock `S` steered to reference `R`, read as clock `M`. See the
/// [module documentation](self).
pub struct DisciplinedClock<S: Clock, M: ClockId, R: ClockId> {
    /// The free-running counter.
    source: S,
    /// Steering constants.
    config: DisciplineConfig,
    /// The sequence counter: odd while the writer is mid-update.
    seq: AtomicU32,
    /// [`State`], packed into words.
    words: [AtomicU32; WORDS],
    /// Whether the [`Discipliner`] has been handed out.
    taken: AtomicBool,
    /// The output and reference clocks.
    _clocks: PhantomData<(M, R)>,
}

impl<S: Clock, M: ClockId, R: ClockId> DisciplinedClock<S, M, R> {
    /// A clock reading `source`, not yet synchronized: it reads 0 at
    /// `source`'s current time and runs at `source`'s rate until the
    /// first mark.
    pub fn new(source: S, config: DisciplineConfig) -> Self {
        let clock = Self {
            config,
            seq: AtomicU32::new(0),
            words: [const { AtomicU32::new(0) }; WORDS],
            taken: AtomicBool::new(false),
            _clocks: PhantomData,
            source,
        };
        let anchor_src = clock.source.now().to_fixed();
        clock.store(&State { anchor_src, ..State::default() });
        clock
    }

    /// The one handle that can steer this clock: `Some` on the first call,
    /// `None` ever after.
    pub fn discipliner(&self) -> Option<Discipliner<'_, S, M, R>> {
        if self.taken.swap(true, Ordering::AcqRel) {
            return None;
        }
        Some(Discipliner { clock: self, freq: 0 })
    }

    /// The source clock, for code that needs the raw counter (to time
    /// a mark, say).
    pub fn source(&self) -> &S {
        &self.source
    }

    /// The steering constants.
    pub fn config(&self) -> &DisciplineConfig {
        &self.config
    }

    /// Write `state`. Only the discipliner calls this (and `new`, before
    /// anyone else can see the clock), so there is one writer at a time.
    fn store(&self, s: &State) {
        let seq = self.seq.load(Ordering::Relaxed);
        // Odd: readers retry from here on.
        self.seq.store(seq.wrapping_add(1), Ordering::Relaxed);
        fence(Ordering::Release);
        let words = [
            s.anchor_src as u32,
            (s.anchor_src >> 32) as u32,
            s.anchor_out as u32,
            (s.anchor_out >> 32) as u32,
            s.rate as u32,
            s.freq as u32,
            s.offset as u32,
            (s.offset >> 32) as u32,
            s.synced as u32,
            s.phase_error_ns as u32,
            s.pulses,
            s.steps,
            s.last_pulse as u32,
            (s.last_pulse >> 32) as u32,
        ];
        for (slot, value) in self.words.iter().zip(words) {
            slot.store(value, Ordering::Relaxed);
        }
        // Even again: the update is visible as a whole.
        self.seq.store(seq.wrapping_add(2), Ordering::Release);
    }

    /// A consistent copy of the state.
    fn load(&self) -> State {
        loop {
            let before = self.seq.load(Ordering::Acquire);
            if before & 1 == 0 {
                let w: [u32; WORDS] = core::array::from_fn(|i| self.words[i].load(Ordering::Relaxed));
                fence(Ordering::Acquire);
                if self.seq.load(Ordering::Relaxed) == before {
                    return State {
                        anchor_src: w[0] as u64 | (w[1] as u64) << 32,
                        anchor_out: w[2] as u64 | (w[3] as u64) << 32,
                        rate: w[4] as i32,
                        freq: w[5] as i32,
                        offset: w[6] as u64 | (w[7] as u64) << 32,
                        synced: w[8] & 1 != 0,
                        phase_error_ns: w[9] as i32,
                        pulses: w[10],
                        steps: w[11],
                        last_pulse: w[12] as u64 | (w[13] as u64) << 32,
                    };
                }
            }
            // The writer is mid-update.
            core::hint::spin_loop();
        }
    }
}

impl<S: Clock, M: ClockId, R: ClockId> Clock for DisciplinedClock<S, M, R> {
    type ThisTime = M;

    fn now(&self) -> Time<M> {
        let state = self.load();
        Time::from_fixed(state.out(self.source.now().to_fixed()))
    }
}

impl<S: Clock, M: ClockId, R: ClockId> Disciplined for DisciplinedClock<S, M, R> {
    type Reference = R;

    fn to_reference(&self, t: Time<M>) -> Option<Time<R>> {
        let state = self.load();
        // reference = output + offset, both 32.32 fixed point.
        state.synced.then(|| Time::from_fixed(t.to_fixed().wrapping_add(state.offset)))
    }

    fn status(&self) -> DisciplineStatus<M, R> {
        let s = self.load();
        let now = s.out(self.source.now().to_fixed());
        let state = if !s.synced {
            DisciplineState::Free
        } else if now.wrapping_sub(s.last_pulse) > self.config.holdover_after.to_fixed() {
            DisciplineState::Holdover
        } else {
            DisciplineState::Locked
        };
        DisciplineStatus {
            state,
            epoch: s.synced.then(|| Time::from_fixed(s.offset)),
            freq_ppb: q32_to_ppb(s.freq as i64),
            rate_ppb: q32_to_ppb(s.rate as i64),
            phase_error_ns: s.phase_error_ns,
            pulses: s.pulses,
            steps: s.steps,
            last_pulse: (s.pulses > 0).then(|| Time::from_fixed(s.last_pulse)),
        }
    }
}

/// The one handle that steers a [`DisciplinedClock`]. See the [module
/// documentation](self).
pub struct Discipliner<'a, S: Clock, M: ClockId, R: ClockId> {
    /// The clock it steers.
    clock: &'a DisciplinedClock<S, M, R>,
    /// The frequency term, in units of 2^-48: finer than the shared
    /// 2^-32, so that small errors still move it.
    freq: i64,
}

impl<'a, S: Clock, M: ClockId, R: ClockId> Discipliner<'a, S, M, R> {
    /// The clock this steers.
    pub fn clock(&self) -> &'a DisciplinedClock<S, M, R> {
        self.clock
    }

    /// The source clock's time now: what to record when a mark is seen.
    pub fn source_now(&self) -> Time<S::ThisTime> {
        self.clock.source.now()
    }

    /// A mark: when the source read `at`, the reference read `reference`.
    /// Call it once per mark, in order, after `at`. See the [module
    /// documentation](self) for what it does.
    pub fn pulse(&mut self, at: Time<S::ThisTime>, reference: Time<R>) -> PulseOutcome {
        let config = &self.clock.config;
        let old = self.clock.load();
        let out_at = old.out(at.to_fixed());
        let mut new = old;
        new.pulses = old.pulses.wrapping_add(1);
        new.last_pulse = out_at;
        // reference − (output + offset), as a signed span.
        let error = reference.to_fixed().wrapping_sub(out_at.wrapping_add(old.offset)) as i64;
        let outcome = if !old.synced || error.unsigned_abs() > config.step_threshold.to_fixed() {
            // Set the correlation; the rate keeps what it learned.
            new.offset = reference.to_fixed().wrapping_sub(out_at);
            new.synced = true;
            new.steps = old.steps.wrapping_add(1);
            new.phase_error_ns = 0;
            PulseOutcome::Stepped
        } else {
            let tau = config.time_constant_s.max(1) as i64;
            let max = ppb_to_q32(config.max_rate_ppb as i64);
            // Integral: error / 4τ², kept to 2^-48.
            self.freq = (self.freq + (error << 16) / (4 * tau * tau)).clamp(-max << 16, max << 16);
            // Proportional: error / τ, for the coming second.
            let rate = ((self.freq >> 16) + error / tau).clamp(-max, max);
            new.freq = (self.freq >> 16) as i32;
            new.rate = rate as i32;
            new.phase_error_ns = to_ns(error);
            PulseOutcome::Slewed { phase_error_ns: new.phase_error_ns }
        };
        // Re-anchor now, so the output is continuous across the change.
        let now = self.clock.source.now().to_fixed();
        new.anchor_src = now;
        new.anchor_out = old.out(now);
        self.clock.store(&new);
        outcome
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use super::*;
    use crate::time::civil::{Met, Utc};

    /// The source's clock domain.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Raw;
    impl ClockId for Raw {
        const NAME: &'static str = "RAW";
    }

    /// A 1 MHz counter the test sets by hand.
    struct Counter {
        us: Cell<u64>,
    }
    impl Clock for Counter {
        type ThisTime = Raw;
        fn now(&self) -> Time<Raw> {
            Time::from_ticks(self.us.get(), 1_000_000)
        }
    }

    type Clk = DisciplinedClock<Counter, Met, Utc>;

    fn clock() -> Clk {
        DisciplinedClock::new(Counter { us: Cell::new(0) }, DisciplineConfig::DEFAULT)
    }

    /// The UTC second `k` after an arbitrary 2026 epoch.
    fn utc(k: u32) -> Time<Utc> {
        Time::new(1_794_314_096 + k, 0)
    }

    /// Run `seconds` marks of a source `ppm` fast, the first at true time
    /// `start_s` after power-on, each handled 300 ms after it. Returns the
    /// worst |output − true elapsed| over the last 20 marks, in µs.
    fn run(clk: &Clk, ppm: f64, start_s: u32, seconds: u32) -> f64 {
        let mut d = clk.discipliner().unwrap();
        let mut worst: f64 = 0.0;
        for k in 0..seconds {
            let truth = (start_s + k) as f64;
            // The counter's reading at true time `t`.
            let src = |t: f64| (t * (1.0 + ppm * 1e-6) * 1e6) as u64;
            clk.source().us.set(src(truth));
            let at = d.source_now();
            clk.source().us.set(src(truth + 0.3));
            d.pulse(at, utc(k));
            if k + 20 >= seconds {
                // UTC predicted for the mark, against the truth.
                let predicted = clk.to_reference(Time::from_fixed(clk.load().last_pulse)).unwrap();
                let err = predicted.to_f64() - utc(k).to_f64();
                worst = worst.max(err.abs() * 1e6);
            }
        }
        worst
    }

    #[test]
    fn free_until_the_first_mark_then_locked() {
        let clk = clock();
        assert_eq!(clk.status().state, DisciplineState::Free);
        assert_eq!(clk.to_reference(clk.now()), None);
        let mut d = clk.discipliner().unwrap();
        clk.source().us.set(5_000_000);
        assert_eq!(d.pulse(clk.source().now(), utc(0)), PulseOutcome::Stepped);
        let s = clk.status();
        assert_eq!((s.state, s.steps, s.pulses), (DisciplineState::Locked, 1, 1));
        // The mark at MET 5 s was UTC second 0, so MET 0 was 5 s earlier.
        assert_eq!(s.epoch, Some(Time::new(1_794_314_091, 0)));
        assert_eq!(clk.to_reference(Time::new(5, 0)), Some(utc(0)));
    }

    #[test]
    fn only_one_discipliner() {
        let clk = clock();
        let first = clk.discipliner();
        assert!(first.is_some());
        assert!(clk.discipliner().is_none());
    }

    #[test]
    fn learns_a_fast_crystal_and_tracks_to_microseconds() {
        let clk = clock();
        let worst = run(&clk, 30.0, 2, 300);
        // A 30 ppm fast source needs the output slowed by about 30 ppm.
        let s = clk.status();
        assert!((s.freq_ppb + 30_000).abs() < 500, "{s:?}");
        // The source counts whole microseconds, so a microsecond or two.
        assert!(worst < 3.0, "worst {worst} us");
        assert_eq!(s.steps, 1);
    }

    #[test]
    fn learns_a_slow_crystal_too() {
        let clk = clock();
        let worst = run(&clk, -45.0, 0, 300);
        assert!((clk.status().freq_ppb - 45_000).abs() < 500, "{:?}", clk.status());
        assert!(worst < 3.0, "worst {worst} us");
    }

    #[test]
    fn tracks_through_millisecond_polling() {
        // Marks timed by polling an edge latch every millisecond, stamped
        // at the middle of the poll interval: up to ±0.5 ms of error each.
        let clk = clock();
        let mut d = clk.discipliner().unwrap();
        let ppm = 25.0;
        let src = |t: f64| (t * (1.0 + ppm * 1e-6) * 1e6) as u64;
        // The polls run on the source's millisecond grid; at 25 ppm the
        // edges sweep across it, 25 µs a second.
        let mut worst: f64 = 0.0;
        for k in 0..600u32 {
            let truth = 1.000_37 + k as f64;
            let edge = src(truth);
            // The poll after the edge, and the one before it.
            let after = edge.div_ceil(1000) * 1000;
            let at = Time::from_ticks(after - 500, 1_000_000);
            clk.source().us.set(src(truth + 0.2));
            d.pulse(at, utc(k));
            if k >= 300 {
                // The true error: UTC at the edge, as the clock tells it.
                let out = Time::from_fixed(clk.load().out(Time::<Raw>::from_ticks(edge, 1_000_000).to_fixed()));
                let err = clk.to_reference(out).unwrap().to_f64() - utc(k).to_f64();
                worst = worst.max(err.abs());
            }
        }
        let s = clk.status();
        assert!((s.freq_ppb + 25_000).abs() < 3_000, "{s:?}");
        // Well inside the capture's own ±0.5 ms: about 225 µs at worst.
        assert!(worst < 300e-6, "worst {} us", worst * 1e6);
    }

    #[test]
    fn output_never_goes_backwards_or_jumps() {
        let clk = clock();
        let mut d = clk.discipliner().unwrap();
        let mut last = clk.now();
        let mut src = 0u64;
        for k in 0..200u32 {
            // A source 80 ppm fast, read every 100 ms, a mark every second
            // with a few hundred microseconds of noise either way.
            for _ in 0..10 {
                src += 100_008;
                clk.source().us.set(src);
                let now = clk.now();
                assert!(now >= last);
                // 100 ms of source is 100 ms of output, to well under 1%.
                assert!((now - last).as_micros() < 101_000);
                last = now;
            }
            let noise = [0, 400, -300, 150, -450][k as usize % 5];
            let at = Time::from_ticks((src as i64 - 300_000 + noise) as u64, 1_000_000);
            d.pulse(at, utc(k));
        }
        assert_eq!(clk.status().steps, 1);
    }

    #[test]
    fn a_large_error_steps_the_correlation_not_the_output() {
        let clk = clock();
        run(&clk, 0.0, 0, 30);
        let mut d = clk.discipliner_for_test();
        let before = clk.now();
        // The reference says a whole second later than expected.
        clk.source().us.set(30_000_000);
        let at = clk.source().now();
        clk.source().us.set(30_300_000);
        assert_eq!(d.pulse(at, utc(31)), PulseOutcome::Stepped);
        assert_eq!(clk.status().steps, 2);
        // The output did not move: one second of source after `before`
        // (read at 29.3 s) is one second of output.
        let span = clk.now() - before;
        assert!(span.as_millis() >= 999 && span.as_millis() <= 1001, "{span:?}");
        // UTC did.
        let at_out = Time::from_fixed(clk.load().last_pulse);
        assert_eq!(clk.to_reference(at_out), Some(utc(31)));
    }

    #[test]
    fn holdover_when_marks_stop() {
        let clk = clock();
        run(&clk, 10.0, 0, 200);
        assert_eq!(clk.status().state, DisciplineState::Locked);
        // Four seconds of silence.
        clk.source().us.set(clk.source().us.get() + 4_000_000);
        let s = clk.status();
        assert_eq!(s.state, DisciplineState::Holdover);
        // The learned frequency still applies.
        assert!(s.freq_ppb < -5_000);
    }

    #[test]
    fn rate_is_bounded() {
        let clk = clock();
        let mut d = clk.discipliner().unwrap();
        d.pulse(clk.source().now(), utc(0));
        // A mark 90 ms late, every second: just under the step threshold.
        for k in 1..100u32 {
            clk.source().us.set(k as u64 * 1_000_000 + 90_000);
            let at = clk.source().now();
            d.pulse(at, utc(k));
        }
        let s = clk.status();
        assert!(s.rate_ppb.unsigned_abs() <= 500_000, "{s:?}");
        assert!(s.freq_ppb.unsigned_abs() <= 500_000, "{s:?}");
    }

    impl Clk {
        /// A second discipliner on the same state, for a test that needs to
        /// carry on after [`run`] dropped the first.
        fn discipliner_for_test(&self) -> Discipliner<'_, Counter, Met, Utc> {
            Discipliner { clock: self, freq: (self.load().freq as i64) << 16 }
        }
    }
}
