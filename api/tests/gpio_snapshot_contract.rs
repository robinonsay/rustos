//! Contract checks for [`api::gpio::InputSnapshot`] and
//! [`api::gpio::InputLevels`] (clauses SNP-1 to SNP-4).
//!
//! The checks drive a group of simulated pins and compare each snapshot with
//! the pin levels. SNP-4 (one instant) is a property of how the
//! implementation reads the hardware and is checked by inspection of the
//! driver and on the development board; here it is checked against a
//! simulation in which the levels change between two separate pin reads.

use std::cell::Cell;
use std::rc::Rc;

use api::common::ErrorType;
use api::gpio::{InputLevels, InputSnapshot};

type Check = Result<(), &'static str>;

fn ok_or(cond: bool, clause: &'static str) -> Check {
    if cond { Ok(()) } else { Err(clause) }
}

/// Pin levels a test controls, as a 32-bit word.
type Pins = Rc<Cell<u32>>;

/// SNP-1 to SNP-3 over a walk of pin patterns.
fn check_levels<S: InputSnapshot>(s: &mut S, pins: &Pins, mask: u32) -> Check {
    for pattern in [
        0,
        u32::MAX,
        0xaaaa_aaaa,
        0x5555_5555,
        1,
        1 << 31,
        0x0f0f_0f0f,
    ] {
        pins.set(pattern);
        let levels = s.snapshot().map_err(|_| "SNP-3")?;
        ok_or(levels.mask() == mask, "SNP-1")?;
        ok_or(levels.bits() & !mask == 0, "SNP-2")?;
        for pin in 0..32 {
            let expected = (mask >> pin) & 1 == 1;
            let level = levels.level(pin);
            ok_or(level.is_some() == expected, "SNP-1")?;
            if let Some(high) = level {
                ok_or(high == ((pattern >> pin) & 1 == 1), "SNP-3")?;
            }
        }
    }
    Ok(())
}

/// SNP-4: `pins` changes from `before` to `after` in the middle of what a
/// pin-by-pin reader would do; a snapshot must equal one of the two, masked.
fn check_one_instant<S: InputSnapshot>(
    s: &mut S,
    flip: &Cell<bool>,
    mask: u32,
    before: u32,
    after: u32,
) -> Check {
    flip.set(true);
    let bits = s.snapshot().map_err(|_| "SNP-4")?.bits();
    ok_or(bits == before & mask || bits == after & mask, "SNP-4")
}

/// Reference model: one read of the whole word.
struct Model {
    pins: Pins,
    mask: u32,
    /// When set, the pins change to `after` right after the first read.
    flip: Rc<Cell<bool>>,
    after: u32,
    /// Fault: reads each pin separately, one word read per pin.
    per_pin: bool,
    /// Fault: forgets to mask.
    unmasked: bool,
}

impl ErrorType for Model {
    type Error = core::convert::Infallible;
}

impl Model {
    fn new(mask: u32) -> Self {
        Self {
            pins: Rc::new(Cell::new(0)),
            mask,
            flip: Rc::new(Cell::new(false)),
            after: 0,
            per_pin: false,
            unmasked: false,
        }
    }
    fn read_word(&self) -> u32 {
        let v = self.pins.get();
        if self.flip.replace(false) {
            self.pins.set(self.after);
        }
        v
    }
}

impl InputSnapshot for Model {
    fn snapshot(&mut self) -> Result<InputLevels, Self::Error> {
        let bits = if self.per_pin {
            (0..32).fold(0, |acc, pin| acc | (self.read_word() & (1 << pin)))
        } else {
            self.read_word()
        };
        if self.unmasked {
            // `InputLevels::new` always masks the bits, so the only way an
            // implementation can get the group wrong is to report the wrong
            // mask.
            return Ok(InputLevels::new(bits, u32::MAX));
        }
        Ok(InputLevels::new(bits, self.mask))
    }
}

const KEYER_MASK: u32 = (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5) | (1 << 6);
/// The keyer group plus pin 0, the first pin a pin-by-pin reader samples, so
/// a change after that first read shows up as a mixed snapshot.
const FLIP_MASK: u32 = KEYER_MASK | 1;

#[test]
fn reference_model_satisfies_snp_1_to_3() {
    let mut m = Model::new(KEYER_MASK);
    let pins = m.pins.clone();
    assert_eq!(check_levels(&mut m, &pins, KEYER_MASK), Ok(()));
}

#[test]
fn reference_model_samples_at_one_instant() {
    let mut m = Model::new(FLIP_MASK);
    m.pins.set(0b0000_0001);
    m.after = 0b0111_1000;
    let flip = m.flip.clone();
    assert_eq!(
        check_one_instant(&mut m, &flip, FLIP_MASK, 0b0000_0001, 0b0111_1000),
        Ok(())
    );
}

#[test]
fn per_pin_reads_are_detected_as_snp_4() {
    let mut m = Model::new(FLIP_MASK);
    m.per_pin = true;
    m.pins.set(0b0000_0001);
    m.after = 0b0111_1000;
    let flip = m.flip.clone();
    assert_eq!(
        check_one_instant(&mut m, &flip, FLIP_MASK, 0b0000_0001, 0b0111_1000),
        Err("SNP-4")
    );
}

#[test]
fn wrong_mask_is_detected_as_snp_1() {
    let mut m = Model::new(KEYER_MASK);
    m.unmasked = true;
    let pins = m.pins.clone();
    assert_eq!(check_levels(&mut m, &pins, KEYER_MASK), Err("SNP-1"));
}

#[test]
fn input_levels_masks_and_reports_levels() {
    let l = InputLevels::new(0b1010, 0b0110);
    assert_eq!((l.bits(), l.mask()), (0b0010, 0b0110));
    assert_eq!(l.level(1), Some(true));
    assert_eq!(l.level(2), Some(false));
    assert_eq!(l.level(3), None);
    assert_eq!(l.level(32), None);
    assert_eq!(l.level(u32::MAX), None);
}
