//! Portable interrupt-controller trait.
//!
//! An interrupt **line** is a numbered input of the processor's interrupt
//! controller that a peripheral raises when it wants attention (a timer alarm
//! firing, a FIFO filling). Enabling a line lets that request reach the
//! processor, which then runs the handler installed for the line; a **pending**
//! line is one whose request has been latched but not yet serviced.
//!
//! The trait deliberately has **no priority operation**. Every line an
//! application enables keeps the controller's reset priority, so no handler
//! can preempt another and every handler runs to completion (cwht coding
//! standard CS-34). An implementation must not offer a priority setter through
//! this trait.
//!
//! # Contract
//!
//! Every implementation satisfies these clauses; `api/tests/irq_contract.rs`
//! states each as a check that runs against any implementation.
//!
//! * **IRQ-1** After `enable(n)` returns `Ok`, `is_enabled(n)` is `true`.
//! * **IRQ-2** After `disable(n)` returns `Ok`, `is_enabled(n)` is `false`.
//! * **IRQ-3** Enabling, disabling, pending or unpending line `n` leaves the
//!   enabled and pending state of every other line unchanged.
//! * **IRQ-4** A line number the controller does not have is rejected with
//!   `Err`, and no line's state changes.
//! * **IRQ-5** After `pend(n)`, `is_pending(n)` is `true` while the line is
//!   disabled; after `unpend(n)`, `is_pending(n)` is `false`.
//! * **IRQ-6** Every line starts disabled and not pending when the
//!   implementation is constructed after reset.

use crate::common::ErrorType;

/// Enables, disables, pends and inspects numbered interrupt lines.
///
/// Line numbers are the controller's own (for the RP2350, 0 to 51, Table 94
/// of its datasheet); the trait carries no chip facts.
///
/// # Errors
///
/// Every method returns `Err` for a line number the controller does not
/// have, and changes nothing (IRQ-4). No other error is permitted.
#[allow(clippy::missing_errors_doc)] // the Errors section above covers every method
pub trait InterruptController: ErrorType {
    /// Let requests on `line` reach the processor (IRQ-1).
    fn enable(&mut self, line: u16) -> Result<(), Self::Error>;
    /// Stop requests on `line` from reaching the processor (IRQ-2). A request
    /// already latched stays pending.
    fn disable(&mut self, line: u16) -> Result<(), Self::Error>;
    /// Whether `line` is enabled.
    fn is_enabled(&mut self, line: u16) -> Result<bool, Self::Error>;
    /// Latch a request on `line` from software (IRQ-5). If the line is
    /// enabled its handler runs as soon as interrupts are unmasked.
    fn pend(&mut self, line: u16) -> Result<(), Self::Error>;
    /// Discard a latched request on `line` (IRQ-5).
    fn unpend(&mut self, line: u16) -> Result<(), Self::Error>;
    /// Whether a request on `line` is latched and not yet serviced.
    fn is_pending(&mut self, line: u16) -> Result<bool, Self::Error>;
}
