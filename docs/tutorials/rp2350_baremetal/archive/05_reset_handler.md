---
document_type: Tutorial Chapter — The Reset Handler
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 5 of 7
revision: A
effective_date: 2026-08-25
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapters 01-04
sources: RP2350 datasheet 3.4, 3.6, 3.7, 3.7.5, Tables 201/229
---

# Chapter 05 — The Reset Handler

The bootrom loads SP from vector word 0 and jumps to word 1. Everything a
hosted program's runtime would have done for you now has to happen here.

## 5.1 What it must do, in order

```rust
#[unsafe(no_mangle)] pub extern "C" fn OnReset() -> ! {
    unsafe {
        enable_fpu();     // 1 - before any FP instruction can execute
        reset_vtor();     // 2 - before any exception can be taken
        reset_data();     // 3 - before any static is read
        reset_bss();      // 4
    }
    main()                // 5 - never returns
}
```

`-> !` is not decoration. The reset handler is entered by the hardware loading
its address into the PC — there is no caller and no return address. If it
returns, the CPU pops garbage into the PC. `-> !` turns "must never return"
from a convention you remember into something the compiler enforces.

## 5.2 Enabling the FPU

### 5.2.1 Why it is off

`thumbv8m.main-none-eabihf` is a **hard-float** target: the compiler may emit FP
instructions anywhere. `CPACR` resets to `0x0` — coprocessor access **denied**
— so the first FP instruction faults.

The M33 has a coprocessor port (64 bits/cycle to the register file). The FPU
occupies CP10/CP11; RP2350 attaches three more of its own (3.4):

| Coprocessor | Block |
|---|---|
| CP0 | GPIO coprocessor (GPIOC) |
| CP4 / CP5 | double-precision coprocessor (DCP), Secure / Non-secure |
| CP7 | redundancy coprocessor (RCP) |
| CP10 / CP11 | standard ARM single-precision FPU |

Section 3.6 states the rule:

> Before accessing a coprocessor from Secure code, that coprocessor must first
> be enabled by setting the corresponding bit in the CPACR.

The point is *access control*, not power saving: an RTOS that does not want to
save 32 FP registers per context switch leaves the FPU off so that any task
using FP traps.

### 5.2.2 `CPACR`

**`0xE000ED88`** = `PPB_BASE` (`0xE0000000`, per 3.7.5) + offset `0x0ED88`.

One **2-bit field per coprocessor**:

```
bits  1:0 -> CP0     bits 15:14 -> CP7
bits  3:2 -> CP1     bits 21:20 -> CP10
   ...              bits 23:22 -> CP11
```

Encoding (ARMv8-M architecture — **not** stated in the RP2350 datasheet):

| Value | Meaning |
|---|---|
| `0b00` | access denied — **reset value** |
| `0b01` | privileged access only |
| `0b10` | reserved |
| `0b11` | full access |

Both CP10 and CP11 must be programmed identically. Table 229:

> If the value of this bit is not programmed to the same value as the CP10
> field, then the value is UNKNOWN.

So the mask is `(0b11 << 20) | (0b11 << 22)` = **`0x00F00000`**.

### 5.2.3 The code

```rust
const PPB_BASE: usize = 0xE000_0000;
const CPACR: *mut u32 = (PPB_BASE + 0x0ED88) as *mut u32;
const CPACR_FPU_FULL: u32 = (0b11 << 20) | (0b11 << 22);

#[inline]
unsafe fn enable_fpu() { unsafe {
    let current = CPACR.read_volatile();      // READ
    let updated = current | CPACR_FPU_FULL;   // MODIFY
    CPACR.write_volatile(updated);            // WRITE
    core::arch::asm!("dsb", "isb", options(nostack, preserves_flags));
}}
```

> **Read-modify-write is mandatory here.** A plain
> `write_volatile(0x00F00000)` zeroes CP0, CP4, CP5 and CP7 — switching off the
> GPIO coprocessor, both double-precision units, and the redundancy
> coprocessor. You would have a working FPU and mysteriously dead coprocessors.
>
> (On a *cold* reset every field is already 0, so RMW is bit-identical to a
> plain write. It matters because the bootrom's CPACR state at handoff is not
> documented.)

`dsb` waits for the store to land; `isb` flushes the pipeline so following
instructions are decoded under the new configuration. Without them an FP
instruction already in the pipeline was decoded while the FPU was disabled.

### 5.2.4 Failure mode

An FP instruction with CP10/CP11 denied raises a **UsageFault with the NOCP
bit** set. UsageFault is not enabled by default, so it escalates straight to
**HardFault** — a likely first-boot failure on a hard-float target, with no
obvious cause.

## 5.3 Setting VTOR

**`0xE000ED08`** = `PPB_BASE + 0x0ED08`.

When exception *N* fires, the NVIC computes `VTOR + (N x 4)`, loads the word
there, and jumps. That is the entire dispatch mechanism — VTOR is the single
register that says where your table lives.

**VTOR resets to `0x00000000`**, which on RP2350 is the **boot ROM**. Until you
write it, every exception dispatches through the ROM's table. Your reset handler
runs fine (the bootrom jumped to it directly, no vector lookup involved) but the
first fault or interrupt vectors into ROM.

```rust
const VTOR: *mut u32 = (PPB_BASE + 0x0ED08) as *mut u32;

#[inline]
unsafe fn reset_vtor() { unsafe {
    VTOR.write_volatile(&raw const VECTOR_TABLE as u32);
    core::arch::asm!("dsb", "isb", options(nostack, preserves_flags));
}}
```

Two deliberate choices:

- **Take the address of the static**, not a literal `0x10000000`. Identical
  codegen, but it stays correct if the table moves.
- **Plain write, not read-modify-write.** `TBLOFF` is bits 31:7 and bits 6:0 are
  reserved — the whole register *is* the address, so there is nothing to
  preserve.

> The general rule: **read-modify-write when a register is a bag of unrelated
> fields; plain write when the register is one value.** Applying RMW everywhere
> out of caution is its own bug — it makes write-1-to-clear registers behave
> very strangely.

### 5.3.1 Does the bootrom set it for you?

The datasheet documents "Set Secure main sp and VTOR" for the **core 1** launch
protocol, but the core-0 flash boot path says only "Set Secure main sp, then
call into the entry point provided." VTOR is not mentioned there.

Set it yourself. Two instructions removes the ambiguity.

### 5.3.2 Are the barriers required?

Strictly, **no** — at this point no exception can be taken: NVIC `ISER` resets
to 0, SysTick is disabled, and `M33_EPPB: NMI_MASK0` resets to `0x00000000` so
nothing is routed to NMI either. The risk of omitting them here is zero, not
small. Keeping them costs two instructions and is correct habit.

## 5.4 `.data` copy and `.bss` zero

### 5.4.1 Why `.bss` must be zeroed

SRAM does not power on as zeros. It powers on as whatever the silicon settles
into — often stable enough across resets to fool you.

Rust guarantees `static mut COUNTER: u32 = 0` reads back as `0`. On a hosted OS
the loader makes that true. Here, exactly one thing upholds it: **your zeroing
loop**. Skip it and you read uninitialised memory while the type system insists
it is zero.

### 5.4.2 The symbol declarations

```rust
unsafe extern "C" {
    static __sidata: u32;  static __sdata: u32;  static __edata: u32;
    static __sbss:   u32;  static __ebss:  u32;
}
```

`static`, not `static mut` — you only ever take addresses, and `static mut`
drags in `static_mut_refs` for no benefit.

### 5.4.3 The implementation

```rust
#[inline]
unsafe fn reset_data() {
    let src   = &raw const __sidata;                 // flash (LMA)
    let dst   = &raw const __sdata as *mut u32;      // RAM (VMA)
    let end   = &raw const __edata as *const u32;
    let count = (end as usize - dst as usize) / 4;
    unsafe { copy_nonoverlapping(src, dst, count) }
}

#[inline]
unsafe fn reset_bss() {
    let p     = &raw const __sbss as *mut u32;
    let end   = &raw const __ebss as *const u32;
    let count = (end as usize - p as usize) / 4;
    unsafe { p.write_bytes(0, count) }
}
```

`copy_nonoverlapping` and `write_bytes` are `memcpy`/`memset`. They link on bare
metal — `compiler_builtins` supplies `__aeabi_memcpy4` / `__aeabi_memclr4`.

Both counts are in **elements** (`u32`), which is why both divide by 4. The
`ALIGN(4)` in the linker script is what makes that division exact.

Verified codegen:

```asm
movw r1,#0x4  ; movt r1,#0x2000    ; r1 = 0x20000004  (__edata)
movw r0,#0x0  ; movt r0,#0x2000    ; r0 = 0x20000000  (__sdata)
subs r2, r1, r0                    ; 4 bytes
movw r1,#0x570; movt r1,#0x1000    ; r1 = 0x10000570  (__sidata)
bl   __aeabi_memcpy4
```

LLVM cancels the `/4` against memcpy's element scaling and passes the byte count
straight through.

### 5.4.4 Two traps in the count expression

> **Trap 1 — the wrong pointer pair.** `end.offset_from(src)` subtracts a
> **flash** address from a **RAM** address:
>
> ```
> __sidata (flash) = 0x100001d8      (__edata - __sidata)/4 = 67,108,747 words
> __sdata  (RAM)   = 0x20000000      (__edata - __sdata )/4 = 1 word
> __edata  (RAM)   = 0x20000004
> ```
>
> That memcpy overwrites all 520 kB of SRAM — including the live stack — and
> then walks off the end into undecoded address space. The count must come from
> **`dst`**.

> **Trap 2 — `.try_into().unwrap()` costs 3.6 kB.** It pulls in `core::fmt` and
> the panic message strings, and puts a **panic path inside the reset handler**
> — running before `.bss` is zeroed.
>
> | | `.text` | `.rodata` | total |
> |---|---|---|---|
> | `.try_into().unwrap()` | 4400 B | 284 B | **4684 B** |
> | `(end as usize - dst as usize) / 4` | 1100 B | 0 B | **1100 B** |
>
> It is also checking for a negative distance between two linker symbols the
> linker itself guarantees are ordered — a branch that can never be taken, paid
> for anyway.

`offset_from` between pointers derived from two distinct extern statics is also
UB in Rust's model, independent of the value being wrong. `usize` address
arithmetic sidesteps both problems.

### 5.4.5 Testing it

An empty `.data`/`.bss` means both loops run **zero iterations** and a clean
build proves nothing. Add:

```rust
#[used] #[unsafe(no_mangle)] static mut DATA_TEST: u32      = 0xDEAD_BEEF;
#[used] #[unsafe(no_mangle)] static mut BSS_TEST:  [u32; 4] = [0; 4];
```

Then confirm `.data` is 4 bytes with a *different* VMA and LMA, `.bss` is 16
bytes, and `__sidata` matches `.data`'s LMA.

## 5.5 Debug versus release

| | debug | release |
|---|---|---|
| `.text` | 3988 B | 1392 B |
| panic / `precondition_check` symbols | 8 | **0** |

Debug builds insert `core::ptr::{read_volatile, write_volatile,
copy_nonoverlapping, write_bytes}::precondition_check` calls that validate
alignment and **panic on failure** — inside `OnReset`, before `.bss` is zeroed.

They cannot actually fire here (the pointers are aligned), but **flash
`--release`** for first bring-up: a third the size and no panic path in startup.

## 5.6 Known deferrals

Cheap, not blocking, and worth doing before hardware debugging gets hard:

- **`SHCSR` fault enables.** `MEMFAULTENA`/`BUSFAULTENA`/`USGFAULTENA`/
  `SECUREFAULTENA` all reset to 0, so every configurable fault escalates to
  HardFault — which means the `t[4]`-`t[7]` handlers are currently dead code
  that can never be entered.
- **`MSPLIM_S`.** Never initialised, and its value at bootrom exit is not
  documented. Writing `0` costs three instructions and closes the question.
- **`cpsid i`** at the top of `OnReset` — harmless on a cold boot (nothing can
  fire), cheap hardening for a warm reboot.
- **Swap `enable_fpu` / `reset_vtor`.** Until VTOR is written, faults vector
  through whatever the bootrom left. Swapping shrinks that window to zero.
