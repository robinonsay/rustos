//! [`ArrayBroker`]: a [`Broker`] with a fixed-size registry.

use core::cell::RefCell;

use heapless::Vec;

use super::{Broker, Mid, Publisher, SbError, Subscriber};

/// A [`Broker`] that holds up to `S` subscribers. LibJuno's
/// `BROKER_ROOT_T<MsgT, PipeN, RegCapacity>` (`broker_api.hpp`) and
/// `JUNO_SB_BROKER_ROOT_T` (`broker_api.h`, `src/juno_broker.c`).
///
/// The registry is a [`heapless::Vec`] of `&'a dyn Subscriber<T>`, so
/// pipes of different capacities (`Pipe<T, 2>`, `Pipe<T, 16>`) share one
/// broker — LibJuno's C++ broker needs one pipe size per broker.
///
/// Not [`Sync`]: one broker per thread of execution (see the
/// [module documentation](super)).
///
/// Every subscriber must outlive the broker, not merely live as long:
/// declare pipes before the broker. A subscriber that itself borrows the
/// broker (a forwarder that republishes, say) cannot meet that on the
/// stack, because each would have to outlive the other; give both
/// `'static` storage instead.
pub struct ArrayBroker<'a, T, const S: usize> {
    /// The subscribers, in the order they subscribed
    /// (`tPipeRegistry[0..zRegistryLength]`).
    registry: RefCell<Vec<&'a dyn Subscriber<T>, S>>,
}

impl<'a, T, const S: usize> ArrayBroker<'a, T, S> {
    /// A broker with no subscribers. LibJuno's `BROKER_IMPL_T::New`.
    /// `S` must be at least 1; zero fails to compile.
    pub const fn new() -> Self {
        // `static_assert(RegCapacity > 0, ...)` in `BROKER_ROOT_T`.
        const { assert!(S > 0, "a broker must hold at least one subscriber") };
        Self {
            // Start with an empty registry.
            registry: RefCell::new(Vec::new()),
        }
    }

    /// Subscribers registered.
    ///
    /// Returns 0 if called from inside a delivery that is also
    /// subscribing; outside a publish it is exact.
    pub fn len(&self) -> usize {
        // A shared borrow; it can only fail while `subscribe` holds the
        // mutable one, which never runs user code.
        self.registry.try_borrow().map_or(0, |r| r.len())
    }

    /// `true` if nothing has subscribed.
    pub fn is_empty(&self) -> bool {
        // Empty is zero length.
        self.len() == 0
    }

    /// The most subscribers this broker holds: `S`.
    pub const fn capacity(&self) -> usize {
        // Fixed by the type.
        S
    }
}

impl<T, const S: usize> Default for ArrayBroker<'_, T, S> {
    fn default() -> Self {
        // The same as `new`.
        Self::new()
    }
}

impl<'a, T, const S: usize> Broker<'a, T> for ArrayBroker<'a, T, S> {
    fn subscribe(&self, sub: &'a dyn Subscriber<T>) -> Result<(), SbError> {
        // A publish holds a shared borrow while it delivers; a subscriber
        // that subscribes from inside `deliver` finds it here.
        let mut registry = self.registry.try_borrow_mut().map_err(|_| SbError::Busy)?;
        // The same object twice would receive every message twice. Compare
        // addresses only: two vtable pointers for one object can differ.
        if registry.iter().any(|s| core::ptr::addr_eq(*s, sub)) {
            return Err(SbError::AlreadySubscribed);
        }
        // Append, or report the registry full (`RegisterSubscriber` returns
        // `JUNO_STATUS_TABLE_FULL_ERROR`, `broker_api.hpp`). The registry is
        // unchanged on failure.
        registry.push(sub).map_err(|_| SbError::RegistryFull)
    }
}

impl<T, const S: usize> Publisher<T> for ArrayBroker<'_, T, S> {
    fn publish(&self, mid: Mid, msg: &T) -> Result<usize, SbError> {
        // A shared borrow: a subscriber may publish again from inside
        // `deliver` (a forwarder, say), but may not subscribe.
        let registry = self.registry.try_borrow().map_err(|_| SbError::Busy)?;
        // Count both outcomes for the report.
        let mut delivered = 0;
        let mut refused = 0;
        // Every subscriber of `mid`, in registration order, as LibJuno's
        // `Publish` iterates its registry (`broker_api.hpp`).
        for sub in registry.iter().filter(|s| s.mid() == mid) {
            // Offer the message. Unlike LibJuno, a refusal does not stop
            // the loop: the rest still get it (see the module docs).
            match sub.deliver(msg) {
                // Accepted.
                Ok(()) => delivered += 1,
                // Refused; the subscriber keeps its own record of why.
                Err(_) => refused += 1,
            }
        }
        // Clean only if nobody refused.
        if refused == 0 {
            Ok(delivered)
        } else {
            Err(SbError::Undelivered { delivered, refused })
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::error::{Error, ErrorKind};
    use crate::sb::Pipe;
    use core::cell::Cell;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Msg(u32);

    #[test]
    fn fans_out_to_matching_pipes_only() {
        let a: Pipe<Msg, 2> = Pipe::new(1);
        let b: Pipe<Msg, 8> = Pipe::new(1);
        let other: Pipe<Msg, 2> = Pipe::new(2);
        let bus: ArrayBroker<'_, Msg, 3> = ArrayBroker::new();
        bus.subscribe(&a).unwrap();
        bus.subscribe(&b).unwrap();
        bus.subscribe(&other).unwrap();
        assert_eq!(bus.publish(1, &Msg(7)), Ok(2));
        assert_eq!(a.pop(), Some(Msg(7)));
        assert_eq!(b.pop(), Some(Msg(7)));
        assert!(other.is_empty());
        assert_eq!(bus.publish(2, &Msg(8)), Ok(1));
        assert_eq!(other.pop(), Some(Msg(8)));
    }

    #[test]
    fn no_subscribers_is_not_an_error() {
        let bus: ArrayBroker<'_, Msg, 1> = ArrayBroker::new();
        assert_eq!(bus.publish(42, &Msg(0)), Ok(0));
    }

    #[test]
    fn a_full_registry_is_refused_and_unchanged() {
        let a: Pipe<Msg, 1> = Pipe::new(1);
        let b: Pipe<Msg, 1> = Pipe::new(1);
        let bus: ArrayBroker<'_, Msg, 1> = ArrayBroker::new();
        bus.subscribe(&a).unwrap();
        assert_eq!(bus.subscribe(&b), Err(SbError::RegistryFull));
        assert_eq!(bus.len(), 1);
        assert_eq!(bus.publish(1, &Msg(3)), Ok(1), "only the first pipe gets it");
        assert!(b.is_empty());
    }

    #[test]
    fn subscribing_twice_is_refused() {
        let a: Pipe<Msg, 4> = Pipe::new(1);
        let bus: ArrayBroker<'_, Msg, 4> = ArrayBroker::new();
        bus.subscribe(&a).unwrap();
        assert_eq!(bus.subscribe(&a), Err(SbError::AlreadySubscribed));
        assert_eq!(bus.publish(1, &Msg(1)), Ok(1));
        assert_eq!(a.len(), 1, "delivered once, not twice");
    }

    #[test]
    fn a_full_pipe_does_not_stop_the_fan_out() {
        let slow: Pipe<Msg, 1> = Pipe::new(1);
        let fast: Pipe<Msg, 4> = Pipe::new(1);
        let bus: ArrayBroker<'_, Msg, 2> = ArrayBroker::new();
        // The full pipe is first, so LibJuno would stop before `fast`.
        bus.subscribe(&slow).unwrap();
        bus.subscribe(&fast).unwrap();
        bus.publish(1, &Msg(1)).unwrap();
        let err = bus.publish(1, &Msg(2)).unwrap_err();
        assert_eq!(err, SbError::Undelivered { delivered: 1, refused: 1 });
        assert_eq!(err.kind(), ErrorKind::Overrun);
        assert_eq!(slow.take_dropped(), 1);
        assert_eq!(fast.pop(), Some(Msg(1)));
        assert_eq!(fast.pop(), Some(Msg(2)));
    }

    /// Tries to subscribe a second subscriber from inside its own delivery.
    struct Meddler<'a, 'b> {
        bus: &'b ArrayBroker<'a, Msg, 4>,
        other: &'a Pipe<Msg, 1>,
        result: Cell<Option<Result<(), SbError>>>,
    }

    impl Subscriber<Msg> for Meddler<'_, '_> {
        fn mid(&self) -> Mid {
            1
        }
        fn deliver(&self, _msg: &Msg) -> Result<(), SbError> {
            self.result.set(Some(self.bus.subscribe(self.other)));
            Ok(())
        }
    }

    #[test]
    fn subscribing_during_a_publish_is_busy() {
        // A subscriber that borrows its own broker needs both to outlive
        // the registry (`heapless::Vec` has a `Drop` impl, so the borrow
        // checker cannot let them die together); leaking gives `'static`.
        extern crate std;
        use std::boxed::Box;
        let other: &'static Pipe<Msg, 1> = Box::leak(Box::new(Pipe::new(1)));
        let bus: &'static ArrayBroker<'static, Msg, 4> = Box::leak(Box::new(ArrayBroker::new()));
        let meddler: &'static Meddler<'static, 'static> =
            Box::leak(Box::new(Meddler { bus, other, result: Cell::new(None) }));
        bus.subscribe(meddler).unwrap();
        assert_eq!(bus.publish(1, &Msg(0)), Ok(1));
        assert_eq!(meddler.result.get(), Some(Err(SbError::Busy)));
        assert_eq!(bus.len(), 1);
    }

    /// Publishes a counter value each step.
    struct Producer<'b, 'a> {
        bus: &'b ArrayBroker<'a, Msg, 4>,
        next: u32,
    }

    impl App for Producer<'_, '_> {
        type Error = SbError;
        fn on_start(&mut self) -> Result<(), SbError> {
            Ok(())
        }
        fn on_process(&mut self) -> Result<(), SbError> {
            self.next += 1;
            self.bus.publish(1, &Msg(self.next)).map(|_| ())
        }
        fn on_exit(&mut self) -> Result<(), SbError> {
            Ok(())
        }
    }

    /// Subscribes in `on_start`, then sums what arrives.
    struct Consumer<'b, 'a> {
        bus: &'b ArrayBroker<'a, Msg, 4>,
        pipe: &'a Pipe<Msg, 4>,
        sum: u32,
    }

    impl App for Consumer<'_, '_> {
        type Error = SbError;
        fn on_start(&mut self) -> Result<(), SbError> {
            self.bus.subscribe(self.pipe)
        }
        fn on_process(&mut self) -> Result<(), SbError> {
            while let Some(Msg(n)) = self.pipe.pop() {
                self.sum += n;
            }
            Ok(())
        }
        fn on_exit(&mut self) -> Result<(), SbError> {
            Ok(())
        }
    }

    #[test]
    fn apps_share_one_broker_through_shared_references() {
        // Pipes, then the broker, then the apps that borrow both.
        let pipe: Pipe<Msg, 4> = Pipe::new(1);
        let bus: ArrayBroker<'_, Msg, 4> = ArrayBroker::new();
        let mut producer = Producer { bus: &bus, next: 0 };
        let mut consumer = Consumer { bus: &bus, pipe: &pipe, sum: 0 };
        // Both hold `&bus` at once; the scheduler would hold `&mut` to each app.
        producer.on_start().unwrap();
        consumer.on_start().unwrap();
        for _ in 0..3 {
            producer.on_process().unwrap();
        }
        consumer.on_process().unwrap();
        assert_eq!(consumer.sum, 1 + 2 + 3);
        assert!(pipe.is_empty());
    }
}
