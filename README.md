# rustos — bare-metal Rust on the Raspberry Pi Pico 2 (RP2350)

This repository is a from-scratch, bare-metal Rust project for the Raspberry
Pi Pico 2 board and its RP2350 microcontroller. "Bare metal" means there is no
operating system and no vendor library underneath the code: this workspace
contains the boot metadata the chip's bootrom looks for, the vector table, the
reset handler, the linker script, and a GPIO driver, all written by hand and
traced to datasheet citations. No external embedded crates (`cortex-m`,
`cortex-m-rt`, `rp-hal`, `embassy`, …) are used; nothing outside this
repository is compiled into the firmware image.

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
interfaces) that application code programs against — `Block` (peripheral
start/stop lifecycle), `ErrorType`, `Read`, `Write`, `Gpio`, `GpioPin`, and
the `Pull` enum — plus one concrete type, `Board`, whose `take` constructor
can succeed only once per boot and starts every peripheral block handed to it.
The crate contains no register addresses and no dependency on any other crate
in the workspace, so it compiles for your host machine as well as for the
microcontroller, which is what makes logic written against it testable
without hardware.

**`firmware/pico2/`** is a **library** crate (it has `src/lib.rs` and no
`main.rs`) holding everything that must exist before and around user code:

- `BOOT_INFO`, the RP2350 `IMAGE_DEF` metadata block the bootrom requires;
- the Cortex-M vector table and the `OnReset` reset handler (enable the FPU,
  set `VTOR`, copy `.data` from flash to RAM, zero `.bss`);
- the `entry!` macro, which an application uses to declare its `main` — the
  macro emits the `__rustos_main` symbol the reset handler calls, and
  type-checks `main` against `fn() -> !` at compile time;
- a GPIO driver (`Rp2350Gpio` / `Rp2350GpioPin`) implementing the `api`
  traits;
- `link.ld`, the linker script that places every section in the RP2350's
  memory map, and `build.rs`, which tells Cargo where to find it.

**`demo/`** is the only **binary** crate — a blinky that toggles the Pico 2's
onboard LED (GPIO pin 25). It constructs the `Rp2350Gpio` driver from
`pico2`, takes the `Board` from `api`, and drives the pin through the
portable `Write<bool>` trait. Building the workspace produces the flashable
image at `target/thumbv8m.main-none-eabihf/release/demo`.

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
            FLASH:        7044 B         4 MB      0.17%
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
verified (2026-08-29) to be byte-identical to a fresh `cargo build --release`
and conversion, so you can flash `blinky.uf2` directly without building.

## Host-side tests

The `api` crate is host-compilable, and the alias `cargo xtest` (defined in
`.cargo/config.toml`) runs its test suite on the host instead of the embedded
target. The alias hard-codes the `aarch64-apple-darwin` host triple, so as
written it only runs on Apple-silicon macOS; on another host, substitute your
own triple. The suite currently contains no unit tests — the alias exists as
the seam for them.

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
GPIO driver, portable traits, and the blinky demo. That is all of the code.

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
