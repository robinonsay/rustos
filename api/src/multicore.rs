//! Starting a second processor core: the launch handshake of the RP2350
//! bootrom, as pure logic over a [`Mailbox`].
//!
//! # The handshake
//!
//! After a reset, core 1 does not run the application. The bootrom parks it
//! in a low-power wait, where it listens on the inter-core FIFO from core 0
//! for six words, in order:
//!
//! ```text
//! 0, 0, 1, vector table, stack pointer, entry point
//! ```
//!
//! It echoes each word it takes back on the FIFO to core 0. Core 0 sends a
//! word, waits for that word to come back, and only then sends the next; any
//! other answer starts the sequence again from the first 0. Before each 0,
//! core 0 first empties the FIFO from core 1 and wakes core 1, which may be
//! waiting for room to send. Once core 1 has taken all six, it sets its
//! stack pointer and `VTOR` from words 4 and 5 and jumps to word 6.
//!
//! The repeated zeros and the start-over rule make the sequence robust to
//! where a recently reset core 1 is when it begins: "anywhere in its boot
//! code, up to and including going to sleep", so the procedure "may be
//! performed at any point after processor core 1 has been reset (either by
//! system reset, or explicitly resetting just processor core 1)". It is not
//! for a core 1 that is already running something else, or is part way
//! through a sequence an earlier, interrupted launch began: then the
//! echoes cannot tell core 0 where core 1 is. The procedure is in the
//! RP2350 datasheet, §5.3 "Launching Code On Processor Core 1" (PDF page
//! 376), as a C listing that this module follows step for step.
//!
//! With one addition: before each 0, the FIFO from core 1 must have stayed
//! empty for [`QUIET_US`], not merely read empty once. The listing's drain
//! catches only the echoes that have already arrived. If one more is still
//! on its way when the drain runs, core 0 takes that late echo as the answer
//! to its next word and is one echo behind from then on: every word it sends
//! gets the previous one's echo, and the sequence starts over forever. On
//! the chip core 1 answers within a few microseconds, so the listing works
//! in practice; waiting for a quiet FIFO makes it work whatever the timing,
//! which the model of core 1 in the tests below found out.
//!
//! # Why a state machine
//!
//! The datasheet's listing blocks on each FIFO operation. A core that holds
//! a hardware watchdog, or anything else with a deadline, cannot wait on
//! another core with no bound, so [`Launch::step`] never blocks: each call
//! does at most a bounded drain, one send and one receive, and returns
//! [`Step::Pending`] until the last echo arrives. The caller decides how long
//! to keep stepping, by its own clock. Nothing here touches hardware, so the
//! sequence is tested on the host against a model of core 1 (the tests at the
//! bottom of this file); the chip crate supplies the [`Mailbox`] and the three
//! addresses.

/// One processor core's end of the two inter-core FIFOs: a word FIFO to the
/// other core and a word FIFO from it.
///
/// Every method returns at once. On the RP2350 these are the SIO's
/// `FIFO_WR`, `FIFO_RD` and `FIFO_ST` registers, and [`wake`](Self::wake) is
/// the Arm `SEV` instruction.
pub trait Mailbox {
    /// Send `word` to the other core if its FIFO has room: `true` if it was
    /// sent, `false` if the FIFO was full and nothing was written. A word
    /// sent also wakes the other core, as [`wake`](Self::wake) does, in case
    /// it is waiting for one.
    fn try_send(&mut self, word: u32) -> bool;

    /// Take the oldest word the other core sent, if there is one.
    fn try_receive(&mut self) -> Option<u32>;

    /// Wake the other core if it is waiting for an event, without sending
    /// anything.
    fn wake(&mut self);
}

/// How long, in microseconds, the FIFO from core 1 must stay empty before
/// [`Launch::step`] sends a 0: far longer than core 1 takes to echo a word,
/// and short beside any deadline a launch has. It adds this much per 0 to a
/// launch: two of them when nothing has to start over.
pub const QUIET_US: u64 = 100;

/// The most words one [`Launch::step`] takes from the FIFO before a 0. The
/// RP2350's FIFOs hold a few words each (§3.1.5 says four, the `FIFO_ST`
/// description eight); a FIFO still not empty after this many means the
/// other core is still sending, and the drain goes on at the next step.
pub const DRAIN_MAX: usize = 16;

/// What [`Launch::step`] reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Not finished: call [`Launch::step`] again.
    Pending,
    /// Core 1 echoed the entry point, the last word: it has been launched.
    Launched,
}

/// The launch handshake, one bounded step at a time.
///
/// ```ignore
/// let mut launch = Launch::new(vector_table, stack_top, entry);
/// let start = timer.now();
/// while launch.step(&mut fifo, timer.now()) == Step::Pending {
///     if timer.elapsed_since(start) > timeout { return Err(NoAnswer) }
/// }
/// ```
#[derive(Debug)]
pub struct Launch {
    /// The six words, in order.
    words: [u32; 6],
    /// Index into `words` of the word being sent or echoed.
    next: usize,
    /// Whether `words[next]` has been sent and its echo is awaited.
    sent: bool,
    /// Before a 0: since when the FIFO from core 1 has read empty at every
    /// look, if it has since the last word taken from it.
    quiet_from: Option<u64>,
}

impl Launch {
    /// The handshake that starts core 1 at `entry` with its stack pointer at
    /// `stack_pointer` and `VTOR` at `vector_table`.
    ///
    /// `entry` is a code address as the core branches to it: on Arm, with
    /// bit 0 (the Thumb bit) set, as a Rust function pointer already has it.
    pub const fn new(vector_table: u32, stack_pointer: u32, entry: u32) -> Self {
        Self {
            words: [0, 0, 1, vector_table, stack_pointer, entry],
            next: 0,
            sent: false,
            quiet_from: None,
        }
    }

    /// One step at time `now`, in microseconds from any clock that counts
    /// up: drain before a 0, send the next word, or take its echo. Never
    /// blocks. Returns [`Step::Launched`] when the echo of the last word
    /// arrives, and again on any later call.
    pub fn step(&mut self, mailbox: &mut impl Mailbox, now: u64) -> Step {
        if self.next == self.words.len() {
            return Step::Launched;
        }
        let word = self.words[self.next];
        if self.sent {
            let Some(echo) = mailbox.try_receive() else {
                return Step::Pending;
            };
            self.sent = false;
            self.quiet_from = None;
            if echo == word {
                self.next += 1;
            } else {
                // Not the echo of what was sent: start over from the first
                // 0, as the datasheet's listing does.
                self.next = 0;
            }
            return if self.next == self.words.len() {
                Step::Launched
            } else {
                Step::Pending
            };
        }
        if word == 0 {
            // "always drain the READ FIFO (from core 1) before sending a 0",
            // then "execute a SEV as core 1 may be waiting for FIFO space";
            // and send only once it has stayed empty for QUIET_US.
            let mut empty = false;
            for _ in 0..DRAIN_MAX {
                if mailbox.try_receive().is_none() {
                    empty = true;
                    break;
                }
                self.quiet_from = None;
            }
            if !empty {
                return Step::Pending;
            }
            let Some(from) = self.quiet_from else {
                self.quiet_from = Some(now);
                mailbox.wake();
                return Step::Pending;
            };
            if now.wrapping_sub(from) < QUIET_US {
                return Step::Pending;
            }
        }
        if mailbox.try_send(word) {
            self.sent = true;
        }
        Step::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A word FIFO of `DEPTH` entries, as the RP2350's.
    struct Fifo<const DEPTH: usize> {
        words: [u32; DEPTH],
        len: usize,
    }

    impl<const DEPTH: usize> Fifo<DEPTH> {
        const fn new() -> Self {
            Self {
                words: [0; DEPTH],
                len: 0,
            }
        }
        fn push(&mut self, w: u32) -> bool {
            if self.len == DEPTH {
                return false;
            }
            self.words[self.len] = w;
            self.len += 1;
            true
        }
        fn pop(&mut self) -> Option<u32> {
            if self.len == 0 {
                return None;
            }
            let w = self.words[0];
            self.words.copy_within(1..self.len, 0);
            self.len -= 1;
            Some(w)
        }
    }

    /// Both FIFOs between the cores, and a recently reset core 1 waiting in
    /// the bootrom.
    ///
    /// The datasheet gives only core 0's side. The model of core 1 follows
    /// the same rules: it takes one word at a time, empties its own incoming
    /// FIFO when it takes a 0, echoes every word it takes, and starts over on
    /// any word out of sequence (a 0 counting as the first of a new
    /// sequence). It launches with the three words it took after `0, 0, 1`.
    /// It starts at the beginning of the sequence with nothing in its
    /// incoming FIFO, which the bootrom empties as core 1 starts; what it
    /// sent before (the SDK documents a 0 it sends once it has emptied its
    /// FIFO) can still be waiting for core 0, or still be on its way.
    struct Chip {
        /// Core 0 to core 1.
        to_core1: Fifo<4>,
        /// Core 1 to core 0.
        to_core0: Fifo<4>,
        /// Core 1's place in `0, 0, 1, a, b, c`.
        seq: usize,
        /// What core 1 took after `0, 0, 1`.
        taken: [u32; 3],
        /// An echo core 1 could not send yet, its FIFO full.
        echo: Option<u32>,
        /// What core 1 launched with, once it has.
        launched: Option<[u32; 3]>,
        /// Words core 0 sent, all told.
        sent: usize,
        /// Times core 0 woke core 1.
        woken: usize,
    }

    impl Chip {
        fn new() -> Self {
            Self {
                to_core1: Fifo::new(),
                to_core0: Fifo::new(),
                seq: 0,
                taken: [0; 3],
                echo: None,
                launched: None,
                sent: 0,
                woken: 0,
            }
        }

        /// Core 1 does one thing: sends a pending echo, or takes one word.
        fn core1(&mut self) {
            if self.launched.is_some() {
                return;
            }
            if let Some(e) = self.echo {
                if self.to_core0.push(e) {
                    self.echo = None;
                    if self.seq == 6 {
                        self.launched = Some(self.taken);
                    }
                }
                return;
            }
            let Some(w) = self.to_core1.pop() else {
                return;
            };
            if w == 0 {
                while self.to_core1.pop().is_some() {}
            }
            self.seq = match (self.seq, w) {
                (0, 0) | (2.., 0) => 1,
                (1, 0) => 2,
                (2, 1) => 3,
                (3..=5, w) => {
                    self.taken[self.seq - 3] = w;
                    self.seq + 1
                }
                _ => 0,
            };
            self.echo = Some(w);
        }
    }

    impl Mailbox for Chip {
        fn try_send(&mut self, word: u32) -> bool {
            let ok = self.to_core1.push(word);
            self.sent += ok as usize;
            ok
        }
        fn try_receive(&mut self) -> Option<u32> {
            self.to_core0.pop()
        }
        fn wake(&mut self) {
            self.woken += 1;
        }
    }

    const VTOR: u32 = 0x1000_0000;
    const SP: u32 = 0x2008_2000;
    const ENTRY: u32 = 0x1000_0201;

    /// Step core 0 and core 1 by `schedule` (true: core 0's turn) until core
    /// 0 reports [`Step::Launched`], for at most `turns` turns of 1 µs each:
    /// the turn it did, if it did.
    fn run(
        chip: &mut Chip,
        turns: usize,
        mut schedule: impl FnMut(usize) -> bool,
    ) -> Option<usize> {
        let mut launch = Launch::new(VTOR, SP, ENTRY);
        for t in 0..turns {
            if schedule(t) {
                if launch.step(chip, t as u64) == Step::Launched {
                    return Some(t);
                }
            } else {
                chip.core1();
            }
        }
        None
    }

    #[test]
    fn core1_starts_with_the_three_words() {
        let mut chip = Chip::new();
        let done = run(&mut chip, 1000, |t| t % 2 == 0).expect("launched");
        assert_eq!(chip.launched, Some([VTOR, SP, ENTRY]));
        // Each word once: nothing had to start over.
        assert_eq!(chip.sent, 6);
        // Once before each of the two zeros.
        assert_eq!(chip.woken, 2);
        // A quiet FIFO waited for before each of them, and no longer.
        assert!(done as u64 >= 2 * QUIET_US && done < 300, "{done}");
    }

    #[test]
    fn what_core1_sent_before_is_drained_not_taken_as_an_answer() {
        // Core 1's 0 from its start, and words an earlier attempt left.
        let mut chip = Chip::new();
        for w in [0, 1, VTOR, 0] {
            chip.to_core0.push(w);
        }
        assert!(run(&mut chip, 1000, |t| t % 2 == 0).is_some());
        assert_eq!(chip.launched, Some([VTOR, SP, ENTRY]));
        assert_eq!(chip.sent, 6);
    }

    #[test]
    fn a_late_word_from_core1_is_waited_out() {
        // Core 1's 0 from its start is still on its way when core 0 first
        // looks, and after that core 1 gets one turn in four. Drained only once, that 0
        // would arrive after the first 0 went out, be taken as its echo, and
        // leave core 0 one answer behind: here it would report a launch
        // while core 1 is still waiting.
        let mut chip = Chip::new();
        chip.echo = Some(0);
        assert!(run(&mut chip, 1000, |t| t == 0 || t % 4 != 0).is_some());
        assert_eq!(chip.launched, Some([VTOR, SP, ENTRY]));
        assert_eq!(chip.sent, 6);
    }

    #[test]
    fn a_wrong_echo_starts_the_sequence_again() {
        let mut chip = Chip::new();
        let mut launch = Launch::new(VTOR, SP, ENTRY);
        let mut now = 0;
        // Through `0, 0, 1`: core 1 answers each word as it is sent.
        while launch.next < 3 {
            assert_eq!(launch.step(&mut chip, now), Step::Pending);
            chip.core1();
            chip.core1();
            now += 1;
        }
        // The vector table goes out; a word that is not its echo comes back.
        assert_eq!(launch.step(&mut chip, now), Step::Pending);
        assert_eq!(chip.to_core1.pop(), Some(VTOR));
        chip.to_core0.push(0xdead_beef);
        assert_eq!(launch.step(&mut chip, now), Step::Pending);
        // Core 0 is back at the first 0: it drains, wakes core 1, and sends
        // the 0 once the FIFO has been quiet for QUIET_US.
        let woken = chip.woken;
        assert_eq!(launch.step(&mut chip, now), Step::Pending);
        assert_eq!(chip.woken, woken + 1);
        assert_eq!(launch.step(&mut chip, now + QUIET_US - 1), Step::Pending);
        assert_eq!(chip.to_core1.pop(), None);
        assert_eq!(launch.step(&mut chip, now + QUIET_US), Step::Pending);
        assert_eq!(chip.to_core1.pop(), Some(0));
        assert_eq!(chip.launched, None);
    }

    #[test]
    fn a_word_from_core1_restarts_the_quiet_wait() {
        let mut chip = Chip::new();
        let mut launch = Launch::new(VTOR, SP, ENTRY);
        assert_eq!(launch.step(&mut chip, 0), Step::Pending);
        // A word turns up half way through the wait: it is drained, and the
        // wait starts again from the next look that finds the FIFO empty.
        chip.to_core0.push(7);
        let half = QUIET_US / 2;
        assert_eq!(launch.step(&mut chip, half), Step::Pending);
        assert_eq!(launch.step(&mut chip, half + 1), Step::Pending);
        assert_eq!(launch.step(&mut chip, QUIET_US + 1), Step::Pending);
        assert_eq!(chip.sent, 0);
        assert_eq!(launch.step(&mut chip, half + 1 + QUIET_US), Step::Pending);
        assert_eq!(chip.sent, 1);
    }

    #[test]
    fn a_full_fifo_is_waited_out() {
        // The FIFO to core 1 refuses the first few words: each is sent again
        // at a later step, and nothing is skipped.
        struct Busy<'a>(&'a mut Chip, usize);
        impl Mailbox for Busy<'_> {
            fn try_send(&mut self, word: u32) -> bool {
                if self.1 > 0 {
                    self.1 -= 1;
                    return false;
                }
                self.0.try_send(word)
            }
            fn try_receive(&mut self) -> Option<u32> {
                self.0.try_receive()
            }
            fn wake(&mut self) {
                self.0.wake()
            }
        }
        let mut chip = Chip::new();
        let mut launch = Launch::new(VTOR, SP, ENTRY);
        let mut launched = false;
        for t in 0..1000 {
            if launch.step(&mut Busy(&mut chip, (t % 7 != 0) as usize), t) == Step::Launched {
                launched = true;
                break;
            }
            chip.core1();
        }
        assert!(launched);
        assert_eq!(chip.launched, Some([VTOR, SP, ENTRY]));
        assert_eq!(chip.sent, 6);
    }

    #[test]
    fn no_answer_is_never_a_launch() {
        // Core 1 never runs: core 0 stays pending, however long it steps,
        // and never sends past the first word.
        let mut chip = Chip::new();
        assert_eq!(run(&mut chip, 100_000, |_| true), None);
        assert_eq!(chip.sent, 1);
        assert_eq!(chip.to_core1.pop(), Some(0));
    }

    #[test]
    fn a_core1_that_keeps_sending_does_not_hold_up_a_step() {
        // An endless stream from core 1: each step still returns, after at
        // most DRAIN_MAX words, without sending.
        struct Flood(usize);
        impl Mailbox for Flood {
            fn try_send(&mut self, _: u32) -> bool {
                panic!("sent a 0 into a FIFO it never emptied")
            }
            fn try_receive(&mut self) -> Option<u32> {
                self.0 += 1;
                Some(5)
            }
            fn wake(&mut self) {}
        }
        let mut flood = Flood(0);
        let mut launch = Launch::new(VTOR, SP, ENTRY);
        for t in 0..3 {
            assert_eq!(launch.step(&mut flood, t * QUIET_US), Step::Pending);
        }
        assert_eq!(flood.0, 3 * DRAIN_MAX);
    }

    #[test]
    fn core1_launches_whatever_the_interleaving() {
        // Pseudo-random schedules, speeds of core 1 and words left waiting
        // for core 0, from a fixed seed so that a failure repeats.
        let mut seed: u32 = 0x1234_5678;
        let mut rand = move || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed
        };
        for _ in 0..2000 {
            let mut chip = Chip::new();
            for _ in 0..rand() % 5 {
                chip.to_core0.push(rand() % 3);
            }
            if rand() % 2 == 0 {
                chip.echo = Some(0);
            }
            // Core 1 gets one turn in 2 to 5, at random.
            let one_in = rand() % 4 + 2;
            let mut r = rand();
            let done = run(&mut chip, 50_000, |_| {
                r ^= r << 13;
                r ^= r >> 17;
                r ^= r << 5;
                r % one_in != 0
            });
            assert!(done.is_some(), "never launched");
            assert_eq!(chip.launched, Some([VTOR, SP, ENTRY]));
            assert_eq!(chip.sent, 6);
        }
    }
}
