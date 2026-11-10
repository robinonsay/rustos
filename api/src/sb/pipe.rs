//! [`Pipe`]: a fixed-size FIFO of messages for one [`Mid`].

use core::cell::{Cell, RefCell};

use heapless::Deque;

use super::{Mid, SbError, Subscriber};

/// A subscriber's queue of messages for one [`Mid`], holding up to `N`.
/// LibJuno's `PIPE_ROOT_T<MsgT, N>` (`broker_api.hpp`): a circular FIFO
/// tagged with the message id it subscribes to.
///
/// The queue is a [`heapless::Deque`], so the storage is inline — a
/// `Pipe<T, N>` is `N` messages plus a few words, wherever it lives. It
/// sits behind a [`RefCell`] so that the broker, holding `&Pipe`, can push
/// while the app that owns it, holding another `&Pipe`, pops.
///
/// No mutable borrow of the queue is ever held while code outside this
/// type runs (a delivered message's `Clone` runs before the queue is
/// borrowed, and a refused one is dropped after), so the `RefCell` is
/// never found mutably borrowed. The one way to make these methods panic
/// is a message type whose `Clone` reaches back into the very pipe being
/// [`peek`](Self::peek)ed and pops from it.
///
/// When the pipe is full, a delivered message is refused and counted
/// ([`take_dropped`](Self::take_dropped)); the messages already queued are
/// kept. That is LibJuno's policy: the oldest unread data is what the
/// consumer is about to act on.
pub struct Pipe<T, const N: usize> {
    /// The message id this pipe subscribes to (`PIPE_ROOT_T::iMsgId`).
    mid: Mid,
    /// The queued messages, oldest at the front
    /// (`PIPE_ROOT_T::tBuf`, `iStartIndex`, `zLength`).
    queue: RefCell<Deque<T, N>>,
    /// Deliveries refused because the queue was full, since the last
    /// [`take_dropped`](Self::take_dropped). Saturates at `u32::MAX`.
    dropped: Cell<u32>,
}

impl<T, const N: usize> Pipe<T, N> {
    /// An empty pipe for message id `mid`. LibJuno's `PIPE_IMPL_T::New`
    /// (`broker_api.hpp`). `N` must be at least 1; zero fails to compile.
    pub const fn new(mid: Mid) -> Self {
        // A pipe that can hold nothing would refuse every message
        // (`static_assert(N > 0, ...)` in `PIPE_ROOT_T`).
        const { assert!(N > 0, "a pipe must hold at least one message") };
        Self {
            // Remember which id to receive.
            mid,
            // Start empty.
            queue: RefCell::new(Deque::new()),
            // Nothing dropped yet.
            dropped: Cell::new(0),
        }
    }

    /// The message id this pipe receives.
    pub const fn mid(&self) -> Mid {
        // As set at construction.
        self.mid
    }

    /// Take the oldest message, or `None` if the pipe is empty. LibJuno's
    /// `Dequeue`, which reports empty as `JUNO_STATUS_OOB_ERROR`.
    pub fn pop(&self) -> Option<T> {
        // Borrow just for the removal; no other code runs meanwhile.
        self.queue.borrow_mut().pop_front()
    }

    /// Append `msg` directly, without going through a broker; hand it back
    /// if the pipe is full. Not counted as dropped: the caller still has
    /// the message.
    pub fn push(&self, msg: T) -> Result<(), T> {
        // Borrow just for the insertion; `Deque::push_back` returns the
        // item when full.
        self.queue.borrow_mut().push_back(msg)
    }

    /// Messages waiting.
    pub fn len(&self) -> usize {
        // Borrow just long enough to read the length.
        self.queue.borrow().len()
    }

    /// `true` if no messages are waiting.
    pub fn is_empty(&self) -> bool {
        // Empty is zero length.
        self.len() == 0
    }

    /// `true` if the next delivery would be refused.
    pub fn is_full(&self) -> bool {
        // Borrow just long enough to compare length and capacity.
        self.queue.borrow().is_full()
    }

    /// The most messages the pipe holds: `N`.
    pub const fn capacity(&self) -> usize {
        // Fixed by the type.
        N
    }

    /// Discard every waiting message. The drop count is unchanged.
    pub fn clear(&self) {
        // Take the queue's contents out under the borrow, then let them be
        // dropped after it ends, so a message's `Drop` never runs while the
        // queue is borrowed.
        let old = self.queue.replace(Deque::new());
        // Messages in `old` are dropped here.
        drop(old);
    }

    /// How many deliveries were refused because the pipe was full since
    /// the last call, and reset the count. A consumer that sees a non-zero
    /// count knows it missed messages.
    pub fn take_dropped(&self) -> u32 {
        // Read and zero in one step.
        self.dropped.replace(0)
    }
}

impl<T: Clone, const N: usize> Pipe<T, N> {
    /// A copy of the oldest message, leaving it queued; `None` if empty.
    /// LibJuno's `Peek`.
    pub fn peek(&self) -> Option<T> {
        // Clone under a shared borrow. `Clone` is user code, but a shared
        // borrow only conflicts with `borrow_mut`, and a `Clone` that
        // reached back into this pipe to pop would be a bug in that `Clone`.
        // `try_borrow` keeps even that case from panicking: it sees an
        // empty pipe.
        let queue = self.queue.try_borrow().ok()?;
        // Copy the front message, if any.
        queue.front().cloned()
    }
}

impl<T: Clone, const N: usize> Subscriber<T> for Pipe<T, N> {
    fn mid(&self) -> Mid {
        // The id given at construction.
        self.mid
    }

    fn deliver(&self, msg: &T) -> Result<(), SbError> {
        // Copy the message before borrowing the queue, so `Clone` (user
        // code) never runs while the queue is borrowed. LibJuno's pipe
        // copies too: `Enqueue` takes `const MsgT &` and stores a copy.
        let copy = msg.clone();
        // Try to queue it; the borrow ends with this statement, so a
        // refused message is dropped outside it.
        let result = self.queue.borrow_mut().push_back(copy);
        match result {
            // Queued.
            Ok(()) => Ok(()),
            // Full: count the loss for the consumer and tell the broker.
            Err(_refused) => {
                // Saturate rather than wrap, so a huge count never reads as small.
                self.dropped.set(self.dropped.get().saturating_add(1));
                Err(SbError::PipeFull)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pops_in_the_order_delivered() {
        let pipe: Pipe<u32, 3> = Pipe::new(7);
        assert_eq!(Subscriber::mid(&pipe), 7);
        assert!(pipe.is_empty());
        pipe.deliver(&1).unwrap();
        pipe.deliver(&2).unwrap();
        assert_eq!(pipe.len(), 2);
        assert_eq!(pipe.peek(), Some(1));
        assert_eq!(pipe.pop(), Some(1));
        assert_eq!(pipe.pop(), Some(2));
        assert_eq!(pipe.pop(), None);
        assert_eq!(pipe.peek(), None);
    }

    #[test]
    fn a_full_pipe_refuses_and_counts_and_keeps_the_oldest() {
        let pipe: Pipe<u32, 2> = Pipe::new(1);
        pipe.deliver(&10).unwrap();
        pipe.deliver(&11).unwrap();
        assert!(pipe.is_full());
        assert_eq!(pipe.deliver(&12), Err(SbError::PipeFull));
        assert_eq!(pipe.deliver(&13), Err(SbError::PipeFull));
        assert_eq!(pipe.take_dropped(), 2);
        assert_eq!(pipe.take_dropped(), 0, "taking the count resets it");
        assert_eq!(pipe.pop(), Some(10));
        assert_eq!(pipe.pop(), Some(11));
    }

    #[test]
    fn push_hands_back_what_does_not_fit() {
        let pipe: Pipe<u32, 1> = Pipe::new(1);
        assert_eq!(pipe.push(5), Ok(()));
        assert_eq!(pipe.push(6), Err(6));
        assert_eq!(pipe.take_dropped(), 0, "push is not a lost delivery");
    }

    #[test]
    fn clear_empties_the_pipe() {
        let pipe: Pipe<u32, 4> = Pipe::new(1);
        pipe.deliver(&1).unwrap();
        pipe.deliver(&2).unwrap();
        pipe.clear();
        assert!(pipe.is_empty());
        assert_eq!(pipe.capacity(), 4);
        pipe.deliver(&3).unwrap();
        assert_eq!(pipe.pop(), Some(3));
    }
}
