---
document_type: "Tutorial Chapter — The Reset Handler"
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 6 of 9
revision: C
effective_date: 2026-08-29
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: chapters 01-05
sources: RP2350 datasheet §3.4 (PDF p37), §3.6 (PDF p101), §3.7.4.8.1 (PDF p138), §3.7.5 (PDF p149, Table 120), §5.2.2 (Table 450, PDF p367-370), §5.9.5.1 (PDF p427); Tables 192, 201, 208, 229, 362
creates: firmware/pico2/src/lib.rs — through §6.5's listing;
  demo/src/main.rs — rewritten to declare its entry point, with one placeholder
---

# Chapter 06 — The Reset Handler

The bootrom loads SP from vector word 0 and jumps to word 1 (chapter 05 §5.6).
From that instruction on, nothing is set up for you. Everything a hosted
program's runtime would have done before `main` now happens in one function,
written by hand, in the order the hardware requires — and this chapter also
builds the bridge that function crosses to reach the application: the
`entry!` macro and the `__rustos_main` symbol.

## 6.1 What it must do, in order

The whole thing, verbatim from `firmware/pico2/src/lib.rs`:

```rust
#[unsafe(no_mangle)]
pub extern "C" fn OnReset() -> ! {
    unsafe {
        enable_fpu();
        reset_vtor();
        reset_data();
        reset_bss();
        __rustos_main()
    }
}
```

Four setup steps, then the application. The order is not arbitrary:

| Step | Must happen before | Why |
|---|---|---|
| `enable_fpu` | any floating-point instruction executes | `thumbv8m.main-none-eabihf` is a hard-float target; the compiler may emit FP anywhere |
| `reset_vtor` | any exception is taken | until VTOR is written, exceptions dispatch through whatever table the bootrom left |
| `reset_data` | any initialised `static` is read | the RAM copy does not exist yet |
| `reset_bss` | any zero-initialised `static` is read | SRAM does not power up as zeros |

`-> !` is not decoration. The hardware enters this function by loading its
address into the PC — there is no caller and no return address, so if it returns
the CPU pops garbage into the PC. The last expression, `__rustos_main()`, is a
call to a function that is itself declared `-> !` (§6.4.4), so there is no
trailing `loop {}`: "the application never returns" is enforced by the type
system rather than by a fallback. All four helpers are `#[inline]`: in
`--release` they vanish into `OnReset` itself (§6.5), and in a debug build they
are real functions reached by `bl` (§6.8).

## 6.2 Enabling the FPU

### 6.2.1 Why it is off

The Cortex-M33 has a coprocessor port carrying up to 64 bits per cycle to
closely-coupled hardware. The single-precision FPU sits on it, and RP2350 adds
three coprocessors of its own (PDF p37):

| Coprocessor | Block |
|---|---|
| CP0 | GPIO coprocessor (GPIOC), §3.6.1 |
| CP4 / CP5 | Secure and Non-secure double-precision coprocessor (DCP), §3.6.2 |
| CP7 | redundancy coprocessor (RCP), §3.6.3 |
| CP10 / CP11 | the standard Arm single-precision FPU |

Section 3.6 states the rule (PDF p101):

> Before accessing a coprocessor from Secure code, that coprocessor must first
> be enabled by setting the corresponding bit in the CPACR.

Every CPACR field resets to `0x0` (Table 229, PDF p194) — access denied. The
gate is *access control*, not power saving: an RTOS that will not spend a
context switch saving 32 FP registers leaves the FPU off so any task touching FP
traps. You are not that RTOS, so you turn it on.

### 6.2.2 CPACR

**`0xe000ed88`** = `PPB_BASE` + offset `0x0ed88`. `PPB_BASE` is `0xe0000000`
(Table 15, PDF p35; §3.7.5, PDF p149: *"The Arm Cortex-M33 registers start at a
base address of 0xe0000000"*), and `0x0ed88` is CPACR's entry in Table 120,
which starts on PDF p149 and reaches CPACR on PDF p155.

CPACR is one 2-bit field per coprocessor (Table 229, PDF p194). The table has
a row for CP0 through CP7 plus CP10 and CP11; the four RP2350 does not populate
(CP1, CP2, CP3, CP6) are elided here — they are `RW`, reset `0x0`, like the
rest:

| Bits | Field | Type | Reset |
|---|---|---|---|
| 23:22 | CP11 | RW | 0x0 |
| 21:20 | CP10 | RW | 0x0 |
| 15:14 | CP7 | RW | 0x0 |
| 11:10 | CP5 | RW | 0x0 |
| 9:8 | CP4 | RW | 0x0 |
| 1:0 | CP0 | RW | 0x0 |

The *encoding* of each field is ARMv8-M architecture and is **not stated in the
RP2350 datasheet**: `0b00` denied (the reset value), `0b01` privileged only,
`0b10` reserved, `0b11` full access.

CP10 and CP11 are not independent. Table 229, on CP11:

> If the value of this bit is not programmed to the same value as the CP10
> field, then the value is UNKNOWN

So the value you want is `0b11` in both: `(0b11 << 20) | (0b11 << 22)` =
**`0x00f00000`**.

### 6.2.3 The code

Verbatim from `lib.rs` (doc comments elided, inline comments kept):

```rust
const PPB_BASE: usize = 0xE000_0000;

const CPACR: *mut u32 = (PPB_BASE + 0x0ED88) as *mut u32;

const CPACR_FPU_FULL: u32 = (0b11 << 20) | (0b11 << 22); // == 0x00F0_0000

#[inline]
unsafe fn enable_fpu() {
    unsafe {
        let current = CPACR.read_volatile(); // READ
        let updated = current | CPACR_FPU_FULL; // MODIFY — preserves CP0/CP4/CP5/CP7
        CPACR.write_volatile(updated); // WRITE
        core::arch::asm!("dsb", "isb", options(nostack, preserves_flags));
    }
}
```

> **Silent-failure trap.** A plain `CPACR.write_volatile(0x00F0_0000)` zeroes
> CP0, CP4, CP5 and CP7 in the same instruction — switching off the GPIO
> coprocessor, both double-precision units and the redundancy coprocessor. You
> get a working FPU and four mysteriously dead accelerators, with no fault and
> nothing in the disassembly that looks wrong. Read-modify-write.
>
> On a *cold* reset every field is already `0x0`, so RMW is bit-identical to a
> plain write. It matters because the CPACR state the bootrom hands you is not
> documented in Table 450 (PDF p367-370), and a warm reboot is not a cold reset.

`dsb` waits for the store to reach the point of coherency; `isb` flushes the
pipeline, so everything after it is decoded under the new configuration. Without
the `isb`, an FP instruction already in flight was decoded while CP10/CP11 still
read as denied.

### 6.2.4 Failure mode

Section 3.7.4.8.1 (PDF p138) is exact about skipping this:

> If any coprocessor instruction is executed when the corresponding coprocessor
> is disabled in the CPACR/NSACR register, the Cortex-M33 processor always
> attempts to take a No coprocessor (NOCP) UsageFault exception.

The status bit is `UFSR_NOCP`, bit 19 of the CFSR block (PDF p187). But
`SHCSR.USGFAULTENA` (bit 18) resets to `0x0` (Table 208, PDF p186), so UsageFault
is not enabled and the fault escalates straight to **HardFault**. On a hard-float
target that is the most likely first-boot failure, and the only evidence is that
you are sitting in `OnHardFault`'s `loop {}`.

## 6.3 Setting VTOR

**`0xe000ed08`** = `PPB_BASE + 0x0ed08` (Table 120, PDF p154; description
PDF p182). When exception *N* fires, the core computes `VTOR + (N * 4)`, loads
the word there and jumps to it. That is the entire dispatch mechanism. VTOR's
only field is `TBLOFF`, bits 31:7, **reset value `0x0000000`** (Table 201,
PDF p183); bits 6:0 being reserved is also the 128-byte alignment requirement,
which the linker script's `ALIGN(512)` more than satisfies (chapter 04 §4.3).

```rust
const VTOR: *mut u32 = (PPB_BASE + 0x0ED08) as *mut u32;

#[inline]
unsafe fn reset_vtor() {
    unsafe {
        VTOR.write_volatile(&raw const VECTOR_TABLE as u32);
        core::arch::asm!("dsb", "isb", options(nostack, preserves_flags));
    }
}
```

Two deliberate choices. **Take the address of the static**, not a literal
`0x10000000` — identical codegen (§6.5 shows it), but the source stays correct
if the table moves. And **plain write, not read-modify-write**: `TBLOFF` is bits
31:7 and bits 6:0 are reserved, so the whole register *is* the address and there
is nothing to preserve. The general rule, which chapter 07 §7.2 develops: RMW
when a register is a bag of unrelated fields, plain write when it is one value.
Applying RMW everywhere out of caution is its own bug — it makes
write-1-to-clear registers behave very strangely.

### 6.3.1 Does the bootrom set it for you?

Table 450 (PDF p367-370) is the processor-controlled boot sequence, and it
mentions VTOR exactly once, in the **Core 1 Wait** step (PDF p367):

> Outcome: Set Secure main sp and VTOR, then jump into the entry point provided.

This firmware takes **Try Flash Boot** on core 0, whose outcome reads only
"Enter flash image in the manner specified by its image definition" (PDF p369).
That image definition is the minimum Arm `IMAGE_DEF` of chapter 05 §5.3, and
§5.9.5.1 (PDF p427) spells out its entry semantics:

> Since the above block does not specify an explicit entry point, the bootrom
> will assume the binary starts with a Cortex-M vector table, and enter via the
> reset handler and initial stack pointer specified in that table (offsets +4
> and +0 bytes into the table).

SP and PC are named. VTOR is not.

**Inferred:** on the core-0 flash path the bootrom reads two words out of your
table without programming VTOR to point at it, so VTOR still holds whatever the
bootrom last left there. That inference is why `reset_vtor()` exists — two
instructions is cheaper than proving what the bootrom did.

### 6.3.2 Are the barriers required?

Strictly, no. At this point in `OnReset` no exception can be taken:

- `NVIC_ISER0` and `NVIC_ISER1` reset to `0x00000000` (Table 192, PDF p180), so
  no external interrupt is enabled.
- `SHCSR`'s four fault enables — `MEMFAULTENA`, `BUSFAULTENA`, `USGFAULTENA`,
  `SECUREFAULTENA`, bits 16 through 19 — all reset to `0x0` (Table 208,
  PDF p186).
- `M33_EPPB: NMI_MASK0` resets to `0x00000000` (Table 362, PDF p233), so nothing
  is routed to NMI either.

**Inferred:** the risk of omitting the barriers *here* is therefore zero rather
than small — the window between the store and the first possible vector fetch
contains no vector fetch. Keeping them costs two instructions and is the right
habit for every later VTOR write, when that reasoning no longer holds.

## 6.4 `.data` copy, `.bss` zero, and the way out

Chapter 04 §4.5 and §4.6 exported five symbols from the linker script; this is
where the firmware consumes them. Both terms are in play: **VMA** (virtual
memory address, where a section runs) and **LMA** (load memory address, where
its bytes sit in the image).

### 6.4.1 Why `.bss` must be zeroed

SRAM does not power on as zeros. It powers on as whatever the silicon settles
into — often stable enough across resets to fool you for an afternoon.

Rust guarantees that any `static` you declare as zero — say
`static COUNTER: AtomicU32 = AtomicU32::new(0)`, which lands in `.bss` — reads
back as `0`. On a hosted OS the loader upholds that; here exactly one thing
does, and it is your zeroing loop. Skip it and you read uninitialised memory
through a type system that insists the value is zero. `.data` is the same
argument with a different mechanism: its initial values live in flash at the LMA
(`__sidata`), its VMA is in RAM, and nothing moves them across unless you do.

This is not hypothetical in this workspace: the finished image's `.bss` holds
one real static — `BOARD_CREATED`, the `AtomicBool` behind `Board::take` in the
`api` crate (chapter 08 §8.12) — and `Board::take`'s once-only guarantee is
only as good as the zero this loop writes.

### 6.4.2 The symbol declarations

Five more linker symbols to declare. Add this block directly under the
`unsafe extern "C" { static _stack_top: u32; }` you wrote in chapter 05 §5.6.2
(the tree carries a `///` doc comment on each; code lines verbatim):

```rust
unsafe extern "C" {
    static __sidata: u32;
    static __sdata: u32;
    static __edata: u32;
    static __sbss: u32;
    static __ebss: u32;
}
```

Two separate `extern` blocks, as written — the tree keeps `_stack_top` in its
own block because it belongs to the vector table, and groups these five because
they belong to the two loops below. All six are `static`, not `static mut`: you
only ever take their *addresses*, the "value" of a linker symbol is a fiction,
and `static mut` would drag in the `static_mut_refs` lint for no benefit.
`&raw const NAME` gives you the address without ever forming a reference to
memory that may not be readable.

### 6.4.3 The implementation

`reset_data` calls `copy_nonoverlapping`, which is a free function in
`core::ptr` and has to be imported. Chapter 01 §1.9 left the file's `use` line
reading `use core::panic::PanicInfo;`; widen it now to what the tree has:

```rust
use core::{panic::PanicInfo, ptr::copy_nonoverlapping};
```

`write_bytes` in `reset_bss` needs no import — it is an inherent method on raw
pointers, always in scope. The asymmetry is invisible from the call sites, and
the two functions below sit a few lines apart, so it is worth naming: a free
function needs a path, a method does not.

```rust
#[inline]
unsafe fn reset_data() {
    let src = &raw const __sidata; // flash (LMA)
    let dst = &raw const __sdata as *mut u32; // RAM (VMA)
    let end = &raw const __edata as *const u32;
    let count = (end as usize - dst as usize) / 4;
    unsafe { copy_nonoverlapping(src, dst, count) }
}

#[inline]
unsafe fn reset_bss() {
    let p = &raw const __sbss as *mut u32;
    let end = &raw const __ebss as *const u32;
    let count = (end as usize - p as usize) / 4;
    unsafe { p.write_bytes(0, count) }
}
```

`copy_nonoverlapping` and `write_bytes` are `memcpy` and `memset`. They link on
bare metal because `compiler_builtins` supplies `__aeabi_memcpy4` and
`__aeabi_memclr4`; §6.5 disassembles the calls to both. Both counts are in
**elements** (`u32`), which is why both divide by 4; the `ALIGN(4)` on `.data`
and `.bss` in the linker script makes that division exact.

### 6.4.4 The way out: `entry!` and `__rustos_main`

`OnReset` ends by calling the application — and the application lives in a
different crate, `demo`, which this library cannot name: `pico2` must not
depend on the binary built from it, or nothing else could ever be built from
it. So the call goes through a **named symbol** instead of a Rust path. The
library declares the symbol without defining it, verbatim from `lib.rs`:

```rust
// The application entry point. This symbol is not defined anywhere in this
// crate — it is defined by the binary that links against it, via `entry!`.
// This is the Rust equivalent of a C forward declaration, with the same
// property that the linker matches on name alone; see `entry!` for how the
// signature is nonetheless checked.
unsafe extern "Rust" {
    fn __rustos_main() -> !;
}
```

An `extern` block is a *declaration*: it tells the compiler "a function of
this name and signature will exist at link time", and the linker later
resolves the name against whatever object file defines it. That is exactly how
C crosses file boundaries, and it carries C's defect: **the declaration is not
checked against the definition.** Declare `fn __rustos_main() -> !` here,
define `fn __rustos_main(x: u32) -> u32` over there, and the linker happily
matches them on name; at runtime the callee reads an argument nobody passed.

The definition comes from the application, through a macro the library
exports — verbatim from `lib.rs`, tree comment included:

```rust
#[macro_export]
macro_rules! entry {
    ($f:path) => {
        #[unsafe(no_mangle)]
        pub extern "Rust" fn __rustos_main() -> ! {
            // Type check: rejects any signature other than fn() -> !.
            let f: fn() -> ! = $f;
            f()
        }
    };
}
```

Expanded in the application, `entry!(main)` defines the real `__rustos_main`:
a shim that calls whatever function you named. The one line doing the safety
work is the coercion:

```rust
let f: fn() -> ! = $f;
```

Coercing the named function to a typed function pointer forces the compiler to
prove the signature matches **before** the symbol is emitted — the hole in the
C-style extern declaration, closed. Why the shim exists at all: on a `no_std`
target you cannot have an ordinary `fn main`. rustc's normal `main` is a shim
that calls `lang_start`, which only `std` provides — the error is a flat
`using 'fn main' requires the standard library` — so the binary declares
`#![no_main]` and the runtime reaches user code through this named symbol
instead. (`#[macro_export]` puts the macro at the crate root, so
`pico2::entry!(..)` works with no accompanying `use`.)

Now rewrite `demo/src/main.rs`. The `use pico2 as _;` placeholder from chapter
01 §1.9 goes — `entry!` names the crate, which is what the placeholder import
was for — and the whole file becomes:

```rust
#![no_std]
#![no_main]

pico2::entry!(main);

// PLACEHOLDER — chapter 08 §8.12 replaces this body
fn main() -> ! {
    loop {}
}
```

`main` is back — as an ordinary, private, mangled function that the shim
calls, not as a language entry point. And the type check is live. Change the
placeholder's signature to `fn main()` and the build fails in the application
crate, pointing at the invocation:

```
error[E0308]: mismatched types
 --> demo/src/main.rs:4:1
  |
4 | pico2::entry!(main);
  | ^^^^^^^^^^^^^^^^^^^
  | |
  | expected fn pointer, found fn item
  | expected due to this
  |
  = note: expected fn pointer `fn() -> !`
                found fn item `fn() -> () {main}`
  = note: this error originates in the macro `pico2::entry` (in Nightly builds, run with -Z macro-backtrace for more info)
```

That is the C defect surfacing as a compile error instead of a runtime
mystery: `main` must diverge because `OnReset` diverges, and the chain of
`-> !` from `main` through `__rustos_main` to `OnReset` is checked end to end.

## 6.5 The file so far, and what it compiles to

`OnReset` now has all four helpers and the extern declaration under it. Here is
the whole of `firmware/pico2/src/lib.rs` as this chapter leaves it, continuing
chapter 05 §5.9 — the tree's code lines with its long doc comments elided. Only
the two `pub mod` lines of chapter 08 are missing relative to the tree:

```rust
#![no_std]

use core::{panic::PanicInfo, ptr::copy_nonoverlapping};

#[used]
#[unsafe(link_section = ".boot_info")]
static BOOT_INFO: [u32; 5] = [
    0xffffded3,
    0x10210142,
    0x000001ff,
    0x00000000,
    0xab123579,
];

unsafe extern "C" {
    static _stack_top: u32;
}

unsafe extern "C" {
    static __sidata: u32;
    static __sdata: u32;
    static __edata: u32;
    static __sbss: u32;
    static __ebss: u32;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
#[derive(Clone, Copy)]
union Vector {
    handler: unsafe extern "C" fn(),
    reset: unsafe extern "C" fn() -> !,
    stack_top: *const u32,
    reserved: u32,
}

unsafe impl Sync for Vector {}

const PPB_BASE: usize = 0xE000_0000;

const CPACR: *mut u32 = (PPB_BASE + 0x0ED88) as *mut u32;

const VTOR: *mut u32 = (PPB_BASE + 0x0ED08) as *mut u32;

const CPACR_FPU_FULL: u32 = (0b11 << 20) | (0b11 << 22); // == 0x00F0_0000

#[inline]
unsafe fn enable_fpu() {
    unsafe {
        let current = CPACR.read_volatile(); // READ
        let updated = current | CPACR_FPU_FULL; // MODIFY — preserves CP0/CP4/CP5/CP7
        CPACR.write_volatile(updated); // WRITE
        core::arch::asm!("dsb", "isb", options(nostack, preserves_flags));
    }
}

#[inline]
unsafe fn reset_vtor() {
    unsafe {
        VTOR.write_volatile(&raw const VECTOR_TABLE as u32);
        core::arch::asm!("dsb", "isb", options(nostack, preserves_flags));
    }
}

#[inline]
unsafe fn reset_data() {
    let src = &raw const __sidata; // flash (LMA)
    let dst = &raw const __sdata as *mut u32; // RAM (VMA)
    let end = &raw const __edata as *const u32;
    let count = (end as usize - dst as usize) / 4;
    unsafe { copy_nonoverlapping(src, dst, count) }
}

#[inline]
unsafe fn reset_bss() {
    let p = &raw const __sbss as *mut u32;
    let end = &raw const __ebss as *const u32;
    let count = (end as usize - p as usize) / 4;
    unsafe { p.write_bytes(0, count) }
}

unsafe extern "Rust" {
    fn __rustos_main() -> !;
}

#[macro_export]
macro_rules! entry {
    ($f:path) => {
        #[unsafe(no_mangle)]
        pub extern "Rust" fn __rustos_main() -> ! {
            // Type check: rejects any signature other than fn() -> !.
            let f: fn() -> ! = $f;
            f()
        }
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn OnReset() -> ! {
    unsafe {
        enable_fpu();
        reset_vtor();
        reset_data();
        reset_bss();
        __rustos_main()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn DefaultHandler() {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn OnHardFault() {
    loop {}
}

#[used]
#[unsafe(link_section = ".vector_table")]
static VECTOR_TABLE: [Vector; 68] = {
    let mut t = [Vector { handler: DefaultHandler }; 68];
    t[0] = Vector { stack_top: &raw const _stack_top };
    t[1] = Vector { reset: OnReset };
    t[3] = Vector { handler: OnHardFault };
    t[8] = Vector { reserved: 0 };
    t[9] = Vector { reserved: 0 };
    t[10] = Vector { reserved: 0 };
    t[13] = Vector { reserved: 0 };
    t
};
```

`cargo build --release`, staged, with `demo/src/main.rs` as §6.4.4 left it:

```
Memory region         Used Size  Region Size  %age Used
           FLASH:        1400 B         4 MB      0.03%
             RAM:          8 KB       520 KB      1.54%
```

Up from §5.9's 300 B, all of it `.text`: `.text` is now `0x454`, and the
`compiler_builtins` memcpy/memset routines the two loops call are linked in.

Here is the whole of `OnReset` as actually compiled, run against the staged
build above:

```
llvm-objdump -d -C --no-show-raw-insn --disassemble-symbols=OnReset \
  target/thumbv8m.main-none-eabihf/release/demo
```

`-C` demangles (chapter 01 §1.7.1); `OnReset` needs no demangling itself, but
`bl` targets in a disassembly generally do.

```asm
10000130 <OnReset>:
10000130:      	push	{r7, lr}
10000132:      	mov	r7, sp
10000134:      	movw	r0, #0xed88
10000138:      	movt	r0, #0xe000
1000013c:      	ldr	r1, [r0]
1000013e:      	orr	r1, r1, #0xf00000
10000142:      	str	r1, [r0]
10000144:      	movw	r1, #0x0
10000148:      	dsb	sy
1000014c:      	isb	sy
10000150:      	movt	r1, #0x1000
10000154:      	str	r1, [r0, #-128]
10000158:      	movw	r1, #0x0
1000015c:      	movw	r0, #0x0
10000160:      	movt	r1, #0x2000
10000164:      	movt	r0, #0x2000
10000168:      	subs	r2, r1, r0
1000016a:      	movw	r1, #0x578
1000016e:      	dsb	sy
10000172:      	isb	sy
10000176:      	movt	r1, #0x1000
1000017a:      	bl	0x10000246 <__aeabi_memcpy4>
1000017e:      	movw	r1, #0x0
10000182:      	movw	r0, #0x0
10000186:      	movt	r1, #0x2000
1000018a:      	movt	r0, #0x2000
1000018e:      	subs	r1, r1, r0
10000190:      	bl	0x10000198 <__aeabi_memclr4>
10000194:      	bl	0x10000124 <__rustos_main>
```

Read that against the calls in §6.1: all four helpers are inlined, and there is
not a single `bl` into `enable_fpu` or `reset_vtor` — the only three `bl`s go
to the two `compiler_builtins` routines and to the application. Three things
are worth stopping on.

**`str r1, [r0, #-128]` is the VTOR write.** `r0` still holds `0xe000ed88` from
the CPACR sequence, and `0xe000ed88 - 0x80` = `0xe000ed08`. LLVM noticed the two
registers are 128 bytes apart and reused the pointer instead of materialising a
second address. That one instruction is the argument for writing
`PPB_BASE + 0x0ED88` and `PPB_BASE + 0x0ED08` as two constants sharing a base
rather than as two opaque literals: the compiler finds the shared base and
spends two instructions instead of four. The `movt r1, #0x1000` just before it
is `&raw const VECTOR_TABLE` — `0x10000000`, exactly the literal you did not
write.

**The barriers are not where you put them.** The `dsb sy` / `isb sy` at
`0x10000148` has `movw r1, #0x0` in front and `movt r1, #0x1000` behind: LLVM
hoisted half of the VTOR address materialisation *above* the FPU barriers,
because `movw`/`movt` are register-only and the barriers order memory, not
arithmetic. It happens again at `0x1000016e`, where the `movw r1, #0x578` that
loads `__sidata` is split around the VTOR barriers. The ordering you asked for
is there; the instruction stream around it is not the one you wrote.

**`movw r1, #0x578` / `movt r1, #0x1000` is `__sidata` = `0x10000578`,** which
matches `llvm-nm -n` on the same staged build:

```
10000578 A __sidata
20000000 B __ebss
20000000 R __edata
20000000 B __sbss
20000000 R __sdata
```

`__sdata` and `__edata` are the same address, so `subs r2, r1, r0` yields zero
and `memcpy4` copies nothing; same for `memclr4` at this stage (§6.7). Note
also what LLVM did with the `/ 4`: there is no `lsr` anywhere. It cancelled the
division against `memcpy`'s element scaling and passes a byte count straight
through.

The last instruction is the handoff: `bl 0x10000124 <__rustos_main>` — the
shim `entry!` expanded to in `demo`, which in this staged build is an inlined
`loop {}` three instructions long.

### 6.5.1 What changes when chapter 08 fills the application in

The numbers in the listing above are functions of how much code is in the
image, and chapter 08 adds the GPIO driver, the `api` layer and a real `main`.
Re-run the same two commands on the finished firmware and you get the same
instructions with different operands:

| | end of chapter 06 | finished firmware |
|---|---|---|
| `.text` | `0x454` | `0x1888` |
| `OnReset` at | `0x10000130` | `0x10000306` |
| `__sidata` | `0x10000578` | `0x10001b84` |
| `__aeabi_memcpy4` | `0x10000246` | `0x10001910` |
| `__aeabi_memclr4` | `0x10000198` | `0x100015cc` |
| `.bss` count materialisation | `movw r1, #0x0` | `movw r1, #0x4` |
| last instruction | `bl <__rustos_main>` | `bl <__rustos_main>` |

`__sidata` is `LOADADDR(.data)`, which sits immediately after `.rodata`, which
sits immediately after `.text` — so it moves when `.text` and `.rodata` grow.
The `.bss` row is the interesting one: in the finished image `__ebss` is
`0x20000004`, because the `api` crate's `BOARD_CREATED` flag landed in `.bss`
(§6.4.1), and the zero loop that moves nothing today moves exactly one word
there. This comparison is worth doing once: it is the difference between
believing the reset handler works and having watched its operands track the
image around it.

## 6.6 Two traps in the count expression

> **Silent-failure trap — the wrong pointer pair.** `end.offset_from(src)`
> subtracts a **flash** address from a **RAM** address. With the test statics of
> §6.7 in the staged build:
>
> ```
> __sidata (flash) = 0x10000578     (__edata - __sidata)/4 = 67,108,515 words
> __sdata  (RAM)   = 0x20000000     (__edata - __sdata )/4 = 1 word
> __edata  (RAM)   = 0x20000004
> ```
>
> That `memcpy` writes 268,434,060 bytes: all 520 kB of SRAM including the live
> stack you are standing on, then off the end into undecoded address space. It
> compiles, it type-checks, and the only symptom is a board that does nothing.
> The count must come from **`dst`**.

The second trap is a size trap, and it is why the code uses `usize` subtraction
rather than the idiomatic `offset_from`. Replace both count expressions with

```rust
// PROPOSED — not in the tree today
let count: usize = unsafe{end.offset_from(dst)}.try_into().unwrap();
```

and rebuild `--release` at this chapter's stage:

| | `.text` | `.rodata` | FLASH total |
|---|---|---|---|
| `end.offset_from(dst).try_into().unwrap()` | 4408 B | 296 B | **4996 B** |
| `(end as usize - dst as usize) / 4` (shipping) | 1108 B | 0 B | **1400 B** |

3596 bytes — two and a half times the entire staged firmware — for a check that
cannot fail. `llvm-nm` on that build shows why: `core::fmt::Formatter::pad`,
`core::fmt::write`, `TryFromIntError`'s `Debug` impl and
`core::panicking::panic_fmt` all get linked in, because `unwrap()` on a
`Result<_, TryFromIntError>` has to be able to format the error. It also puts a
**panic path inside the reset handler**, running before `.bss` is zeroed, to
check for a negative distance between two linker symbols the linker itself
emitted in order. There is a correctness argument too: `offset_from` between
pointers derived from two *distinct* extern statics is UB in Rust's memory
model, independent of whether the number comes out right. `usize` address
arithmetic sidesteps both problems and is smaller. (The finished image ends up
carrying the `core::fmt` machinery anyway — chapter 08 §8.13 shows the
application pulling it in through its own `unwrap`s — but there it runs after
startup, from a zeroed `.bss`, which is a different bargain from panicking
inside the reset handler.)

## 6.7 Testing the loops

> **Silent-failure trap.** At this chapter's stage `.data` and `.bss` are
> **both zero-length** — look again at §6.5's symbol dump, where `__sdata`,
> `__edata`, `__sbss` and `__ebss` are all `0x20000000`. Both loops run zero
> iterations. `__aeabi_memcpy4` and `__aeabi_memclr4` are still linked, still
> called, and still do nothing. A clean build proves **nothing at all** about
> this code: a completely broken `reset_bss` ships green today, and starts
> corrupting `Board::take` in chapter 08, when `.bss` grows its first real
> word.

The only way to see the loops work is to give them something to move. Add two
statics to `demo/src/main.rs` — these are not in the tree:

```rust
// PROPOSED — not in the tree today
#[used] #[unsafe(no_mangle)] static mut DATA_TEST: u32      = 0xDEAD_BEEF;
#[used] #[unsafe(no_mangle)] static mut BSS_TEST:  [u32; 4] = [0; 4];
```

Rebuild `--release` and the sections are no longer empty
(`llvm-objdump --section-headers`, note the LMA column):

```
Idx Name            Size     VMA      LMA      Type
  3 .text           00000454 10000124 10000124 TEXT
  4 .rodata         00000000 10000578 10000578 DATA
  5 .data           00000004 20000000 10000578 DATA
  6 .bss            00000010 20000004 20000004 BSS
  7 .stack          00002000 20000018 20000018 BSS
```

Four things to check, all visible above: `.data` is 4 bytes; its VMA
(`0x20000000`) and LMA (`0x10000578`) **differ**, which is the entire reason
`reset_data` exists; that LMA equals `__sidata`, still `0x10000578`; and `.bss`
is 16 bytes, with `.stack` pushed up behind it to `0x20000018`. The initialiser
really is in flash — `llvm-objdump -s -j .data` reads the bytes at the LMA:

```
Contents of section .data:
 20000000 efbeadde                             ....
```

`0xdeadbeef`, little-endian. And `OnReset` now passes real counts; compare
these four lines against `0x10000158`-`0x1000018e` in §6.5:

```asm
10000158:      	movw	r1, #0x4          ; __edata = 0x20000004
1000015c:      	movw	r0, #0x0          ; __sdata = 0x20000000
1000017e:      	movw	r1, #0x14         ; __ebss  = 0x20000014
10000182:      	movw	r0, #0x4          ; __sbss  = 0x20000004
```

Zero became 4 and 16. That is the test. Delete the statics afterwards: the
linker report drops from 1404 B back to 1400 B of flash, and RAM from 8216 B
back to 8 kB.

## 6.8 Debug versus release

Same source, two builds, measured on 2026-08-29 — on the **finished** tree,
because the point concerns the shipping image:

| | debug | release |
|---|---|---|
| `.text` | 9776 B (`0x2630`) | 6280 B (`0x1888`) |
| `.rodata` | 3112 B (`0xc28`) | 472 B (`0x1d8`) |
| FLASH total, from the linker report | 13180 B | 7044 B |
| `precondition_check` symbols | 5 | **0** |
| `core::panicking::*` symbols | 7 | 2 |

The five `precondition_check` symbols are
`core::ptr::{write_bytes, read_volatile, write_volatile, copy_nonoverlapping}`
and `core::hint::unreachable_unchecked`. They validate alignment and **panic on
failure** — inside `OnReset`, before `.bss` is zeroed. Here is `reset_data` in
the debug build, a real function because `#[inline]` is advisory:

```
llvm-objdump -d -C --no-show-raw-insn --disassemble-symbols=pico2::reset_data \
  target/thumbv8m.main-none-eabihf/debug/demo
```

`reset_data` is not `#[no_mangle]`, so the symbol to ask for is the demangled
path `pico2::reset_data` and `-C` is what makes that spelling match (chapter 01
§1.7.1). Note the path: `debug`, not `release`. Abridged to the two branches
that matter:

```asm
1000097e <pico2::reset_data>:
    ...
100009a2:      	cmp	r0, r1
100009a4:      	blo	0x100009fc <pico2::reset_data+0x7e>
    ...
100009e6:      	bl	0x10001034 <core::ptr::copy_nonoverlapping::precondition_check>
100009f4:      	bl	0x100026b6 <__aeabi_memcpy4>
100009f8:      	add	sp, #0x30
100009fa:      	pop	{r7, pc}
100009fc:      	movw	r0, #0x2c38
10000a00:      	movt	r0, #0x1000
10000a04:      	bl	0x10001e2e <core::panicking::panic_const::panic_const_sub_overflow>
```

Two panic paths in one small function. The `blo` at `0x100009a4` is the
**overflow-checked subtraction** `end as usize - dst as usize`: debug builds
enable overflow checks, so the subtraction gets a branch to
`panic_const_sub_overflow`. Neither path can fire here — the pointers are
aligned and the linker emitted the symbols in order — but both are compiled in,
they account for a good share of the 6.1 kB difference, and they route into
your `#[panic_handler]`'s `loop {}`, where a debugger is the only way to tell.

One honest asymmetry in the release column: the panic machinery is stripped
from **startup**, not from the image. `--release` removes every
`precondition_check` and every panic path in `OnReset`'s inlined body — but
`core::panicking::panic` and `panic_fmt` are still in the release image,
because the finished application calls `unwrap()` (chapter 08 §8.13). The
difference is *where* they can run: after `.bss` is zeroed and the machine is
initialised, not in the middle of bringing it up. Flash `--release` for first
bring-up: half the size, and no panic path in startup.

## 6.9 Known deferrals

None are blocking, all are cheap, and all get more annoying once you are
debugging real hardware.

- **`SHCSR` fault enables.** Bits 16 to 19 all reset to `0x0` (Table 208,
  PDF p186), so every configurable fault escalates to HardFault. The
  `DefaultHandler` slots 4 through 7 of chapter 05 §5.6's vector table are dead
  code that can never be entered.
- **`MSPLIM_S`.** The Secure main stack limit register (PDF p135) is never
  written, and its value at bootrom exit is not documented in Table 450. Writing
  `0` costs three instructions; writing `__ebss` buys you a real stack-overflow
  fault instead of silent corruption of `.bss`.
- **`cpsid i` at the top of `OnReset`.** Harmless on a cold boot, since nothing
  can fire (§6.3.2). Cheap hardening for a warm reboot, where those reset values
  are no longer what you are looking at.
- **Swapping `enable_fpu` and `reset_vtor`.** Until VTOR is written, any fault
  vectors through whatever the bootrom left. `enable_fpu` runs first today, so
  that window is a handful of instructions wide; swapping the two calls shrinks
  it to zero at no cost.

Next: chapter 07 §7.1 takes the `read_volatile` / `write_volatile` /
read-modify-write vocabulary this chapter used on CPACR and VTOR and makes it
the subject, before chapter 08 applies it to GPIO.
