/// Declares the single error type a peripheral reports.
///
/// `Error` below is an **associated type**: a type member of a trait, for
/// which each implementing type chooses one concrete type. Every fallible
/// trait in this crate ([`Write`], [`Read`], and the factory traits in
/// [`crate::gpio`]) takes `ErrorType` as a **supertrait**: writing
/// `trait Write<T>: ErrorType` means a type may implement `Write` only if it
/// also implements `ErrorType`, which is why `Self::Error` is nameable
/// inside `Write`. A type therefore names its error **once**, and all of its
/// operations agree on it.
///
/// # Why this is a separate trait
///
/// The obvious alternative is to give each trait its own associated error:
///
/// ```ignore
/// pub trait Write<T> { type Error; fn write(&mut self, v: T) -> Result<(), Self::Error>; }
/// pub trait Read<T>  { type Error; fn read(&mut self) -> Result<T, Self::Error>; }
/// ```
///
/// That compiles, and it is what `embedded-hal` 0.2 did. It has two problems
/// that only appear once you write generic code against it.
///
/// First, the errors are unrelated. A function generic over
/// `T: Write<bool> + Read<bool>` sees two independent associated types and
/// cannot convert or unify them, so every caller ends up writing
/// `where T::Error: From<...>` bounds by hand.
///
/// Second, naming the error becomes ambiguous. With two `Error` types in
/// scope, `T::Error` is a compile error — you must write
/// `<T as Write<bool>>::Error`, and that spelling leaks into every signature.
///
/// Hoisting the error into one supertrait fixes both: `T::Error` is
/// unambiguous, and there is exactly one error type per peripheral to handle.
/// `embedded-hal` 1.0 made this same change for the same reasons.
///
/// # The `Debug` bound
///
/// Every fallible operation here returns [`Result`], an enum that is either
/// `Ok(value)` or `Err(error)`; [`Result::unwrap`] and [`Result::expect`]
/// return the value, or halt the program on `Err` — and to print the error
/// as they halt, they require it to be `Debug`-formattable. So does a logger
/// in the style of defmt or RTT, which streams log records to an attached
/// debug probe. That is all the bound is for. It is deliberately weaker than
/// `core::error::Error`, which would drag in `Display` and force every
/// implementor to write human-readable strings into flash, the chip's
/// nonvolatile program memory.
pub trait ErrorType {
    /// The error reported by every fallible operation on this type.
    ///
    /// Use [`core::convert::Infallible`] when an operation genuinely cannot
    /// fail — it is an empty enum, so `Result<T, Infallible>` is the same size
    /// as `T` and the error branch is eliminated entirely. Do not reach for it
    /// merely because failure is *unlikely*: an implementation with
    /// `Error = Infallible` has no way to construct an error value, so a real
    /// failure must panic, hang, or silently produce wrong data — the caller
    /// loses any way to diagnose it at the point where it would have
    /// mattered.
    type Error: core::fmt::Debug;
}

/// Sends a value of type `T` to a peripheral.
///
/// Generic over the value rather than the peripheral, so a single type can be
/// written in several representations. A GPIO pin implements `Write<bool>`; a
/// UART (a peripheral that sends and receives bytes serially over a pair of
/// wires) would implement `Write<u8>`; a DAC (digital-to-analog converter)
/// might implement both `Write<u16>` and `Write<f32>`.
///
/// Implementations should be **idempotent and non-blocking** where the
/// hardware allows. Writing the same value twice must be safe, and `write`
/// should not spin waiting on the peripheral unless that is inherent to the
/// operation.
pub trait Write<T>: ErrorType {
    /// Write `value` to the peripheral.
    ///
    /// Takes `&mut self` because a write is a state change. Two kinds of
    /// reference exist in Rust: `&self` is a *shared* reference, any number
    /// of which may point at a value at once; `&mut self` is an *exclusive*
    /// reference — the compiler (specifically the borrow checker) proves it
    /// is the only live reference to the value for its whole duration, and
    /// rejects the program otherwise. That exclusivity is half of the
    /// guarantee against two code paths driving one pin: the other half is
    /// [`crate::device::PinHandle`] uniqueness, which ensures at most one
    /// pin value exists per physical pin in safe code. Given one value and
    /// one live reference to it, the classic embedded failure — two code
    /// paths performing interleaved read-modify-write sequences on one
    /// register, each overwriting bits the other just set — becomes a
    /// compile error rather than a debugging session.
    fn write(&mut self, value: T) -> Result<(), Self::Error>;
}

/// Reads a value of type `T` from a peripheral.
///
/// The counterpart to [`Write`], with the same "generic over the value" rule.
pub trait Read<T>: ErrorType {
    /// Read the peripheral's current value.
    ///
    /// Takes `&mut self`, the same shape as [`Write::write`]: the borrow
    /// checker guarantees the caller exclusive access to the peripheral for
    /// the duration of the read. That is deliberately stronger than a pure
    /// observation needs, and the reason is that not every hardware read *is*
    /// a pure observation — reading a UART data register removes a byte from
    /// the receive FIFO, the hardware first-in-first-out buffer where
    /// incoming bytes queue until read; reading
    /// an interrupt status register may clear it. `&mut self` lets an
    /// implementation be one of those without mutating peripheral state
    /// behind a shared borrow. The cost is paid by reads that genuinely only
    /// observe (the RP2350 GPIO implementation samples `GPIO_IN`, which
    /// changes nothing): they still demand exclusive access, so two borrowers
    /// cannot sample the same peripheral concurrently even where that would
    /// be harmless.
    fn read(&mut self) -> Result<T, Self::Error>;
}
