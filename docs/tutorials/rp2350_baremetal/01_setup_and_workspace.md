---
document_type: "Tutorial Chapter — Toolchain and Workspace"
program: rustos (Raspberry Pi Pico 2 / RP2350)
chapter: 1 of 9
revision: C
effective_date: 2026-08-29
parent_index: docs/tutorials/rp2350_baremetal/index.md
prerequisites: none — this is the first chapter
sources: RP2350 datasheet §3.6 (PDF p101), §3.6.4 (PDF p124), §3.7.3.1
  (PDF p130), PDF p36; Pico 2 datasheet p4, p5. Everything else here is
  measured, not cited — reproduced on 2026-08-29 with rustc 1.98.0
  (88d9e12ae 2026-08-18), cargo 1.98.0, LLD 22.1.8, picotool v2.3.0,
  host aarch64-apple-darwin.
creates: Cargo.toml, .cargo/config.toml, api/Cargo.toml, demo/Cargo.toml,
  firmware/pico2/Cargo.toml, firmware/pico2/build.rs; the api crate's four
  source files (copied from the tree, §1.3.2); the first lines of
  firmware/pico2/src/lib.rs and demo/src/main.rs (§1.9)
---

# Chapter 01 — Toolchain and Workspace

You end this chapter with a three-crate workspace that cross-compiles to
`thumbv8m.main-none-eabihf`, a build-and-inspect loop you use in every later
chapter, and one deliberate failure that chapter 02 exists to fix. No RP2350
register appears here.

## 1.1 What you need installed

```
rustup target add thumbv8m.main-none-eabihf
rustup component add llvm-tools
brew install picotool          # macOS; elsewhere build from raspberrypi/picotool
```

`rustup` and a stable toolchain are assumed. Verify:

```
$ rustc --version
rustc 1.98.0 (88d9e12ae 2026-08-18)
$ rustup target list --installed
aarch64-apple-darwin
thumbv8m.main-none-eabihf
$ picotool version
picotool v2.3.0 (Darwin, AppleClang-21.0.0.21000099, Release)
```

Checking `llvm-tools` is awkward: the binaries are present in the sysroot on
this machine and `rustup component list --installed` still does not name them.
`llvm-tools-aarch64-apple-darwin` appears in the *full* `rustup component list`
but not in the installed one, and this tutorial does not know why. Treat the
installed list as unreliable for this component and list the directory instead —
§1.7 puts it on `PATH`. Eighteen entries land there; this tutorial uses
`llvm-objdump`, `llvm-nm`, `llvm-size`, `llvm-readobj` and `rust-lld`.

`picotool` inspects an image file (chapter 05) and flashes a board (chapter 08).

### 1.1.1 There is no `rust-toolchain.toml`

Verified: none exists anywhere in this repository, so the toolchain is whatever
`stable` resolves to on your machine. That is a **known gap, not a design
choice**: every measured number in this tutorial came from rustc 1.98.0 with
LLD 22.1.8, and another stable release will move some of them. Pinning is one
file, which would also make both `rustup` commands above unnecessary:

```toml
# PROPOSED — not in the tree today: rust-toolchain.toml
[toolchain]
channel = "1.98.0"
targets = ["thumbv8m.main-none-eabihf"]
components = ["llvm-tools"]
```

## 1.2 Why this target triple

`thumbv8m.main-none-eabihf` is five decisions in one string.

| Piece | Means |
|---|---|
| `thumb` | the Thumb instruction set — the only one a Cortex-M executes |
| `v8m.main` | ARMv8-M **Mainline**, as opposed to `v8m.base`, the Baseline profile |
| `none` | no operating system; no libc, no loader, no process |
| `eabi` | the ARM Embedded ABI — calling convention, struct layout, symbol naming |
| `hf` | **hard float**: floating-point arguments travel in FPU registers |

Each piece matches the chip. RP2350 has two core sockets, and "the processor
plugged into each socket is selectable at boot time: A Cortex-M33 processor,
implementing the Armv8-M Main instruction set, plus extensions" — Cortex-M33
being the default (PDF p36). Mainline, therefore `v8m.main`. Same page: "They
are configured with the Security, DSP and FPU extensions"; §3.7.3.1 (PDF p130)
lists compliance with the Armv8-M Main and Floating-point Extensions.

The triple is readable back out of a finished ELF: `llvm-readobj
--arch-specific` on the release build of §1.7 reports `CPU_arch: ARM v8-M
Mainline`, `THUMB_ISA_use: Permitted` beside `ARM_ISA_use: Not Permitted`
(`thumb` as a hard constraint — A32 is not available at all), and
`ABI_VFP_args: AAPCS VFP`, which is the `hf`. (AAPCS is the Arm Architecture
Procedure Call Standard — the document that defines the calling convention the
`eabi` piece names, including how floating-point arguments are passed and how
the stack must be aligned. Chapter 04 meets its stack-alignment rule.)

### 1.2.1 The `hf` is why chapter 06 enables the FPU

`hf` means the compiler may emit FPU instructions and pass floats in FPU
registers, on any call, without asking. The FPU on RP2350 is a coprocessor:
"The Cortex-M33 cores on RP2350 are configured with the standard Arm
single-precision floating point unit (FPU). Coprocessor ports 10 and 11 access
the FPU" (§3.6.4, PDF p124). Coprocessors start off: "Before accessing a
coprocessor from Secure code, that coprocessor must first be enabled by setting
the corresponding bit in the CPACR" (§3.6, PDF p101).

Choosing `hf` therefore obliges you to enable coprocessors 10 and 11 in `CPACR`
before any code that might touch the FPU runs. The firmware does that as the
first statement of its reset handler, in chapter 06 — an obligation you take on
here, in the target triple, long before you pay it.

## 1.3 The workspace

Four manifests, eleven Rust source files, a build script, a linker script and
a config file:

```text
Cargo.toml, .cargo/config.toml (§1.5)
api/             Cargo.toml, src/{lib.rs, common/{mod.rs, board.rs}, gpio/mod.rs}
demo/            Cargo.toml, src/main.rs
firmware/pico2/  Cargo.toml, build.rs (§1.6), link.ld (chapters 02, 04),
                 src/{lib.rs, common/{mod.rs, reg.rs, reset.rs}, gpio/{mod.rs, gpio.rs}}
```

The root `Cargo.toml`, verbatim:

```toml
[workspace]
resolver = "3"
members = ["api", "demo", "firmware/pico2"]

[profile.dev]
panic = "abort"
[profile.release]
panic = "abort"
```

`firmware/pico2/Cargo.toml`, `demo/Cargo.toml` and `api/Cargo.toml`, verbatim,
in that order:

```toml
[package]
name = "pico2"
version = "0.1.0"
edition = "2024"

[dependencies]
api = { path = "../../api" }
```

```toml
[package]
name = "demo"
version = "0.1.0"
edition = "2024"

[dependencies]
api = { path = "../api" }
pico2 = { path = "../firmware/pico2" }
```

```toml
[package]
name = "api"
version = "0.1.0"
edition = "2024"



[dependencies]
```

All three crates are edition 2024, and `api` has no dependencies: nothing
outside this repository is compiled into the image.

### 1.3.1 Three crates, one binary

Only one of the three crates is a program. `demo/src/main.rs` is the single
binary in the workspace, so the ELF that every later chapter inspects and
flashes is

```text
target/thumbv8m.main-none-eabihf/release/demo
```

The other two are libraries, and the split is a dependency arrow that only
points one way:

- **`api`** defines traits — named method sets like "a thing a `bool` can be
  written to" — plus one concrete type (`Board`). It contains no register
  addresses and no RP2350 knowledge, which is why it also compiles for your
  laptop; §1.5.1's `xtest` alias does exactly that.
- **`pico2`** is the runtime and the chip support: the boot metadata block,
  the vector table, the reset handler, the `entry!` macro that names the
  application's entry point (chapter 06), and a GPIO driver that implements
  `api`'s traits (chapter 08). It is a **library** — building it alone
  produces `libpico2.rlib`, not a flashable image.
- **`demo`** is the application: it declares its `main` to `pico2` and blinks
  the LED through `api`'s traits. Chapter 08 finishes it.

`api` never depends on `pico2`, so a driver for a different chip could
implement the same traits and `demo`'s logic would move across unchanged. That
is the entire reason the layer exists, and chapters 07 §7.7 and 08 §8.11 pick
it back up. Nothing about *booting* the chip involves `api`; chapters 02
through 06 live almost entirely in `firmware/pico2`.

### 1.3.2 The `api` crate is taken as given

This tutorial's subject is the bare-metal material — the linker script, the
boot block, the registers. The `api` crate contains none of that: it is four
files of ordinary portable Rust (`lib.rs`, `common/mod.rs`, `common/board.rs`,
`gpio/mod.rs`) that would compile in any project. Copy them from the tree now,
so the workspace resolves, and read them when chapter 08 starts implementing
them; chapter 07 §7.7 describes what each one declares.

## 1.4 Why `panic = "abort"`

Rust's default panic strategy unwinds: it walks back up the stack running
destructors. Unwinding is a runtime, driven on ARM by `.ARM.exidx`, a section
of per-function unwind tables the compiler emits alongside your code — and
nothing here implements it: no `libunwind`, no personality routine.

`panic = "abort"` removes the requirement: panics reach the `#[panic_handler]`
in `firmware/pico2/src/lib.rs` and never return, so no unwind tables are
needed. It is set in **both** profiles because you will build both.

This is also what makes the `/DISCARD/` rule for `.ARM.exidx` in the linker
script meaningful rather than superstitious — chapter 04 reads that line.
Confirmed on the current release ELF: `llvm-readobj --sections` piped through
`grep -i exidx` prints nothing, so no such section survives.

## 1.5 `.cargo/config.toml`

Three blocks, verbatim:

```toml
[build]
target = "thumbv8m.main-none-eabihf"

[target.thumbv8m.main-none-eabihf]
rustflags = [
    "-C", "link-arg=-Tlink.ld",
    "-C", "link-arg=--print-memory-usage",
    "-C", "link-arg=-Map=firmware.map",
]

[alias]
xtest = "test -p api --target aarch64-apple-darwin"
```

- **`[build] target`** — a bare `cargo build` cross-compiles; you never type
  `--target`. This is why output lands in `target/thumbv8m.main-none-eabihf/`.
- **`-Tlink.ld`** — hand the linker your script instead of its built-in
  default. Chapters 02 and 04 rest on this flag; §1.6 is how it is found.
- **`--print-memory-usage`** — a region utilisation table on every build (§1.8).
- **`-Map=firmware.map`** — which input object contributed each byte, at what
  address. Written to the **workspace root**, not `firmware/pico2/`.
- **`xtest`** — the escape hatch from `[build] target`, below.

### 1.5.1 The `xtest` alias

`[build] target` is global, so `cargo test` cross-compiles too — and there is
no test harness for a bare-metal target:

```
$ cargo test -p api
error[E0463]: can't find crate for `test`

error: `#[panic_handler]` function required, but not found

For more information about this error, try `rustc --explain E0463`.
error: could not compile `api` (lib test) due to 2 previous errors
```

The alias overrides both the package and the target:

```
$ cargo xtest
     Running unittests src/lib.rs (target/aarch64-apple-darwin/debug/deps/api-...)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests api

running 1 test
test api/src/common/mod.rs - common::ErrorType (line 61) ... ignored

test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

It compiles and runs — for the host — which is the point: the mechanism for
testing `api` off-board exists and works. What it has to run is currently
thin: `api` contains **no unit tests** (verified — no `#[test]` anywhere in
`api/src/`), and its one doctest is an `ignore`d illustration. Chapter 07
§7.7 shows the kind of test this slot is for.

The `aarch64-apple-darwin` is hardcoded, so the alias works only on an Apple
Silicon host. Elsewhere, substitute your own `rustc -vV | grep host`.

## 1.6 `build.rs`

The code of `firmware/pico2/build.rs`, four lines (the tree carries an
explanatory comment block above them):

```rust
fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search={dir}");
    println!("cargo:rerun-if-changed=link.ld");
}
```

Two `println!`s, two distinct reasons. Both are demonstrated below by breaking
them in a scratch copy of this workspace; the tree's `build.rs` and `link.ld`
are untouched.

### 1.6.1 `rustc-link-search` — so `-Tlink.ld` resolves

Cargo runs `rustc` with the working directory set to the **workspace root**,
not the crate directory, so a bare `-Tlink.ld` resolves there — where the
script is not. Delete `build.rs` and the link fails:

```
$ cargo build --release
error: linking with `rust-lld` failed: exit status: 1
  = note: rust-lld: error: cannot find linker script link.ld
error: could not compile `demo` (bin "demo") due to 1 previous error
```

Note who the error is charged to: `demo`. Only binaries get linked — `pico2`
and `api` compile to `.rlib` archives and never meet the linker on their own —
so every linker-side failure in this workspace is reported against `demo`,
whichever crate actually caused it.

Confirm the cwd claim by leaving `build.rs` deleted and putting a copy of
`link.ld` at the workspace root: the link then succeeds. The script is found —
in the wrong place, which is the point.

`cargo:rustc-link-search={dir}` adds `CARGO_MANIFEST_DIR`, here
`firmware/pico2/`, to the linker's search path, so `-Tlink.ld` finds the script
beside the crate that owns it. It arrives as a `-L` flag — the unabridged
`= note:` line above shows `"-L" ".../firmware/pico2"` among the linker
arguments, with `"-Tlink.ld" "--print-memory-usage" "-Map=firmware.map"` at
the end of them.

### 1.6.2 `rerun-if-changed=link.ld` — so edits are not ignored

Cargo does not know a linker script is an input unless told, and the rule is
subtle enough to state exactly. A build script printing **no**
`rerun-if-changed` directive gets Cargo's default: re-run when anything in the
package directory changes, `link.ld` included. A build script printing **any**
such directive gets only what it names.

This line therefore does not add tracking on top of a default — it preserves
tracking that any other `rerun-if-changed` would take away. Measured, with
`rerun-if-changed=build.rs` substituted and `link.ld` edited to move `ORIGIN`:

```
$ cargo build --release
    Finished `release` profile [optimized] target(s) in 0.00s
$ llvm-objdump --section-headers target/thumbv8m.main-none-eabihf/release/demo
  1 .vector_table   00000000 10000000 DATA   <-- old address; script says 0x10001000
```

The section is empty because the scratch crate has no vector table.

> **Silent-failure trap.** A build script naming any file other than `link.ld`
> in `rerun-if-changed` makes linker-script edits invisible to Cargo. You get
> `Finished` in 0.00s and the previous binary, and every symptom you then chase
> belongs to the image you already had. If a linker-script change appears to do
> nothing, `touch firmware/pico2/src/lib.rs` and build again first.

## 1.7 The build-and-inspect loop

```
cargo build --release
```

That is the loop. Everything after it is inspection, and inspection is most of
the work: on a board whose only output is one LED, the ELF is the only place
the build's correctness can be examined before flashing. Put `llvm-tools` on
`PATH` first; this line derives sysroot and host triple:

```
export PATH="$PATH:$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin"
```

| Command | Answers |
|---|---|
| `llvm-objdump --section-headers <elf>` | did my sections land where the script said? (name, size, VMA, type) |
| `llvm-objdump -s -j .boot_info <elf>` | what bytes are actually in this section? |
| `llvm-objdump -d -C <elf>` | disassembly of the whole image |
| `llvm-objdump -d -C --disassemble-symbols=OnReset <elf>` | disassembly of one function |
| `llvm-nm -n <elf>` | every symbol by address (`T`=.text `R`=.rodata `D`=.data `B`=.bss `A`=absolute) |
| `llvm-size <elf>` | text / data / bss totals |
| `llvm-readobj --file-headers <elf>` | the ELF entry point |
| `llvm-readobj --arch-specific <elf>` | the ARM build attributes of §1.2 |

There is no `llvm-readelf`: where another tutorial says `readelf -S`, use
`llvm-objdump --section-headers` or `llvm-readobj --sections`. One of them on
the firmware, so you know what a healthy result looks like:

```
$ llvm-readobj --file-headers target/thumbv8m.main-none-eabihf/release/demo | grep Entry
  Entry: 0x10000307
```

An odd address. That is not a mistake, and chapter 05 says why.

The build is not silent, and the noise is expected. A clean release build of
the finished tree emits exactly **one** warning, a `linker_messages` warning
that is not a warning at all but the memory report of §1.8 — rustc routes
linker stdout through the warning channel, and it lands on `demo` because
`demo` is the crate that gets linked (§1.6.1).

### 1.7.1 `-C`, and the two kinds of function name in this image

`-C` is the short form of `--demangle`, and every disassembly listing in this
tutorial was taken with it. Without it, a Rust function prints under its mangled
symbol. The instruction below is the runtime handing control to the
application — chapter 06 explains the `__rustos_main` symbol it lives in:

```
$ llvm-objdump -d --no-show-raw-insn --disassemble-symbols=__rustos_main <elf> | tail -1
100002fc:      	bl	0x10000124 <_RNvCsgEed0wyhMeN_4demo4main>
$ llvm-objdump -d -C --no-show-raw-insn --disassemble-symbols=__rustos_main <elf> | tail -1
100002fc:      	bl	0x10000124 <demo::main>
```

The hash in the middle (`CsgEed…`) is the crate disambiguator; it is not stable
across toolchains or across a `Cargo.toml` edit, so do not match on it.

The image ends up holding both kinds of name, and the difference is `no_mangle`.
`OnReset`, `DefaultHandler`, `OnHardFault` (chapter 05) and `__rustos_main`
(chapter 06) carry `#[unsafe(no_mangle)]` because the linker script, the vector
table, or another crate has to name them from outside normal Rust name
resolution, so their symbols *are* their source names. `demo::main` and
everything in `gpio.rs` do not, so their symbols are mangled.

> **Not-a-bug trap.** `--disassemble-symbols` matches the name as printed, which
> means `-C` changes which spelling it accepts. Without `-C` it wants the mangled
> symbol; with `-C` it wants the demangled path. So `--disassemble-symbols=main`
> fails either way, with
> `llvm-objdump: warning: … failed to disassemble missing symbol main`, and that
> message reads exactly like the function was optimised away. It was not:
> `-C --disassemble-symbols=demo::main` prints it. `OnReset` works bare because
> it is `no_mangle`, which is why the asymmetry is easy to trip over.

## 1.8 Reading `--print-memory-usage`

Every build prints a table of this shape. The numbers in it, and in the section
listing below, are the **finished** firmware's — the end of chapter 08. Your own
build gets no further than §1.9 today, and chapter 04 §4.11.1 is the first one
that prints a table of its own:

```
Memory region         Used Size  Region Size  %age Used
           FLASH:        7044 B         4 MB      0.17%
             RAM:        8200 B       520 KB      1.54%
```

The region names and sizes come from the `MEMORY` block of `link.ld`, not from
the chip — the linker has no idea what an RP2350 is. They do match the board:
"RP2350A microcontroller with 4 MB flash", "520 kB multi-bank high performance
SRAM" (Pico 2 datasheet p4, p5).

Both numbers come straight off the section table:

```
$ llvm-objdump --section-headers target/thumbv8m.main-none-eabihf/release/demo
Idx Name            Size     VMA      Type
  1 .vector_table   00000110 10000000 DATA
  2 .boot_info      00000014 10000110 DATA
  3 .text           00001888 10000124 TEXT
  4 .rodata         000001d8 100019ac DATA
  5 .data           00000000 20000000 DATA
  6 .bss            00000004 20000000 BSS
  7 .stack          00002000 20000008 BSS
```

`7044 B` of flash is the four non-empty flash sections added up,
`0x110 + 0x14 + 0x1888 + 0x1d8 = 7044`. The RAM figure needs one more step:
`.data` is empty, `.bss` is exactly **4 bytes** — one zero-initialised static,
a flag inside the `api` crate that chapter 08 explains — and `.stack` is the
8 kB reservation, pushed up to `0x20000008` so it starts 8-byte aligned. RAM
used is `0x20002008 - 0x20000000` = 8200 bytes: four of data, four of
alignment padding, 8192 of stack. Chapters 03 to 08 explain every other
number there.

That is the entire reason `.stack` exists. Stack usage is invisible to a
linker — the stack is a pointer moving down through RAM at runtime, and nothing
in the ELF records it. Reserving a named, empty 8 kB section makes the stack
something the linker can count and, more usefully, assert about. Chapter 04
reads that section and the `ASSERT` guarding the gap between it and `.bss`.

**Inferred:** the table is therefore a budget check, not a correctness check.
It tells you 8 KB was reserved; it cannot tell you whether 8 KB is enough,
because nothing here measured a call depth. Reading `1.54%` as reassurance
about stack safety is the mistake this paragraph exists to prevent.

## 1.9 What does not work yet

Copy the build files into an empty workspace — the four manifests, the config
file, `build.rs`, and the `api` crate's sources (§1.3.2) — then give each of
the two remaining crates the smallest file that compiles.

`firmware/pico2/src/lib.rs`, eight lines:

```rust
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

The panic handler is **real** — it is verbatim the one in the tree's
`firmware/pico2/src/lib.rs`, not scaffolding you throw away. The `use` line
starts narrower than the tree's (chapter 06 §6.4.3 widens it to add a second
import at the point the code needs it) and the tree's two `pub mod`
declarations arrive in chapter 08, when the files they name exist. The
attributes earn their place now:

- `#![no_std]` — no `std`, because `std` wants an OS underneath it.
- `use core::panic::PanicInfo;` — `core` is what `no_std` leaves you.
- `#[panic_handler]` — `std` normally provides this. Without an OS it is yours
  to write, and it must diverge. `loop {}` is what the firmware ships. It lives
  in the **library**, so every binary built on `pico2` inherits it — and,
  since a program may have exactly one, no such binary may define its own.

`demo/src/main.rs`, two lines first, on purpose:

```rust
#![no_std]
#![no_main]
```

- `#![no_main]` — no Rust `main` entry point, because the hardware enters
  through the vector table instead (chapter 05 §5.6). Chapter 06 shows how a
  function named `main` comes back in through the `entry!` macro.

Build that, and it fails before any linker is involved:

```
$ cargo build --release
error: `#[panic_handler]` function required, but not found

error: could not compile `demo` (bin "demo") due to 1 previous error
```

The handler exists — you just wrote it — but it is in a crate `demo` never
mentions, and rustc only loads a dependency the source actually names.
Declaring `pico2` in `Cargo.toml` makes it *available*, not *present*. The fix
is one line that imports the crate for its side effects — its panic handler —
and binds it to no name:

```rust
#![no_std]
#![no_main]

use pico2 as _;
```

This is the one line in the tutorial that later stops being true to the tree:
chapter 06 replaces it with `pico2::entry!(main);`, which names the crate and
makes the placeholder import redundant. Everything else you type stays.

Build again, with the `.cargo/config.toml` of §1.5 exactly as written and no
`link.ld` yet:

```
$ cargo build --release
error: linking with `rust-lld` failed: exit status: 1
  = note: rust-lld: error: cannot find linker script link.ld
error: could not compile `demo` (bin "demo") due to 1 previous error
```

Loud, specific, correct — the good failure. Now the interesting one: drop the
`-Tlink.ld` flag, the reasonable-looking move when the error names a file you
have not written yet, and build again.

```
$ cargo build --release
warning: linker stderr: rust-lld: cannot find entry symbol _start; not setting start address

warning: linker stdout: Memory region         Used Size  Region Size  %age Used

warning: `demo` (bin "demo") generated 2 warnings
    Finished `release` profile [optimized] target(s) in 0.34s
$ llvm-objdump --section-headers target/thumbv8m.main-none-eabihf/release/demo
Idx Name            Size     VMA      Type
  1 .comment        00000099 00000000
  2 .ARM.attributes 00000038 00000000
  3 .symtab         00000030 00000000
  4 .shstrtab       00000034 00000000
  5 .strtab         0000003a 00000000
$ llvm-readobj --file-headers target/... | grep Entry
  Entry: 0x0
```

It succeeded, and produced an ELF with no `.text` — no code at all, only ELF
bookkeeping — and an entry point of zero. (The second warning is
`--print-memory-usage` printing a table with no rows: with no script there is
no `MEMORY` block to report on.) Without a script the linker fell back to its
hosted-ELF default: it looked for `_start`, did not find it, kept nothing, and
called that a warning.

> **Silent-failure trap.** Losing the linker script downgrades a hard error to
> a warning and still produces an ELF. `cargo build` says `Finished`, a file
> appears where you expect one, and the chip then sits dead with no diagnostic.
> Whenever a build "succeeds" but nothing happens on the board, check
> `--section-headers` for a `.text` and `--file-headers` for a non-zero entry.

None of that is an RP2350 problem. It is the linker asking a question nobody
has answered yet: **what address does each byte get?** On a hosted OS a loader
you never see answers it. Here there is no loader — chapter 02 answers it.
