---
document_type: Tutorial Chapter — Registers and Bit Manipulation
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 6 of 7
revision: A
effective_date: 2026-08-25
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapters 01-05
sources: RP2350 datasheet 2.1.3
---

# Chapter 06 — Registers and Bit Manipulation

## 6.1 `volatile`

A plain `*ptr` read is, to the compiler, a pure load from memory. It is free to
cache the value in a register, reorder it, duplicate it, merge two accesses into
one, or delete it entirely if the result looks unused. All valid for ordinary
memory — all catastrophic for a hardware register, where **the access itself is
the point**.

`read_volatile` / `write_volatile` tell LLVM: this access has effects you cannot
see, so emit exactly the accesses I wrote, exactly once, in the order I wrote
them.

The guarantee stops at the CPU boundary. `volatile` orders accesses in the
*instruction stream*, not through write buffers or the pipeline — that is what
`dsb` / `isb` are for.

## 6.2 Read-modify-write, or plain write?

| Register shape | Correct access |
|---|---|
| several independent live fields (e.g. `CPACR`) | **read-modify-write** |
| the whole register is one value (e.g. `VTOR`) | **plain write** |
| freshly-reset register where all other fields want 0 | plain write is *safer* |

Applying RMW everywhere out of caution is its own bug — it makes
write-1-to-clear registers behave very strangely, and it can preserve stale
fields you meant to reset.

## 6.3 Rust has no bitfields

There is no equivalent of `struct { unsigned x : 3; }`, and that is a feature.
C bitfields leave allocation order, straddling, and underlying type all
implementation-defined, which is why most embedded C style guides ban them for
MMIO. Rust simply does not offer the footgun.

**Shift and mask explicitly.**

### 6.3.1 Single-bit flags

```rust
pub const PADS_ISO: u32 = 1 << 8;
pub const PADS_OD:  u32 = 1 << 7;
pub const PADS_IE:  u32 = 1 << 6;
pub const PADS_PDE: u32 = 1 << 2;
```

Set with `|`, clear with `& !`.

### 6.3.2 Multi-bit fields — clear, then set

```rust
pub const FUNCSEL_MASK: u32 = 0x1f;
pub const FUNCSEL_SIO:  u32 = 5;

#[inline]
pub const fn with_funcsel(reg: u32, funcsel: u32) -> u32 {
    (reg & !FUNCSEL_MASK) | (funcsel & FUNCSEL_MASK)
}
```

> **The clear-first step is load-bearing, and `FUNCSEL` is the perfect example.**
> Its reset value is `0x1f` — *all five bits set*:
>
> ```rust
> assert_eq!(with_funcsel(0x1f, FUNCSEL_SIO), 5);
> assert_eq!(0x1f | FUNCSEL_SIO, 0x1f);   // the naive version
> ```
>
> `reg | 5` leaves it at `0x1f` (NULL) and the pin stays disconnected. A bug
> that reads correctly and does nothing.

For a field not at bit 0 the full form is:

```rust
(reg & !(MASK << SHIFT)) | ((val & MASK) << SHIFT)
```

## 6.4 This is the host-testable seam

Register *addresses*, *masks*, *shifts* and *offset arithmetic* are pure
computation. Only the `_volatile` calls touch hardware.

That is exactly the split the workspace was built around: put the arithmetic in
the `no_std` library crate, unit-test it on the host with `cargo test`, and keep
only the volatile pokes in the firmware binary.

```rust
#[inline] pub const fn io_bank0_ctrl_offset(pin: u32) -> u32 { 0x004 + 8 * pin }
#[inline] pub const fn pads_bank0_offset(pin: u32)    -> u32 { 0x004 + 4 * pin }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn funcsel_replaces_not_ors() {
        assert_eq!(with_funcsel(0x1f, FUNCSEL_SIO), 5);
    }
    #[test] fn funcsel_preserves_upper_bits() {
        assert_eq!(with_funcsel(0xDEAD_BE00 | 0x1f, FUNCSEL_SIO), 0xDEAD_BE00 | 5);
    }
    #[test] fn gp25_offsets() {
        assert_eq!(io_bank0_ctrl_offset(25), 0xCC);   // 8-byte stride
        assert_eq!(pads_bank0_offset(25),    0x68);   // 4-byte stride
    }
}
```

A `#![no_std]` library is still host-testable: `no_std` only suppresses the
implicit `extern crate std`; when `cargo test` builds it for the host, the
library compiles unchanged and the *test harness* links `std`.

The two different strides for the same pin number are exactly the kind of thing
worth a test.

## 6.5 Atomic register aliases

Every APB/AHB peripheral block is mirrored three times (2.1.3):

| Offset from block base | Effect |
|---|---|
| `+0x0000` | normal read/write |
| `+0x1000` | atomic XOR |
| `+0x2000` | atomic bitmask **set** |
| `+0x3000` | atomic bitmask **clear** |

This is the clean way to flip bits without a read-modify-write — e.g. releasing
two `RESETS` bits in one store.

> **`SIO` is explicitly excluded** from these aliases. It has its own
> `_SET`/`_CLR`/`_XOR` *registers* instead. Getting this backwards writes
> garbage into `0xd0002000`.

## 6.6 Building addresses

Thumb-2 instructions are at most 32 bits, so there is no room for a full 32-bit
immediate. ARM materialises constants in pairs:

```asm
movw r0, #0xed88      ; low  halfword (and zero the top)
movt r0, #0xe000      ; high halfword (leave the low alone)
                      ; r0 = 0xE000ED88
```

And ARM is a **load/store architecture** — arithmetic works only on registers,
memory only via `ldr`/`str`. There is no instruction that ORs a value into a
memory location, so a read-modify-write is structurally three instructions:

```asm
ldr  r1, [r0]             ; READ
orr  r1, r1, #0xf00000    ; MODIFY
str  r1, [r0]             ; WRITE
```

Your Rust maps onto that one-to-one because it has to.
