//! A scripted register file for host tests of the drivers (test builds only).
//!
//! [`FakeRegs`] implements [`Regs`] over an in-memory map from address to
//! value. It records every access in order, so a test can assert the exact
//! register sequence a driver performs against the datasheet's programming
//! sequence, and it can be scripted so that a register returns a series of
//! values (a status bit that sets after N polls, or never). Writes through the
//! atomic aliases of §2.1.3 are applied to the base register the way the bus
//! applies them, except for the blocks that have no aliases (SIO, and the
//! Cortex-M33 private peripherals).

extern crate std;

use std::collections::{BTreeMap, VecDeque};
use std::vec::Vec;

use super::{ALIAS_CLR, ALIAS_SET, ALIAS_XOR, RegAddr, Regs};

/// One recorded register access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Access {
    /// A read of `block + offset` that returned `value`.
    Read {
        block: RegAddr,
        offset: usize,
        value: u32,
    },
    /// A write of `value` to `block + offset` (offset includes any alias).
    Write {
        block: RegAddr,
        offset: usize,
        value: u32,
    },
}

/// In-memory register file with an access log and scripted reads.
pub(crate) struct FakeRegs {
    values: BTreeMap<(usize, usize), u32>,
    scripted: BTreeMap<(usize, usize), VecDeque<u32>>,
    log: Vec<Access>,
}

/// Blocks without the `+0x1000/+0x2000/+0x3000` alias views (§2.1.3).
fn has_aliases(block: RegAddr) -> bool {
    !matches!(block, RegAddr::SIO)
}

impl FakeRegs {
    /// An empty register file: every register reads 0 until set or written.
    pub(crate) fn new() -> Self {
        Self {
            values: BTreeMap::new(),
            scripted: BTreeMap::new(),
            log: Vec::new(),
        }
    }

    fn key(block: RegAddr, offset: usize) -> (usize, usize) {
        (block as usize, offset)
    }

    /// Preset the value of a register without logging an access.
    pub(crate) fn set(&mut self, block: RegAddr, offset: usize, value: u32) {
        self.values.insert(Self::key(block, offset), value);
    }

    /// Current value of a register (0 if never set or written).
    pub(crate) fn get(&self, block: RegAddr, offset: usize) -> u32 {
        self.values
            .get(&Self::key(block, offset))
            .copied()
            .unwrap_or(0)
    }

    /// The next reads of `block + offset` return `values` in order; after the
    /// script runs out the register returns its last scripted value. Replaces
    /// any script set earlier for the same register.
    pub(crate) fn script_reads(&mut self, block: RegAddr, offset: usize, values: &[u32]) {
        self.scripted
            .insert(Self::key(block, offset), values.iter().copied().collect());
    }

    /// Every access so far, in order.
    pub(crate) fn log(&self) -> &[Access] {
        &self.log
    }

    /// The writes so far, in order, as `(block, offset, value)`.
    pub(crate) fn writes(&self) -> Vec<(RegAddr, usize, u32)> {
        self.log
            .iter()
            .filter_map(|a| match *a {
                Access::Write {
                    block,
                    offset,
                    value,
                } => Some((block, offset, value)),
                Access::Read { .. } => None,
            })
            .collect()
    }

    /// How many times `block + offset` has been read.
    pub(crate) fn reads_of(&self, block: RegAddr, offset: usize) -> usize {
        self.log
            .iter()
            .filter(|a| matches!(a, Access::Read { block: b, offset: o, .. } if *b == block && *o == offset))
            .count()
    }

    /// Whether any write went to `block` at all.
    pub(crate) fn wrote_block(&self, block: RegAddr) -> bool {
        self.writes().iter().any(|(b, _, _)| *b == block)
    }
}

impl Regs for FakeRegs {
    fn read(&mut self, block: RegAddr, offset: usize) -> u32 {
        let key = Self::key(block, offset);
        let value = match self.scripted.get_mut(&key) {
            Some(queue) if queue.len() > 1 => queue.pop_front().unwrap_or(0),
            Some(queue) => queue.front().copied().unwrap_or(0),
            None => self.get(block, offset),
        };
        self.log.push(Access::Read {
            block,
            offset,
            value,
        });
        value
    }

    fn write(&mut self, block: RegAddr, offset: usize, value: u32) {
        self.log.push(Access::Write {
            block,
            offset,
            value,
        });
        let alias = offset & 0x3000;
        if !has_aliases(block) || alias == 0 {
            self.set(block, offset, value);
            return;
        }
        let base = offset - alias;
        let current = self.get(block, base);
        let updated = match alias {
            ALIAS_XOR => current ^ value,
            ALIAS_SET => current | value,
            ALIAS_CLR => current & !value,
            _ => current,
        };
        self.set(block, base, updated);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_writes_apply_set_clear_and_xor_to_the_base_register() {
        let mut regs = FakeRegs::new();
        regs.set(RegAddr::RESET, 0, 0b1010);
        regs.write(RegAddr::RESET, ALIAS_SET, 0b0100);
        assert_eq!(regs.get(RegAddr::RESET, 0), 0b1110);
        regs.write(RegAddr::RESET, ALIAS_CLR, 0b1000);
        assert_eq!(regs.get(RegAddr::RESET, 0), 0b0110);
        regs.write(RegAddr::RESET, ALIAS_XOR, 0b0011);
        assert_eq!(regs.get(RegAddr::RESET, 0), 0b0101);
    }

    #[test]
    fn sio_offsets_are_plain_registers() {
        let mut regs = FakeRegs::new();
        regs.write(RegAddr::SIO, 0x018, 7);
        assert_eq!(regs.get(RegAddr::SIO, 0x018), 7);
    }

    #[test]
    fn scripted_reads_hold_their_last_value() {
        let mut regs = FakeRegs::new();
        regs.script_reads(RegAddr::XOSC, 4, &[1, 2]);
        assert_eq!(
            [
                regs.read(RegAddr::XOSC, 4),
                regs.read(RegAddr::XOSC, 4),
                regs.read(RegAddr::XOSC, 4)
            ],
            [1, 2, 2]
        );
    }
}
