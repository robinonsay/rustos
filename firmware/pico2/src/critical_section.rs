//! Critical sections and the one sound way to share state with an interrupt
//! handler.
//!
//! [`with`] runs a closure with interrupts masked on the running core and
//! hands it a [`CriticalSection`] token; [`CsCell`] is a `static`-friendly
//! cell whose contents can only be reached through such a token. Together
//! they replace `static mut` (cwht coding standard CS-09): this module is the
//! only place in the crate that masks interrupts.
//!
//! ## Why masking is enough here
//!
//! Masking interrupts (setting `PRIMASK`, Cortex-M33 core register, datasheet
//! §3.7.4.5 p134) stops every handler from running on this core until the
//! mask is restored, so code inside [`with`] cannot be interleaved with a
//! handler that touches the same cell. It does nothing about the *other*
//! core: rustos starts only core 0, and core 1 stays in its bootrom wait
//! state (cwht coding standard CS-23). An application that starts core 1
//! must not share a `CsCell` with it.
//!
//! [`with`] restores `PRIMASK` to the value it read, not to "enabled", so
//! nesting is correct: an inner `with` inside an outer one leaves interrupts
//! masked when it returns.

use core::cell::{Cell, UnsafeCell};
use core::marker::PhantomData;

/// Proof that interrupts are masked on the running core, valid for the
/// closure passed to [`with`].
///
/// Zero-sized, neither `Send` nor `Sync` (the raw-pointer marker), and only
/// constructed by [`with`], so a reference to one cannot outlive the masked
/// region or cross to another context.
pub struct CriticalSection<'cs> {
    _masked: PhantomData<(&'cs (), *const ())>,
}

impl CriticalSection<'_> {
    /// The token for a region in which interrupts are masked. Crate-private:
    /// [`with`] on the target, the host tests for [`CsCell`].
    const fn new() -> Self {
        Self {
            _masked: PhantomData,
        }
    }
}

/// Run `f` with interrupts masked on this core, then restore the previous
/// mask state exactly.
///
/// Keep `f` short: while it runs, no interrupt handler can start, so the
/// keyer's alarm and the 1 kHz sampling tick are delayed by its duration.
#[cfg(target_os = "none")]
#[inline]
pub fn with<R>(f: impl FnOnce(&CriticalSection<'_>) -> R) -> R {
    let primask: u32;
    // SAFETY: `mrs` reads PRIMASK into a general register and `cpsid i` sets PRIMASK to mask
    // every configurable-priority exception (Armv8-M; datasheet section 3.7.4.5, p134). Neither
    // touches memory or the stack. The asm is not `nomem`, so it is also a compiler barrier:
    // no access to shared state inside `f` is moved above the mask.
    unsafe {
        core::arch::asm!("mrs {0}, PRIMASK", "cpsid i", out(reg) primask, options(nostack, preserves_flags))
    };
    let result = f(&CriticalSection::new());
    // SAFETY: `msr PRIMASK` writes back the exact value read above: interrupts are unmasked only
    // if they were unmasked on entry, so nested regions stay masked. No memory or stack access;
    // not `nomem`, so no access inside `f` is moved below the restore.
    unsafe {
        core::arch::asm!("msr PRIMASK, {0}", in(reg) primask, options(nostack, preserves_flags))
    };
    result
}

/// The cell was already borrowed by an enclosing [`CsCell::with_mut`] in the
/// same critical section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Busy;

/// A cell for a `static` shared between thread code and interrupt handlers.
///
/// Every access takes a [`CriticalSection`], so no handler on this core can
/// run during it; a borrow flag makes a nested access to the same cell inside
/// [`with_mut`](Self::with_mut) return [`Busy`] instead of a second `&mut`.
///
/// ```ignore
/// static TICK: CsCell<Option<Rp2350Alarm<1>>> = CsCell::new(None);
/// critical_section::with(|cs| TICK.replace(cs, Some(alarm)));    // thread code
/// critical_section::with(|cs| TICK.with_mut(cs, |slot| { /* ... */ })); // handler
/// ```
pub struct CsCell<T> {
    value: UnsafeCell<T>,
    borrowed: Cell<bool>,
}

// SAFETY: `Sync` lets a `static CsCell<T>` be reached from thread code and from interrupt
// handlers. Every access to `value` and `borrowed` goes through a method that takes a
// `&CriticalSection`, which exists only while interrupts are masked on this core, so accesses
// never overlap on core 0; core 1 runs no rustos code (module doc; cwht CS-23). `T: Send`
// because a value put in by one context is taken out by another.
unsafe impl<T: Send> Sync for CsCell<T> {}

impl<T> CsCell<T> {
    /// A cell holding `value`; `const`, so it can initialise a `static`.
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            borrowed: Cell::new(false),
        }
    }

    /// Put `value` in the cell and return what it held.
    ///
    /// # Errors
    ///
    /// [`Busy`] inside an enclosing [`with_mut`](Self::with_mut) of the same
    /// cell; the cell is unchanged and `value` is dropped.
    pub fn replace(&self, _cs: &CriticalSection<'_>, value: T) -> Result<T, Busy> {
        if self.borrowed.get() {
            return Err(Busy);
        }
        // SAFETY: interrupts are masked (the `_cs` token) and no `with_mut` of this cell is
        // active (`borrowed` is false), so no other reference into `value` exists; the `&mut`
        // lives only for this `mem::replace`.
        Ok(core::mem::replace(unsafe { &mut *self.value.get() }, value))
    }

    /// Run `f` on the contents.
    ///
    /// # Errors
    ///
    /// [`Busy`] when called inside an enclosing `with_mut` of the same cell;
    /// `f` is not run.
    pub fn with_mut<R>(
        &self,
        _cs: &CriticalSection<'_>,
        f: impl FnOnce(&mut T) -> R,
    ) -> Result<R, Busy> {
        if self.borrowed.replace(true) {
            return Err(Busy);
        }
        // SAFETY: interrupts are masked (the `_cs` token), and `borrowed` was false and is now
        // true, so this is the only live reference into `value` until it is cleared below; a
        // nested `replace` or `with_mut` of this cell returns `Busy` without touching `value`.
        let result = f(unsafe { &mut *self.value.get() });
        self.borrowed.set(false);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_returns_the_previous_value() {
        let cell = CsCell::new(1u32);
        let cs = CriticalSection::new();
        assert_eq!(cell.replace(&cs, 2), Ok(1));
        assert_eq!(cell.replace(&cs, 3), Ok(2));
    }

    #[test]
    fn with_mut_changes_the_contents() {
        let cell = CsCell::new(10u32);
        let cs = CriticalSection::new();
        assert_eq!(
            cell.with_mut(&cs, |v| {
                *v += 5;
                *v
            }),
            Ok(15)
        );
        assert_eq!(cell.replace(&cs, 0), Ok(15));
    }

    #[test]
    fn nested_access_to_the_same_cell_is_busy_and_harmless() {
        let cell = CsCell::new(7u32);
        let cs = CriticalSection::new();
        let outer = cell.with_mut(&cs, |v| {
            let inner_replace = cell.replace(&cs, 99);
            let inner_with = cell.with_mut(&cs, |w| *w = 98);
            *v += 1;
            (inner_replace, inner_with)
        });
        assert_eq!(outer, Ok((Err(Busy), Err(Busy))));
        assert_eq!(cell.replace(&cs, 0), Ok(8));
    }

    #[test]
    fn borrow_flag_clears_after_with_mut() {
        let cell = CsCell::new(0u8);
        let cs = CriticalSection::new();
        assert_eq!(cell.with_mut(&cs, |_| ()), Ok(()));
        assert_eq!(cell.with_mut(&cs, |_| ()), Ok(()));
    }

    #[test]
    fn different_cells_nest() {
        let a = CsCell::new(1u8);
        let b = CsCell::new(2u8);
        let cs = CriticalSection::new();
        assert_eq!(a.with_mut(&cs, |x| b.with_mut(&cs, |y| *x + *y)), Ok(Ok(3)));
    }
}
