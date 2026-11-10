//! The software bus: publish/subscribe message passing between apps.
//!
//! An app that produces data *publishes* it under a message id ([`Mid`]);
//! every app that wants that data *subscribes* a [`Pipe`] — a fixed-size
//! FIFO tagged with one id — and drains it in its own
//! [`on_process`](crate::app::App::on_process). The publisher never names
//! its consumers, and a consumer never names its producer: adding a second
//! consumer of a message, or replacing its producer, touches neither side.
//!
//! This is the Rust form of LibJuno's software bus
//! (`juno/sb/broker_api.h`, `juno/sb/broker_api.hpp`, `src/juno_broker.c`):
//!
//! | rustos | LibJuno | What |
//! |--------|---------|------|
//! | [`Mid`] | `JUNO_SB_MID_T` | message id, `u32` |
//! | [`Pipe`] | `JUNO_SB_PIPE_T` / `PIPE_ROOT_T` | a FIFO tagged with one [`Mid`] |
//! | [`Subscriber`] | (the pipe's `Enqueue`) | anything a broker can deliver to |
//! | [`Broker`], [`Publisher`] | `JUNO_SB_BROKER_API_T` | `RegisterSubscriber`; `Publish` |
//! | [`ArrayBroker`] | `JUNO_SB_BROKER_ROOT_T` / `BROKER_ROOT_T` | a broker with a fixed-size registry |
//!
//! Storage is fixed at compile time, with no heap: a pipe's queue is a
//! [`heapless::Deque`] and a broker's registry a [`heapless::Vec`], each
//! sized by a const generic. A full pipe or registry is an error, never an
//! allocation.
//!
//! # One broker per thread of execution
//!
//! Like LibJuno's (`broker_api.h`, file comment), a broker is deliberately
//! *not* thread-safe: pipes use [`RefCell`](core::cell::RefCell), so
//! neither [`Pipe`] nor [`ArrayBroker`] is [`Sync`], and the compiler
//! refuses to share them with an interrupt handler or the other core.
//! Each thread of execution — the main loop of one core, say — owns its
//! own broker. Moving a message between threads is a job for whatever IPC
//! the platform has (a hardware FIFO, a DMA ring, an RTOS queue): receive
//! it on the destination thread, then publish it there. [`IpcApp`] does
//! exactly that over a pair of lock-free queues.
//!
//! # Differences from LibJuno
//!
//! * **A full pipe does not stop the fan-out.** LibJuno's `Publish` returns
//!   at the first pipe that will not take the message, so later subscribers
//!   never see it. Here every matching subscriber is offered the message,
//!   and [`Publisher::publish`] reports how many refused
//!   ([`SbError::Undelivered`]): one slow consumer cannot starve the rest,
//!   the same rule the scheduler applies to failing apps.
//! * **A pipe counts what it dropped** ([`Pipe::take_dropped`]), so the
//!   *consumer* learns it fell behind, not only the publisher.
//! * **Subscribing the same subscriber twice is an error**
//!   ([`SbError::AlreadySubscribed`]); LibJuno would deliver every message
//!   to it twice.
//! * **Brokers take `&self`.** Every app that publishes holds a shared
//!   reference to the one broker, so publishing and subscribing work
//!   through `&self`, with the registry behind a `RefCell`. That lets an
//!   app subscribe in its [`on_start`](crate::app::App::on_start), as
//!   LibJuno's examples do (`examples/udp-threads`, `MonitorApp`).
//!
//! # Example
//!
//! The broker carries one message type, `T`. A system with several kinds
//! of message uses an `enum` with one variant per kind, and gives each its
//! own [`Mid`]:
//!
//! ```ignore
//! use api::app::App;
//! use api::error::ErrorKind;
//! use api::sb::{ArrayBroker, Broker, Mid, Pipe, Publisher};
//!
//! const ENGINE_CMD: Mid = 0x1800;
//!
//! #[derive(Clone, Debug)]
//! enum Msg { EngineCmd { rpm: f32 } }
//!
//! struct Engine<'a> { cmds: &'a Pipe<Msg, 4>, rpm: f32 }
//!
//! impl App for Engine<'_> {
//!     type Error = ErrorKind;
//!     fn on_start(&mut self) -> Result<(), ErrorKind> { Ok(()) }
//!     fn on_process(&mut self) -> Result<(), ErrorKind> {
//!         // Take every command that arrived since last time.
//!         while let Some(Msg::EngineCmd { rpm }) = self.cmds.pop() {
//!             self.rpm = rpm;
//!         }
//!         Ok(())
//!     }
//!     fn on_exit(&mut self) -> Result<(), ErrorKind> { Ok(()) }
//! }
//!
//! fn main() -> ! {
//!     // Pipes first: the broker holds references to them.
//!     let cmds: Pipe<Msg, 4> = Pipe::new(ENGINE_CMD);
//!     let bus: ArrayBroker<'_, Msg, 8> = ArrayBroker::new();
//!     bus.subscribe(&cmds).unwrap();
//!     let mut engine = Engine { cmds: &cmds, rpm: 0.0 };
//!     // ... a commanding app holds `&bus` and calls
//!     // `bus.publish(ENGINE_CMD, &Msg::EngineCmd { rpm: 2500.0 })` ...
//!     loop { /* run the scheduler */ }
//! }
//! ```

pub mod broker;
pub mod ipc;
pub mod pipe;

pub use broker::ArrayBroker;
pub use ipc::{IpcApp, IpcError, IpcStats, ReportFn};
pub use pipe::Pipe;

use crate::error::{Error, ErrorKind};

/// A message id: which kind of message this is, and so which pipes it goes
/// to. LibJuno's `JUNO_SB_MID_T` (`broker_api.h`), a `uint32_t`.
///
/// Ids are the system's to assign; the only rule is one id per kind of
/// message.
pub type Mid = u32;

/// Something a [`Broker`] can deliver messages to: in practice a [`Pipe`],
/// but any type can subscribe — a counter, a forwarder to another thread's
/// IPC, a test probe.
///
/// Both methods take `&self` because the broker holds a shared reference
/// to each subscriber while its owner still reads from it; a subscriber
/// that changes state on delivery uses interior mutability, as [`Pipe`]
/// does.
pub trait Subscriber<T> {
    /// The message id this subscriber receives.
    fn mid(&self) -> Mid;

    /// Accept one published message, or refuse it (a full pipe returns
    /// [`SbError::PipeFull`]). Called by the broker, once per publish of
    /// this subscriber's id.
    fn deliver(&self, msg: &T) -> Result<(), SbError>;
}

/// The publishing half of a broker: what an app that only produces
/// messages needs. LibJuno's `Publish` (`broker_api.h`).
///
/// Split from [`Broker`] because it needs no subscriber lifetime: an app
/// holds a `&dyn Publisher<T>` however long the broker's subscribers live,
/// where a `&'a dyn Broker<'a, T>` would tie its borrow to them, which a
/// broker with a destructor cannot satisfy.
pub trait Publisher<T> {
    /// Offer `msg` to every subscriber of `mid`, in the order they
    /// subscribed. Returns how many accepted it; no subscribers is
    /// `Ok(0)`, not an error, since a message nobody wants yet is normal
    /// while a system starts.
    ///
    /// # Errors
    ///
    /// [`SbError::Undelivered`] if any subscriber refused the message. All
    /// the others still got it.
    fn publish(&self, mid: Mid, msg: &T) -> Result<usize, SbError>;
}

/// A publish/subscribe broker: LibJuno's `JUNO_SB_BROKER_API_T`
/// (`broker_api.h`), carrying messages of type `T`. Publishing is the
/// [`Publisher`] supertrait.
///
/// `'a` is how long the subscribers it holds must live; it must not
/// outlive them, which the borrow checker enforces where LibJuno's
/// "the pipe must remain valid" (`broker_api.hpp`, `RegisterSubscriber`)
/// was a comment.
pub trait Broker<'a, T>: Publisher<T> {
    /// Register `sub` to receive every later message published with its
    /// [`mid`](Subscriber::mid). LibJuno's `RegisterSubscriber`.
    ///
    /// # Errors
    ///
    /// * [`SbError::RegistryFull`] if no more subscribers fit.
    /// * [`SbError::AlreadySubscribed`] if `sub` is already registered.
    /// * [`SbError::Busy`] if called from inside a delivery of this
    ///   broker's own [`publish`](Publisher::publish).
    fn subscribe(&self, sub: &'a dyn Subscriber<T>) -> Result<(), SbError>;
}

/// What can go wrong on the software bus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SbError {
    /// The broker's registry holds as many subscribers as it can.
    /// LibJuno's `JUNO_STATUS_TABLE_FULL_ERROR` (`broker_api.hpp`).
    RegistryFull,
    /// This subscriber is already registered with the broker.
    AlreadySubscribed,
    /// The registry could not be changed because the broker is in the
    /// middle of a publish: a subscriber tried to subscribe from inside
    /// [`Subscriber::deliver`].
    Busy,
    /// A pipe was full and refused the message. The pipe counted it in
    /// [`Pipe::take_dropped`].
    PipeFull,
    /// A publish reached some subscribers but not all.
    Undelivered {
        /// Subscribers of the id that accepted the message.
        delivered: usize,
        /// Subscribers of the id that refused it.
        refused: usize,
    },
}

impl Error for SbError {
    fn kind(&self) -> ErrorKind {
        // Map each variant onto the shared vocabulary.
        match self {
            // A fixed-capacity table had no room.
            SbError::RegistryFull => ErrorKind::Full,
            // The argument repeats an earlier one; repeating it again fails again.
            SbError::AlreadySubscribed => ErrorKind::InvalidInput,
            // The registry is in use by the publish in progress.
            SbError::Busy => ErrorKind::Busy,
            // A fixed-capacity queue had no room.
            SbError::PipeFull => ErrorKind::Full,
            // Messages were lost because a consumer did not keep up.
            SbError::Undelivered { .. } => ErrorKind::Overrun,
        }
    }
}
