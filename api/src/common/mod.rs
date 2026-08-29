/// A peripheral that must be brought up before use and can be shut back down.
///
/// Implemented by drivers rather than by a central reset controller, so each
/// driver keeps the knowledge of which reset lines it needs next to the code
/// that needs them. This trait names only the lifecycle; which registers are
/// involved is the implementing crate's business. On the RP2350, for example,
/// each driver owns one or more bits in the chip-wide `RESETS` register, and
/// a driver that reaches pins generally owns more than one bit: `UART0`
/// alone does not produce a working UART, because the signal still has to get
/// through `IO_BANK0`'s function mux and a `PADS_BANK0` pad.
pub trait Block
{
    /// Release this peripheral's blocks from reset and block until the
    /// hardware reports them ready.
    ///
    /// Must be called before touching any of the peripheral's registers. On
    /// the RP2350, accesses to a block still in reset do not fault — they are
    /// accepted by the bus and discarded — so skipping this produces a
    /// peripheral that silently ignores every write.
    ///
    /// # Safety
    ///
    /// Writes a chip-wide control register shared with every other driver, and
    /// leaves hardware running. Implementations must touch only their own bits.
    unsafe fn start(&mut self);

    /// Return this peripheral's blocks to reset.
    ///
    /// # Safety
    ///
    /// Any handle to this peripheral becomes non-functional. As with
    /// [`start`](Block::start), implementations must confine themselves to
    /// their own reset lines. The stakes can be fatal to the program: on the
    /// RP2350, asserting reset on `IO_QSPI` or `PADS_QSPI` cuts the pins that
    /// XIP (execute-in-place, code running directly from flash) fetches
    /// instructions from, and execution stops mid-fetch with no fault and no
    /// output.
    unsafe fn stop(&mut self);
}


/// Declares the single error type a peripheral reports.
///
/// Every fallible trait in this crate ([`Write`], [`Read`], and the factory
/// traits in [`crate::gpio`]) takes `ErrorType` as a supertrait and returns
/// `Self::Error`. A type therefore names its error **once**, and all of its
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
/// Required so callers can use [`Result::unwrap`] and [`Result::expect`], and
/// so errors can be formatted by a defmt/RTT-style logger. It is deliberately
/// weaker than `core::error::Error`, which would drag in `Display` and force
/// every implementor to write human-readable strings into flash.
pub trait ErrorType {
    /// The error reported by every fallible operation on this type.
    ///
    /// Use [`core::convert::Infallible`] when an operation genuinely cannot
    /// fail — it is an empty enum, so `Result<T, Infallible>` is the same size
    /// as `T` and the error branch is eliminated entirely. Do not reach for it
    /// merely because failure is *unlikely*; that turns a diagnosable error
    /// into undefined behaviour at the point where it would have mattered.
    type Error: core::fmt::Debug;
}

/// Sends a value of type `T` to a peripheral.
///
/// Generic over the value rather than the peripheral, so a single type can be
/// written in several representations. A GPIO pin implements `Write<bool>`; a
/// UART would implement `Write<u8>`; a DAC might implement both `Write<u16>`
/// and `Write<f32>`.
///
/// Implementations should be **idempotent and non-blocking** where the
/// hardware allows. Writing the same value twice must be safe, and `write`
/// should not spin waiting on the peripheral unless that is inherent to the
/// operation.
pub trait Write<T>: ErrorType {
    /// Write `value` to the peripheral.
    ///
    /// Takes `&mut self` because a write is a state change. This is what stops
    /// two owners from driving the same pin from different places — the borrow
    /// checker enforces exclusive access to the peripheral handle, so the
    /// usual embedded hazard of two subsystems fighting over one register
    /// becomes a compile error rather than a debugging session.
    fn write(&mut self, value: T) -> Result<(), Self::Error>;
}

/// Reads a value of type `T` from a peripheral.
///
/// The counterpart to [`Write`], with the same "generic over the value" rule.
pub trait Read<T>: ErrorType {
    /// Read the peripheral's current value.
    ///
    /// Takes `&self`: this trait models reads that only observe, so several
    /// borrowers can sample the same peripheral concurrently. That choice has
    /// a cost worth knowing about — some hardware reads have side effects
    /// (reading a UART data register pops a FIFO; reading an interrupt status
    /// register may clear it), and an implementation of *this* trait for such
    /// a register would mutate peripheral state behind a shared borrow. Those
    /// operations belong behind `&mut self` methods instead. The RP2350 GPIO
    /// implementation samples `GPIO_IN`, which changes nothing.
    fn read(&self) -> Result<T, Self::Error>;
}
