//! A fixed-capacity byte FIFO.
//!
//! The USB serial driver buffers the byte stream in two of these: one for
//! bytes received from the host and not yet read by the application, one
//! for bytes the application wrote that the host has not yet collected.
//! Fixed capacity is deliberate: with no heap, every buffer's size is
//! decided at compile time, and a full buffer is reported to the caller
//! (as a short count) rather than growing.

/// A first-in first-out queue of up to `N` bytes, stored inline.
///
/// Internally a circular buffer: `head` is the index of the oldest byte and
/// `len` how many bytes are stored, so the newest byte sits at
/// `(head + len - 1) % N`. Keeping a length rather than a tail index means
/// "full" (`len == N`) and "empty" (`len == 0`) are distinct without
/// sacrificing a slot.
///
/// Not synchronised: it is meant to be owned by one driver and touched from
/// one execution context (here, the polling loop).
#[derive(Clone, Debug)]
pub struct ByteRing<const N: usize> {
    buf: [u8; N],
    head: usize,
    len: usize,
}

impl<const N: usize> ByteRing<N> {
    /// An empty ring.
    pub const fn new() -> Self {
        Self { buf: [0; N], head: 0, len: 0 }
    }

    /// Total capacity, `N`.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Bytes currently stored.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// True if no bytes are stored.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Room left, in bytes.
    pub const fn free(&self) -> usize {
        N - self.len
    }

    /// Discard everything.
    pub fn clear(&mut self) {
        self.head = 0;
        self.len = 0;
    }

    /// Append one byte. Returns `false`, storing nothing, if full.
    pub fn push(&mut self, b: u8) -> bool {
        if self.len == N {
            return false;
        }
        let tail = (self.head + self.len) % N;
        self.buf[tail] = b;
        self.len += 1;
        true
    }

    /// Remove and return the oldest byte, or `None` if empty.
    pub fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }
        let b = self.buf[self.head];
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Some(b)
    }

    /// Append as much of `data` as fits; returns how many bytes were taken.
    pub fn push_slice(&mut self, data: &[u8]) -> usize {
        let mut n = 0;
        for &b in data {
            if !self.push(b) {
                break;
            }
            n += 1;
        }
        n
    }

    /// Move up to `out.len()` of the oldest bytes into `out`; returns how
    /// many were moved.
    pub fn pop_slice(&mut self, out: &mut [u8]) -> usize {
        let mut n = 0;
        for slot in out.iter_mut() {
            match self.pop() {
                Some(b) => *slot = b,
                None => break,
            }
            n += 1;
        }
        n
    }
}

impl<const N: usize> Default for ByteRing<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_order_and_wraparound() {
        let mut r: ByteRing<4> = ByteRing::new();
        assert!(r.is_empty());
        assert_eq!(r.push_slice(&[1, 2, 3]), 3);
        assert_eq!(r.pop(), Some(1));
        assert_eq!(r.pop(), Some(2));
        // Tail wraps past the end of the array here.
        assert_eq!(r.push_slice(&[4, 5, 6, 7]), 3);
        assert_eq!(r.len(), 4);
        assert_eq!(r.free(), 0);
        assert!(!r.push(9));
        let mut out = [0u8; 8];
        assert_eq!(r.pop_slice(&mut out), 4);
        assert_eq!(&out[..4], &[3, 4, 5, 6]);
        assert_eq!(r.pop(), None);
    }

    #[test]
    fn clear_and_capacity() {
        let mut r: ByteRing<8> = ByteRing::default();
        assert_eq!(r.capacity(), 8);
        r.push_slice(b"abc");
        r.clear();
        assert!(r.is_empty());
        assert_eq!(r.free(), 8);
        assert_eq!(r.pop_slice(&mut [0u8; 2]), 0);
    }
}
