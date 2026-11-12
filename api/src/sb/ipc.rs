//! [`IpcApp`]: carries software-bus messages between two threads of
//! execution — the two cores of a dual-core chip, say — over a pair of
//! lock-free single-producer, single-consumer queues.
//!
//! A broker is not [`Sync`] (see the [module documentation](super)): each
//! thread has its own. To move messages between two of them, each thread
//! runs an `IpcApp`, the mirror image of the other's:
//!
//! ```text
//!   thread A                                         thread B
//!   broker A ──▶ pipes ──▶ IpcApp A ──▶ queue A→B ──▶ IpcApp B ──▶ broker B
//!   broker A ◀────────────── IpcApp A ◀── queue B→A ◀── IpcApp B ◀── pipes ◀── broker B
//! ```
//!
//! Each run, an `IpcApp`
//!
//! 1. drains its **outbound** pipes — one per message id the other side
//!    wants, subscribed to the local broker — into its queue to the other
//!    side, and
//! 2. publishes everything waiting in the queue **from** the other side on
//!    the local broker, under the id it was sent with.
//!
//! The queues are [`heapless::spsc`]: one side only enqueues and the other
//! only dequeues, through atomic loads and stores, so neither side ever
//! waits for the other. Messages are moved, not shared: once enqueued, a
//! message belongs to the receiving side.
//!
//! # When the other side falls behind
//!
//! An app never waits. When the queue to the other side is full, the
//! `IpcApp` stops draining its pipes, leaving messages there; the pipes are
//! the overflow buffer. When a pipe fills too, the bus drops new messages
//! for it and the pipe counts them, and the next run reports them as
//! [`IpcStats::pipe_drops`] and as an error to the scheduler.
//!
//! # When this side falls behind
//!
//! A thread that stalls — on a slow write, say — leaves the other side's
//! messages piling up in the queue, and its next run would publish them
//! all at once, more than its own pipes may hold.
//! [`with_max_inbound`](IpcApp::with_max_inbound) caps how many a run
//! publishes; the rest wait in the queue for the next run, so the backlog
//! reaches the local pipes at a rate their consumers can keep up with.
//!
//! # Loops
//!
//! A message id must cross in one direction only. If both sides forwarded
//! the same id, each would republish what the other sent and send it
//! back, forever. An `IpcApp` refuses — counts in [`IpcStats::looped`] and
//! does not publish — any message from the other side whose id it also
//! forwards outbound.

use heapless::spsc::{Consumer, Producer};

use super::{Mid, Pipe, Publisher};
use crate::app::App;
use crate::error::{Error, ErrorKind};

/// What an [`IpcApp`] has done, as running totals. All counts wrap.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IpcStats {
    /// Runs so far.
    pub runs: u32,
    /// Messages sent to the other side.
    pub sent: u32,
    /// Messages received from the other side and published locally.
    pub received: u32,
    /// Messages the bus dropped because an outbound pipe was full.
    pub pipe_drops: u32,
    /// Messages from the other side refused because their id also goes
    /// outbound (see [Loops](self#loops)).
    pub looped: u32,
    /// Messages from the other side that a local subscriber refused.
    pub undelivered: u32,
}

/// Why an [`IpcApp`] run reported a problem. The run itself always
/// completes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpcError {
    /// Outbound messages were dropped: the other side is not keeping up.
    Dropped {
        /// How many, this run.
        count: u32,
    },
    /// Messages from the other side carried an id this side also sends
    /// out, and were refused.
    Looped {
        /// How many, this run.
        count: u32,
    },
}

impl Error for IpcError {
    fn kind(&self) -> ErrorKind {
        match self {
            // A consumer did not keep up.
            IpcError::Dropped { .. } => ErrorKind::Overrun,
            // The two sides are configured inconsistently.
            IpcError::Looped { .. } => ErrorKind::InvalidInput,
        }
    }
}

/// Builds the message an [`IpcApp`] publishes its stats as.
pub type ReportFn<T> = fn(&IpcStats) -> (Mid, T);

/// One side of a two-way software-bus bridge. See the
/// [module documentation](self).
///
/// `T` is the bus's message type; `N` the depth of the outbound pipes.
pub struct IpcApp<'a, T, const N: usize> {
    /// One pipe per message id to send, subscribed to the local broker.
    outbound: &'a [&'a Pipe<T, N>],
    /// The queue to the other side.
    tx: Producer<'a, (Mid, T)>,
    /// The queue from the other side.
    rx: Consumer<'a, (Mid, T)>,
    /// The local broker.
    bus: &'a dyn Publisher<T>,
    /// Running totals.
    stats: IpcStats,
    /// Every this many runs, publish the stats as made by the function.
    report: Option<(u32, ReportFn<T>)>,
    /// The most messages from the other side one run publishes.
    max_inbound: usize,
}

impl<'a, T, const N: usize> IpcApp<'a, T, N> {
    /// Send what arrives on `outbound` through `tx`, and publish what
    /// arrives on `rx` on `bus`.
    pub fn new(
        outbound: &'a [&'a Pipe<T, N>],
        tx: Producer<'a, (Mid, T)>,
        rx: Consumer<'a, (Mid, T)>,
        bus: &'a dyn Publisher<T>,
    ) -> Self {
        Self { outbound, tx, rx, bus, stats: IpcStats::default(), report: None, max_inbound: usize::MAX }
    }

    /// Publish at most `max` messages from the other side per run, leaving
    /// the rest in the queue (see
    /// [When this side falls behind](self#when-this-side-falls-behind)).
    /// `max` of 0 is taken as 1.
    pub fn with_max_inbound(mut self, max: usize) -> Self {
        self.max_inbound = max.max(1);
        self
    }

    /// Also publish the stats on the local bus every `runs` runs, as the
    /// message `make` builds from them. `runs` of 0 is taken as 1.
    pub fn with_report(mut self, runs: u32, make: ReportFn<T>) -> Self {
        self.report = Some((runs.max(1), make));
        self
    }

    /// The running totals.
    pub const fn stats(&self) -> IpcStats {
        self.stats
    }
}

impl<'a, T: Clone, const N: usize> App for IpcApp<'a, T, N> {
    type Error = IpcError;

    fn on_process(&mut self) -> Result<(), IpcError> {
        self.stats.runs = self.stats.runs.wrapping_add(1);
        // Outbound: while the queue has room, move messages from the pipes.
        // When it is full, they wait in the pipes.
        let mut dropped = 0u32;
        for pipe in self.outbound {
            while self.tx.ready() {
                let Some(msg) = pipe.pop() else { break };
                // Room checked by `ready`; only this side enqueues.
                if self.tx.enqueue((pipe.mid(), msg)).is_ok() {
                    self.stats.sent = self.stats.sent.wrapping_add(1);
                }
            }
            dropped = dropped.wrapping_add(pipe.take_dropped());
        }
        self.stats.pipe_drops = self.stats.pipe_drops.wrapping_add(dropped);
        // Inbound: publish what the other side sent, up to the cap.
        let mut looped = 0u32;
        let mut taken = 0;
        while taken < self.max_inbound
            && let Some((mid, msg)) = self.rx.dequeue()
        {
            taken += 1;
            // An id that also goes outbound would come straight back.
            if self.outbound.iter().any(|p| p.mid() == mid) {
                looped = looped.wrapping_add(1);
                continue;
            }
            self.stats.received = self.stats.received.wrapping_add(1);
            if self.bus.publish(mid, &msg).is_err() {
                // The subscriber's own pipe counted the drop as well.
                self.stats.undelivered = self.stats.undelivered.wrapping_add(1);
            }
        }
        self.stats.looped = self.stats.looped.wrapping_add(looped);
        // Own stats, on the local bus.
        if let Some((every, make)) = self.report
            && self.stats.runs.is_multiple_of(every)
        {
            let (mid, msg) = make(&self.stats);
            let _ = self.bus.publish(mid, &msg);
        }
        // Dropped data first: it is the more serious.
        if dropped > 0 {
            Err(IpcError::Dropped { count: dropped })
        } else if looped > 0 {
            Err(IpcError::Looped { count: looped })
        } else {
            Ok(())
        }
    }

    fn on_exit(&mut self) -> Result<(), IpcError> {
        // Nothing held: undelivered messages stay in the queues and pipes.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use heapless::spsc::Queue;

    use super::*;
    use crate::sb::{ArrayBroker, Broker};

    /// Two message ids, one per direction.
    const A_TO_B: Mid = 1;
    const B_TO_A: Mid = 2;
    const STATUS: Mid = 3;

    #[test]
    fn carries_messages_both_ways() {
        let mut ab: Queue<(Mid, u32), 8> = Queue::new();
        let mut ba: Queue<(Mid, u32), 8> = Queue::new();
        let (ab_tx, ab_rx) = ab.split();
        let (ba_tx, ba_rx) = ba.split();
        // Side A: forwards A_TO_B, receives B_TO_A.
        let a_out: Pipe<u32, 4> = Pipe::new(A_TO_B);
        let a_in: Pipe<u32, 4> = Pipe::new(B_TO_A);
        let a_bus: ArrayBroker<'_, u32, 4> = ArrayBroker::new();
        a_bus.subscribe(&a_out).unwrap();
        a_bus.subscribe(&a_in).unwrap();
        // Side B: the mirror image.
        let b_out: Pipe<u32, 4> = Pipe::new(B_TO_A);
        let b_in: Pipe<u32, 4> = Pipe::new(A_TO_B);
        let b_bus: ArrayBroker<'_, u32, 4> = ArrayBroker::new();
        b_bus.subscribe(&b_out).unwrap();
        b_bus.subscribe(&b_in).unwrap();
        let a_pipes = [&a_out];
        let b_pipes = [&b_out];
        let mut a = IpcApp::new(&a_pipes, ab_tx, ba_rx, &a_bus);
        let mut b = IpcApp::new(&b_pipes, ba_tx, ab_rx, &b_bus);
        // Each side publishes locally; one run each carries it over.
        a_bus.publish(A_TO_B, &10).unwrap();
        b_bus.publish(B_TO_A, &20).unwrap();
        assert_eq!(a.on_process(), Ok(()));
        assert_eq!(b.on_process(), Ok(()));
        assert_eq!(b_in.pop(), Some(10));
        // B's message reaches A on A's next run.
        assert_eq!(a.on_process(), Ok(()));
        assert_eq!(a_in.pop(), Some(20));
        assert_eq!((a.stats().sent, a.stats().received), (1, 1));
        assert_eq!((b.stats().sent, b.stats().received), (1, 1));
    }

    #[test]
    fn a_full_queue_leaves_messages_in_the_pipe_then_reports_drops() {
        // Capacity N − 1 = 1.
        let mut q: Queue<(Mid, u32), 2> = Queue::new();
        let mut back: Queue<(Mid, u32), 2> = Queue::new();
        let (tx, mut far_rx) = q.split();
        let (_, rx) = back.split();
        let out: Pipe<u32, 2> = Pipe::new(A_TO_B);
        let bus: ArrayBroker<'_, u32, 1> = ArrayBroker::new();
        bus.subscribe(&out).unwrap();
        let pipes = [&out];
        let mut app = IpcApp::new(&pipes, tx, rx, &bus);
        // Two messages: one fits the queue, one waits in the pipe.
        bus.publish(A_TO_B, &1).unwrap();
        bus.publish(A_TO_B, &2).unwrap();
        assert_eq!(app.on_process(), Ok(()));
        assert_eq!(out.len(), 1);
        // Two more: the pipe holds one of them; the other is dropped.
        bus.publish(A_TO_B, &3).unwrap();
        assert!(bus.publish(A_TO_B, &4).is_err());
        assert_eq!(app.on_process(), Err(IpcError::Dropped { count: 1 }));
        assert_eq!(app.stats().pipe_drops, 1);
        // The far side catches up; everything not dropped arrives in order.
        let mut got = heapless::Vec::<u32, 4>::new();
        for _ in 0..3 {
            while let Some((_, m)) = far_rx.dequeue() {
                got.push(m).unwrap();
            }
            let _ = app.on_process();
        }
        assert_eq!(&got[..], &[1, 2, 3]);
    }

    #[test]
    fn refuses_an_id_that_would_loop() {
        let mut out_q: Queue<(Mid, u32), 4> = Queue::new();
        let mut in_q: Queue<(Mid, u32), 4> = Queue::new();
        let (tx, _far_rx) = out_q.split();
        let (mut far_tx, rx) = in_q.split();
        let out: Pipe<u32, 2> = Pipe::new(A_TO_B);
        let bus: ArrayBroker<'_, u32, 1> = ArrayBroker::new();
        bus.subscribe(&out).unwrap();
        let pipes = [&out];
        let mut app = IpcApp::new(&pipes, tx, rx, &bus);
        // The other side sends an id this side also sends out.
        far_tx.enqueue((A_TO_B, 5)).unwrap();
        assert_eq!(app.on_process(), Err(IpcError::Looped { count: 1 }));
        // It was not republished, so it is not on its way back.
        assert!(out.is_empty());
        assert_eq!(app.stats().looped, 1);
    }

    #[test]
    fn reports_its_stats_on_the_local_bus() {
        let mut out_q: Queue<(Mid, u32), 4> = Queue::new();
        let mut in_q: Queue<(Mid, u32), 4> = Queue::new();
        let (tx, _) = out_q.split();
        let (_, rx) = in_q.split();
        let status: Pipe<u32, 2> = Pipe::new(STATUS);
        let bus: ArrayBroker<'_, u32, 1> = ArrayBroker::new();
        bus.subscribe(&status).unwrap();
        let pipes: [&Pipe<u32, 2>; 0] = [];
        let mut app = IpcApp::new(&pipes, tx, rx, &bus).with_report(2, |s| (STATUS, s.runs));
        app.on_process().unwrap();
        assert!(status.is_empty());
        app.on_process().unwrap();
        assert_eq!(status.pop(), Some(2));
    }

    #[test]
    fn a_capped_side_publishes_a_backlog_a_few_at_a_time() {
        let mut q: Queue<(Mid, u32), 8> = Queue::new();
        let mut back: Queue<(Mid, u32), 2> = Queue::new();
        let (mut far_tx, rx) = q.split();
        let (tx, _) = back.split();
        let inbox: Pipe<u32, 2> = Pipe::new(B_TO_A);
        let bus: ArrayBroker<'_, u32, 1> = ArrayBroker::new();
        bus.subscribe(&inbox).unwrap();
        let pipes: [&Pipe<u32, 2>; 0] = [];
        let mut app = IpcApp::new(&pipes, tx, rx, &bus).with_max_inbound(2);
        // Five waiting; a pipe of two, emptied after every run, loses none.
        for m in 0..5 {
            far_tx.enqueue((B_TO_A, m)).unwrap();
        }
        let mut got = heapless::Vec::<u32, 8>::new();
        for _ in 0..3 {
            app.on_process().unwrap();
            while let Some(m) = inbox.pop() {
                got.push(m).unwrap();
            }
        }
        assert_eq!(&got[..], &[0, 1, 2, 3, 4]);
        assert_eq!(inbox.take_dropped(), 0);
    }
}
