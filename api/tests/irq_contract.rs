//! Contract checks for [`api::irq::InterruptController`] (clauses IRQ-1 to
//! IRQ-6 of the trait's module documentation).
//!
//! Each `check_*` function states one clause against any implementation and
//! returns the clause id on failure. The tests run every check against a
//! reference model of the contract, and run chosen checks against broken
//! models to show that each check detects the violation it names.

use api::common::ErrorType;
use api::irq::InterruptController;

/// Result of one contract check: `Err` carries the violated clause.
type Check = Result<(), &'static str>;

fn ok_or(cond: bool, clause: &'static str) -> Check {
    if cond { Ok(()) } else { Err(clause) }
}

/// IRQ-6, then IRQ-1 and IRQ-2 on every line.
fn check_enable_disable<C: InterruptController>(c: &mut C, lines: u16) -> Check {
    for n in 0..lines {
        ok_or(!c.is_enabled(n).map_err(|_| "IRQ-6")?, "IRQ-6")?;
        c.enable(n).map_err(|_| "IRQ-1")?;
        ok_or(c.is_enabled(n).map_err(|_| "IRQ-1")?, "IRQ-1")?;
        c.disable(n).map_err(|_| "IRQ-2")?;
        ok_or(!c.is_enabled(n).map_err(|_| "IRQ-2")?, "IRQ-2")?;
    }
    Ok(())
}

/// IRQ-3: an operation on one line leaves every other line as it was.
fn check_independence<C: InterruptController>(c: &mut C, lines: u16) -> Check {
    for n in 0..lines {
        c.enable(n).map_err(|_| "IRQ-3")?;
        c.pend(n).map_err(|_| "IRQ-3")?;
        for m in (0..lines).filter(|m| *m != n) {
            ok_or(!c.is_enabled(m).map_err(|_| "IRQ-3")?, "IRQ-3")?;
            ok_or(!c.is_pending(m).map_err(|_| "IRQ-3")?, "IRQ-3")?;
        }
        c.unpend(n).map_err(|_| "IRQ-3")?;
        c.disable(n).map_err(|_| "IRQ-3")?;
    }
    Ok(())
}

/// IRQ-4: every operation on the first line past the end is rejected and
/// changes nothing.
fn check_out_of_range<C: InterruptController>(c: &mut C, lines: u16) -> Check {
    ok_or(c.enable(lines).is_err(), "IRQ-4")?;
    ok_or(c.disable(lines).is_err(), "IRQ-4")?;
    ok_or(c.is_enabled(lines).is_err(), "IRQ-4")?;
    ok_or(c.pend(lines).is_err(), "IRQ-4")?;
    ok_or(c.unpend(lines).is_err(), "IRQ-4")?;
    ok_or(c.is_pending(lines).is_err(), "IRQ-4")?;
    ok_or(c.enable(u16::MAX).is_err(), "IRQ-4")?;
    for n in 0..lines {
        ok_or(!c.is_enabled(n).map_err(|_| "IRQ-4")?, "IRQ-4")?;
        ok_or(!c.is_pending(n).map_err(|_| "IRQ-4")?, "IRQ-4")?;
    }
    Ok(())
}

/// IRQ-5: pend and unpend on a disabled line.
fn check_pending<C: InterruptController>(c: &mut C, lines: u16) -> Check {
    for n in 0..lines {
        c.pend(n).map_err(|_| "IRQ-5")?;
        ok_or(c.is_pending(n).map_err(|_| "IRQ-5")?, "IRQ-5")?;
        c.unpend(n).map_err(|_| "IRQ-5")?;
        ok_or(!c.is_pending(n).map_err(|_| "IRQ-5")?, "IRQ-5")?;
    }
    Ok(())
}

/// Run every check on fresh controllers from `make`.
fn check_all<C: InterruptController>(make: impl Fn() -> C, lines: u16) -> Check {
    check_enable_disable(&mut make(), lines)?;
    check_independence(&mut make(), lines)?;
    check_out_of_range(&mut make(), lines)?;
    check_pending(&mut make(), lines)
}

/// The reference model: two bit sets of up to 64 lines.
#[derive(Default)]
struct Model {
    lines: u16,
    enabled: u64,
    pending: u64,
    /// Fault injection for the negative tests.
    fault: Fault,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum Fault {
    #[default]
    None,
    /// `enable(n)` also enables line `n + 1` (violates IRQ-3).
    EnableSpillsOver,
    /// Out-of-range lines are silently accepted (violates IRQ-4).
    AcceptsAnyLine,
    /// `disable` does nothing (violates IRQ-2).
    DisableIgnored,
}

#[derive(Debug)]
struct OutOfRange;

impl ErrorType for Model {
    type Error = OutOfRange;
}

impl Model {
    fn new(lines: u16, fault: Fault) -> Self {
        Self {
            lines,
            fault,
            ..Self::default()
        }
    }
    fn bit(&self, line: u16) -> Result<u64, OutOfRange> {
        if line < self.lines {
            Ok(1 << line)
        } else if self.fault == Fault::AcceptsAnyLine {
            Ok(0)
        } else {
            Err(OutOfRange)
        }
    }
}

impl InterruptController for Model {
    fn enable(&mut self, line: u16) -> Result<(), OutOfRange> {
        let bit = self.bit(line)?;
        self.enabled |= bit;
        if self.fault == Fault::EnableSpillsOver {
            self.enabled |= bit << 1;
        }
        Ok(())
    }
    fn disable(&mut self, line: u16) -> Result<(), OutOfRange> {
        let bit = self.bit(line)?;
        if self.fault != Fault::DisableIgnored {
            self.enabled &= !bit;
        }
        Ok(())
    }
    fn is_enabled(&mut self, line: u16) -> Result<bool, OutOfRange> {
        Ok(self.enabled & self.bit(line)? != 0)
    }
    fn pend(&mut self, line: u16) -> Result<(), OutOfRange> {
        self.pending |= self.bit(line)?;
        Ok(())
    }
    fn unpend(&mut self, line: u16) -> Result<(), OutOfRange> {
        self.pending &= !self.bit(line)?;
        Ok(())
    }
    fn is_pending(&mut self, line: u16) -> Result<bool, OutOfRange> {
        Ok(self.pending & self.bit(line)? != 0)
    }
}

#[test]
fn reference_model_satisfies_the_contract() {
    assert_eq!(check_all(|| Model::new(52, Fault::None), 52), Ok(()));
}

#[test]
fn spill_over_is_detected_as_irq_3() {
    assert_eq!(
        check_independence(&mut Model::new(52, Fault::EnableSpillsOver), 52),
        Err("IRQ-3")
    );
}

#[test]
fn accepting_any_line_is_detected_as_irq_4() {
    assert_eq!(
        check_out_of_range(&mut Model::new(52, Fault::AcceptsAnyLine), 52),
        Err("IRQ-4")
    );
}

#[test]
fn ignored_disable_is_detected_as_irq_2() {
    assert_eq!(
        check_enable_disable(&mut Model::new(52, Fault::DisableIgnored), 52),
        Err("IRQ-2")
    );
}
