# RustOS

RustOS is an operating system designed from the ground up in Rust, living
entirely in the Rust ecosystem. The premise: mainstream operating systems
carry decades of accumulated design constraints — C ABIs, unsafe-by-default
interfaces, invariants enforced by convention and runtime checks because
their languages could not express them. Rust was created on the principle of
keeping the lessons of previous language generations while discarding their
accumulated compromises; RustOS applies that same principle to the operating
system. Concretely: ownership and the type system move OS invariants
to compile time wherever the language can express them — forging a pin
handle, configuring the same pin twice, and writing an error branch for an
operation that cannot fail are all build errors. The invariants that are
inherently per-boot facts — has the board been claimed yet, has this driver
been brought up yet — cannot be compile-time, and are instead enforced by
runtime constructors that succeed at most once. The
[API section](#the-api-invariants-at-compile-time) below shows exactly which
is which, in the code that exists today.

It is also built for the joy of building it: the author is a professional
engineer doing this because writing code is fun.

## The first target: Raspberry Pi Pico 2 (RP2350)

The first hardware target is the Raspberry Pi Pico 2 board and its RP2350
microcontroller, brought up fully bare metal: no operating system and no
vendor library underneath the code. This workspace contains every piece the
chip needs to boot and blink, all written by hand and traced to datasheet
citations:

- the boot metadata the chip's **bootrom** looks for (the bootrom is the
  fixed program in the chip's ROM that runs first after power-on, finds a
  valid image in flash, and starts it);
- the **vector table**, the array at a fixed address whose entries the CPU
  reads directly: the initial stack pointer, then the addresses of the
  functions to run on reset and on each interrupt;
- the **reset handler**, the function named in that table that runs before
  anything else of ours;
- the **linker script**, the file that tells the linker at which address in
  the chip's memory map to place each part of the program;
- and a **GPIO** driver (general-purpose input/output — a chip pin the
  program can drive high or low, or read the level of).

Each of those pieces is taught from scratch in this repository's
[bare-metal tutorial](docs/tutorials/rp2350_baremetal/index.md). No external
embedded crates (`cortex-m`, `cortex-m-rt`, `rp-hal`, `embassy`, …) are
used; nothing outside this repository is compiled into the firmware image.

The repository also carries a large set of **design documents for a planned
flight software system named "Juno"** (for a rocket flight test called FT1).
Those documents describe modules — GPS, IMU, barometer, LoRa radio, SD
logging, a navigation filter, a scheduler, and more — that are **not yet
implemented in code**. See [What is implemented and what is
planned](#what-is-implemented-and-what-is-planned) below so the two are never
confused.

## Repository layout

The code is a Cargo **workspace** (one repository containing several Rust
packages, called *crates*, built together) with three crates:

```
rustos/
├── api/                portable hardware-interface definitions (library)
├── firmware/pico2/     RP2350 runtime + drivers (library)
├── demo/               the blinky application (binary — this is what you flash)
├── docs/               datasheets, tutorials, and the Juno planning baseline
├── blinky.elf          prebuilt demo binary (ELF format)
└── blinky.uf2          prebuilt demo binary (UF2 format, ready to drag onto the board)
```

**`api/`** is the portability boundary. It defines the traits (Rust
interfaces) that application code programs against — `ErrorType`, `Read`,
`Write`, `Gpio`, `GpioPinIn`, `GpioPinOut`, and the `Pull` enum — plus the
`PinHandle` ownership type and the `define_board!` macro described in the
next section. The crate contains no register addresses and no dependency on
any other crate in the workspace, so it compiles for your host machine as
well as for the microcontroller, which is what makes logic written against
it testable without hardware.

**`firmware/pico2/`** is a **library** crate (it has `src/lib.rs` and no
`main.rs`) holding everything that must exist before and around user code:

- `BOOT_INFO`, the RP2350 `IMAGE_DEF` metadata block the bootrom requires;
- the Cortex-M vector table and the `OnReset` reset handler, which does the
  work C startup code normally hides: enable the FPU (the floating-point
  unit, powered off at reset), set `VTOR` (the CPU register that holds the
  vector table's address), copy `.data` (the linker section holding the
  initial values of initialized global variables) from flash to RAM, and
  zero `.bss` (the section for zero-initialized globals);
- the `entry!` macro, which an application uses to declare its `main` — the
  macro emits the `__rustos_main` symbol the reset handler calls, and
  type-checks `main` against `fn() -> !` at compile time (`!` is the type of
  a function that never returns; on bare metal there is nothing to return
  to, so a `main` that could fall off the end is a type error);
- `common/board.rs`, which instantiates `api`'s `define_board!` macro as the
  `Rp2350` board type with the Pico 2's pin map;
- a GPIO driver (`Rp2350Gpio`, producing `Rp2350GpioIn` / `Rp2350GpioOut`
  pins) implementing the `api` traits;
- `link.ld`, the linker script that places every section in the RP2350's
  memory map, and `build.rs`, which tells Cargo where to find it.

**`demo/`** is the only **binary** crate — a blinky that toggles the Pico 2's
onboard LED. It takes the `Rp2350` board singleton (a value the program can
obtain exactly once — the next section shows the mechanism), constructs the
`Rp2350Gpio` driver, passes the board's `led` pin handle to that driver to
receive a configured output pin, and drives it through the portable
`Write<bool>` trait. Building the
workspace produces the flashable image at
`target/thumbv8m.main-none-eabihf/release/demo`.

## The API: invariants at compile time

This is the part of the mission that already exists in code, so it is worth
stating precisely. Three invariants that embedded C enforces by convention
are enforced here by the type system:

**A pin you hold exists, and nobody else holds it.**
`api::device::PinHandle<const N: usize>` is a zero-sized type — it has no
fields and occupies no memory in the compiled program. Its parameter `N` is
a *const generic*: Rust generics can be parameterized by constant values as
well as by types, and here the physical pin number is part of the type
itself, so `PinHandle<25>` and `PinHandle<24>` are two distinct types and
passing one where the other is required is a type error. The handle's only
constructor, `PinHandle::new`, is declared `unsafe`, which means safe code
cannot call it at all; the only place it is invoked is inside the code
generated by the `api::define_board!` macro, which a firmware crate invokes
once with the pin map of a real board — so a handle can only come from a
board's `take()`. The macro generates a `Pins` struct containing exactly one
`PinHandle` field per pin that physically exists on that board, wrapped in a
board struct whose `take()` hands the whole struct out at most once per
boot. `take()` returns `Option<Self>` — `Option` is Rust's built-in type for
a value that may be absent, with two variants: `Some(value)` and `None`.
The once-only behavior rests on an `AtomicBool`, a boolean flag whose
operations each execute as one indivisible step even with multiple CPU cores
running; `take()` performs a *compare-exchange* on it, which checks that the
flag is `false` and sets it `true` in that single step. A plain
`if !taken { taken = true; ... }` is two separate steps, and two cores could
both pass the check before either sets the flag; with compare-exchange,
even simultaneous calls yield exactly one `Some` — every other and every
later caller gets `None`.

Holding a `PinHandle<25>`, then, proves two things by two different
mechanisms. That pin 25 exists on this board is checked at compile time: the
generated `Pins` struct has a field only for each pin that exists, so there
is no way to obtain a `PinHandle<40>`. That no other code path holds one is
a runtime-plus-contract property: `take()` succeeds once, and because
`PinHandle::new` is `unsafe`, safe code cannot construct a second handle —
an `unsafe` block could, and not doing so is the stated contract on every
`unsafe` block in this tree. `firmware/pico2/src/common/board.rs`
instantiates the macro as `Rp2350`/`Rp2350Pins`: `gpio0`–`gpio22` for the
free header pins, then `smps_ps` (23), `vbus_sense` (24), and `led` (25) —
named for the on-board circuits the Pico 2 commits them to; see the
[Pico 2 datasheet](docs/pico-2-datasheet.pdf) for the power-supply (SMPS),
USB-voltage (VBUS), and system-voltage (VSYS) circuits behind those names —
then `gpio26`–`gpio28`, which can also feed the chip's ADC
(analog-to-digital converter: it measures the voltage on a pin and returns
it as a number), and `vsys_adc` (29).

**Configuration consumes the proof.**
`api::gpio::Gpio` is the factory trait a GPIO driver implements. One Rust
rule makes this invariant work, and it is where Rust departs from C: passing
a value of a non-copyable type to a function does not copy it, it *moves*
it — ownership transfers into the function, the caller's variable is
invalidated, and any later use of that variable is a compile error.
`PinHandle` is non-copyable, and the factory's two methods,
`input_from_handle(handle, pull)` and `output_from_handle(handle)`, take it
**by value**: configuring a pin consumes the handle, so using the same
handle to configure the pin twice is a compile error, not a runtime check.
What you get back is named by the trait's *associated types* — placeholder
types declared inside a trait that each implementing type must fill in with
concrete types of its own. Here the placeholders themselves take the
pin-number parameter (*generic* associated types):
`type Input<const N: usize>: GpioPinIn<N>` and
`type Output<const N: usize>: GpioPinOut<N>`, so an implementation supplies
one concrete pin type per pin number. `GpioPinIn`
and `GpioPinOut` are marker traits — traits with no methods of their own,
existing to name a capability — over `api::common::Read<bool>` and
`api::common::Write<bool>`, so a configured pin is used through the same
portable `read`/`write` traits any other peripheral would use.
`api::common::ErrorType` gives each peripheral a single associated error
type that all of its operations agree on.

**An operation that cannot fail returns an error type with no values.**
The RP2350 implementation, `pico2::gpio::gpio::Rp2350Gpio`, is zero-sized;
its `new()` performs the reset bring-up of the `IO_BANK0` and `PADS_BANK0`
blocks and is a once-per-boot singleton by the same `AtomicBool`
compare-exchange as `take()`, returning `Option<Self>`. The pin types it
hands out, `Rp2350GpioIn<N>` and `Rp2350GpioOut<N>`, declare
`Error = core::convert::Infallible` — an enum with no variants. Unlike a C
enum, whose values are just integers whether or not any enumerator names
them, a Rust enum value must be one of the enum's declared variants, so a
zero-variant enum has no values at all: the `Err` case of a
`Result<T, Infallible>` can never be constructed, and the compiler removes
that branch. Every failure mode was already handled before the pin existed;
the type signature says so.

The blinky in `demo/src/main.rs` is the whole pattern. Its `main`, minus the
busy-wait `delay()`, is:

```rust
fn main() -> ! {
    let board = Rp2350::take().unwrap();
    let mut gpio = Rp2350Gpio::new().unwrap();
    let mut led = gpio.output_from_handle(board.pins.led).unwrap();
    loop {
        led.write(true);
        delay();
        led.write(false);
        delay();
    }
}
```

All three setup calls can report failure — the two `Option`s described
above, and `output_from_handle` returns a `Result` — and the demo
`.unwrap()`s each one, meaning: halt if the call failed. That is the right
choice here: the `Option`s are `None` only on a second call to a
once-per-boot constructor, which is a program bug, not a condition to
recover from. The same
listing, annotated line by line, is the crate-level doc of
`firmware/pico2/src/lib.rs` ("Writing an application"). Run
`cargo doc --open` to browse the API documentation; when wiring your own
*input* pin, read `api::gpio::Pull` there (source: `api/src/gpio/mod.rs`),
which explains how to choose the pull configuration for a button or other
input circuit.

## Prerequisites

1. **Rust**, installed via [rustup](https://rustup.rs). Verified with
   `rustc 1.98.0`.
2. **The cross-compilation target.** The RP2350's Arm cores are Cortex-M33
   processors; Rust names that platform with the *target triple*
   `thumbv8m.main-none-eabihf`. Install the precompiled core library for it:

   ```sh
   rustup target add thumbv8m.main-none-eabihf
   ```

3. **picotool** (Raspberry Pi's tool for inspecting and converting RP2350
   images), only needed for flashing. On macOS: `brew install picotool`.
   Verified with picotool v2.3.0.

## Building

```sh
cargo build --release
```

That is the whole command: `.cargo/config.toml` in the repository root sets
the target triple and the linker-script arguments, so no extra flags are
needed. The linker prints a memory report as part of the build. As of this
commit it reads:

```
Memory region         Used Size  Region Size  %age Used
            FLASH:        2096 B         4 MB      0.05%
              RAM:        8200 B       520 KB      1.54%
```

(These numbers change whenever the code does; the report printed by your own
build is the authoritative one.) The build also emits one `linker_messages`
warning attributed to `demo` — that is expected; it is how Cargo surfaces the
memory report above.

## Flashing

The build output is an **ELF** file (the standard executable format the
linker produces). The Pico 2's bootrom accepts drag-and-drop flashing in the
**UF2** format, so convert first. picotool identifies file types by
extension, so give the ELF a `.elf` name before converting:

```sh
cp target/thumbv8m.main-none-eabihf/release/demo blinky.elf
picotool uf2 convert blinky.elf blinky.uf2 --family rp2350-arm-s
```

Then hold the **BOOTSEL** button on the Pico 2 while plugging in its USB
cable. The board enumerates as a USB mass-storage drive; copy `blinky.uf2`
onto that drive. The board reboots itself and the onboard LED blinks.
Alternatively, with the board in BOOTSEL mode: `picotool load blinky.uf2`
followed by `picotool reboot`.

The `blinky.elf` and `blinky.uf2` files committed at the repository root were
verified (2026-08-30) to be byte-identical to a fresh `cargo build --release`
and conversion, so you can flash `blinky.uf2` directly without building.

## Host-side tests

The `api` crate is host-compilable, and the alias `cargo xtest` (defined in
`.cargo/config.toml`) runs its test suite on the host instead of the embedded
target. The alias hard-codes the `aarch64-apple-darwin` host triple, so as
written it only runs on Apple-silicon macOS; on another host, substitute your
own triple. The suite currently contains no unit tests; the alias is kept so
that when tests are added to `api/`, `cargo xtest` will already compile and
run them on the host with no further setup.

## Learning path

The main learning path is the bare-metal tutorial:

- **[Bare-Metal Rust on the Raspberry Pi Pico 2](docs/tutorials/rp2350_baremetal/index.md)**
  — nine chapters, from an empty directory to the blinking LED: toolchain
  setup, linker scripts, the RP2350 memory map, boot metadata and vector
  tables, the reset handler, register-level GPIO. Where a tutorial listing
  disagrees with the code in the tree (the workspace has been restructured
  since parts of it were written), the tree is authoritative.

- **[Kalman Filter and Navigation](docs/tutorials/nav_kalman/index.md)** —
  a twelve-chapter mathematics tutorial (linear algebra through sensor
  fusion) written for the planned `nav_lib`/`nav_app` modules of the Juno
  flight software. It is self-contained math; none of the code it targets
  exists yet.

The primary hardware references live in `docs/`:
`docs/rp2350-datasheet.pdf` (the microcontroller),
`docs/pico-2-datasheet.pdf` (the board), and extracted register-level
interface documents under `docs/icd/rp2350/` and `docs/icd/pico2_pinout/`,
which the GPIO driver and the tutorial cite by section and page.

## What is implemented and what is planned

**Implemented today:** the three-crate workspace described above — boot path,
GPIO driver, portable traits, the board-definition macro, and the blinky
demo. That is all of the code.

**Planned, documented, not implemented:** everything under `docs/design/`,
`docs/requirements/`, `docs/test_cases/`, `docs/sdp/`, `docs/sprints/`,
`docs/inspections/`, `docs/reviews/`, and `docs/REVIEW.md` is a
planning/specification baseline for the **Juno FT1 flight software** — a
sensor-fusion flight computer with 27 planned modules (GPS, IMU, barometer,
LoRa telemetry, SD logging, navigation, scheduling, and so on). "Juno" and
"Juno FT1 FSW" in those documents are the project name of that planned
system; no module they describe has source code in this repository yet, and
the `tools/*.py` scripts some of them reference are not in this repository
either. Under `docs/icd/`, the device ICDs (GPS, IMU, baro, LoRa, SD,
avionics wiring) likewise describe planned hardware integrations; only the
`rp2350/` and `pico2_pinout/` ICDs relate to implemented code.
