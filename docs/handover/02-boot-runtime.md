# 02 Boot and runtime

> This page covers the code that runs between power-on and the application's `main`, and what happens when the program faults or panics. All of it lives in the `pico2` library crate: [`firmware/pico2/src/lib.rs`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L1-L518) (the boot metadata block, the vector table, the reset handler `OnReset`, the `entry!` macro, the fault handlers), [`firmware/pico2/link.ld`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L1-L195) (where every byte goes in the RP2350's memory) and [`firmware/pico2/build.rs`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/build.rs#L1-L18) (how the linker finds `link.ld`). rustos uses no external embedded crates (no `cortex-m-rt`), so every step a C startup file or `cortex-m-rt` would hide is written out by hand here. An application supplies one function, `main`, through `pico2::entry!`. It may also name a *safe state* function that the fault handlers call before they stop. The keyer box firmware in cw-message-gateway uses that function to open the radio's key line on a fault.

Terms used on this page, on first use:

- **RP2350**: the microcontroller on the Raspberry Pi Pico 2 board. It has two Arm Cortex-M33 cores, 520 kB of on-chip SRAM, a 32 kB boot ROM, and no on-chip program flash. The Pico 2 adds a 4 MB external flash chip.
- **Bootrom**: the fixed program in the chip's ROM that runs first after every reset. It finds a valid program in flash and starts it.
- **XIP (execute-in-place)**: the QSPI memory interface (QMI) makes the external flash chip readable as ordinary memory from `0x1000_0000`, so the CPU fetches instructions straight from flash.
- **Vector table**: an array of words at a fixed address. Word 0 is the initial stack pointer. Every later word is the address of the function the CPU runs for one exception or interrupt.
- **VTOR, CPACR, MSPLIM**: Cortex-M33 control registers. VTOR holds the vector table's address, CPACR switches coprocessors such as the floating-point unit (FPU) on, and MSPLIM is the lowest address the main stack may reach.
- **LMA / VMA**: a section's load address (where its bytes are stored in the image) and its run-time address (where code expects to find them).
- **Safe state**: the level that outputs driving something outside the chip, such as a radio's key line, must return to when the program stops.
- **Key line**: the radio input that makes it transmit while it is closed, the contact a Morse key closes. "Keying" is closing it.
- **PTT (push-to-talk)**: the line that switches a radio to transmit.
- **HardFault, UsageFault**: Cortex-M33 fault exceptions. UsageFault reports usage errors such as an undefined instruction, the FPU used while it is off, or a stack-limit violation. HardFault is the catch-all; a fault whose own exception is not enabled escalates to it.
- **Lockup**: the stopped state a core enters when a fault cannot be handled even at HardFault priority. Only a reset ends it.
- **STKOF**: the UsageFault cause bit for a stack-limit (`MSPLIM`) violation.
- **IPSR, ICSR.VECTACTIVE**: registers that hold the number of the exception now running.
- **NVIC**: the Cortex-M33's interrupt controller, one per core. An interrupt reaches its vector-table slot only once it is enabled there.
- **Isolation latch (ISO)**: a per-pad latch on the RP2350 that holds a pad's last state until software releases it. Pads come out of a power-on reset isolated.

The cw-message-gateway glossary ([17 Glossary][gw-17]) has the radio terms. The long-form teaching version of this page is the repository's [bare-metal tutorial](../tutorials/rp2350_baremetal/index.md), chapters 03 to 06. This page is the reference; the tutorial explains the reasoning, shows disassembly, and lists the traps.

## Where it lives

All line ranges are at rustos `master` `80bf599` unless marked PR #6 (rustos PR #6, open, not on master, head `410dffa`).

| Path | What it holds | Lines |
|---|---|---|
| `firmware/pico2/src/lib.rs` | Crate docs, including the boot sequence in prose | [`lib.rs:1-102`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L1-L102) |
| | `#![no_std]` and the crate's public modules: `common`, `clocks`, `timer`, `systick`, `watchdog` ([04 Drivers: core](04-drivers-core.md)); `gpio`, `usb`, `pwm`, `dma`, `adc` ([05 Drivers: GPIO, USB, PWM, ADC, DMA](05-drivers-io.md)); `multicore` at `lib.rs:118` on PR #6 ([06 Multicore](06-multicore.md)). The traits they implement are in the `api` crate ([03 The api crate](03-api-crate.md)) | [`lib.rs:104-117`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L104-L117) |
| | `BOOT_INFO`, the `IMAGE_DEF` block | [`lib.rs:119-160`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L119-L160) |
| | The linker symbols, declared to Rust | [`lib.rs:162-185`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L162-L185) |
| | The `Vector` union (one vector table slot) | [`lib.rs:187-212`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L187-L212) |
| | `PPB_BASE`, `CPACR`, `VTOR`, `CPACR_FPU_FULL` | [`lib.rs:214-231`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L214-L231) |
| | The four reset steps: `enable_fpu`, `reset_vtor`, `reset_data`, `reset_bss` | [`lib.rs:233-332`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L233-L332) |
| | `__rustos_main` and `__rustos_safe_state`, declared | [`lib.rs:334-342`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L334-L342) |
| | The `entry!` macro and `no_safe_state` | [`lib.rs:344-418`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L344-L418) |
| | `OnReset`, `DefaultHandler`, `OnHardFault` | [`lib.rs:420-469`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L420-L469) |
| | `VECTOR_TABLE` | [`lib.rs:471-518`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L471-L518) |
| `firmware/pico2/link.ld` | The linker script: `MEMORY`, `ENTRY`, `SECTIONS`, the stack symbols, five `ASSERT`s | [`link.ld:1-195`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L1-L195) |
| `firmware/pico2/build.rs` | Puts `link.ld` on the linker search path | [`build.rs:14-18`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/build.rs#L14-L18) |
| `Cargo.toml` (workspace) | Members, and `panic = "abort"` for both profiles | [`Cargo.toml:1-12`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/Cargo.toml#L1-L12) |
| `.cargo/config.toml` | The `cargo pico2` alias (builds the library only; no link) | [`.cargo/config.toml:6-7`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/.cargo/config.toml#L6-L7) |
| `templates/pico2/` | The application template: target triple, `-Tlink.ld`, flash runner, a panic handler, `entry!` | [`templates/pico2/.cargo/config.toml:4-13`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/.cargo/config.toml#L4-L13), [`templates/pico2/src/main.rs:15-60`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/src/main.rs#L15-L60) |
| `firmware/pico2/src/common/` | Peripheral base addresses (`reg.rs`), the reset controller (`reset.rs`), the Pico 2 board singleton (`board.rs`), `MAX_GPIO_PIN` (`mod.rs`). Summarised below; covered in [03 The api crate](03-api-crate.md) and [04 Drivers: core](04-drivers-core.md) | [`common/mod.rs:9-11`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/mod.rs#L9-L11) |
| `firmware/pico2/src/lib.rs` (PR #6) | `pub mod multicore`, and the note that both cores share the vector table and fault handlers | [`lib.rs:118`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/lib.rs#L118), [`lib.rs:397-400`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/lib.rs#L397-L400) |
| `firmware/pico2/src/multicore.rs` (PR #6) | Core 1's stack (`Core1Stack`) and its first code (`core1_start`, `core1_run`). Covered in [06 Multicore](06-multicore.md) | [`multicore.rs:246-303`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L246-L303), [`multicore.rs:398-430`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L398-L430) |
| `docs/tutorials/rp2350_baremetal/` | The tutorial: [03 Memory map](../tutorials/rp2350_baremetal/03_memory_map.md), [04 Linker script](../tutorials/rp2350_baremetal/04_the_linker_script.md), [05 Boot metadata and vector table](../tutorials/rp2350_baremetal/05_boot_and_vectors.md), [06 Reset handler](../tutorials/rp2350_baremetal/06_reset_handler.md) | |

The RP2350 datasheet is in the tree as the PDF, [`docs/rp2350-datasheet.pdf`](../rp2350-datasheet.pdf), and as extracted text, [`docs/extracted/rp2350-datasheet.md`](../extracted/rp2350-datasheet.md). The text loses line art, so check anything load-bearing against the PDF. Page numbers in the code are PDF page numbers, one more than the number printed in the page footer (stated in [`multicore.rs:94-95`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L94-L95) on PR #6 and in the tutorial index).

## How it works

### Power-on to `main`

![Power-on to main: the bootrom finds the IMAGE_DEF block, enters through the vector table, OnReset runs its four steps and calls the application's main; below it, the fault path runs the safe state first and then stops, unless a fault the core cannot take locks it up until a reset](diagrams/02-boot-sequence.svg)

*Figure: read the top half top to bottom for a normal boot. Every fault and unexpected exception the core can take is routed to the red box before the program stops. A fault the core cannot take at HardFault priority locks it up instead ("Or: lockup"), before or during the safe state, and only a reset ends that. This page analyses two such cases: a fault inside the safe state, and a stack overflow under `MSPLIM` (analysis, untested; see [Known bugs](#known-bugs-limits-and-flakes)). A panic reaches the red box only if the application's panic handler calls it.*

1. **Reset.** Power-on, the RUN pin or a watchdog reset starts both cores in the bootrom. Core 1 parks in a low-power wait until software on core 0 launches it. rustos on master never launches it; PR #6 adds the code that does ([`multicore.rs:26-37`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L26-L37), PR #6). The rest of this list is core 0.
2. **The bootrom looks for an `IMAGE_DEF`.** It searches the first 4 kB of flash for a valid `IMAGE_DEF` metadata block. If it finds none it does not run the image and falls through to USB mass-storage (BOOTSEL) mode ([`lib.rs:60-62`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L60-L62)). rustos's block is `BOOT_INFO`, five words that mark the image as an executable, Secure, Arm image for the RP2350 ([`lib.rs:152-160`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L152-L160)).
3. **The bootrom enters through the vector table.** `BOOT_INFO` names no entry point, so the bootrom assumes the image starts with a Cortex-M vector table. It loads word 0 into the stack pointer (SP) and jumps to word 1 ([`lib.rs:63-67`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L63-L67), [`link.ld:57-59`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L57-L59)). `link.ld` places `VECTOR_TABLE` at `0x1000_0000`, the first byte of flash. Word 0 is `_stack_top` (`0x2008_2000`) and word 1 is `OnReset`.
4. **`OnReset` runs four steps, then calls the application** ([`lib.rs:436-445`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L436-L445)). At entry, flash is readable over XIP and SP is valid, but RAM holds whatever it held before the reset ([`lib.rs:423-426`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L423-L426)). In order:
   1. `enable_fpu`: switch the FPU on. The target is `thumbv8m.main-none-eabihf` (hard float), so the compiler may emit FPU instructions anywhere, and any FPU instruction before this step faults.
   2. `reset_vtor`: point VTOR at `VECTOR_TABLE`, so that later exceptions dispatch through rustos's table.
   3. `reset_data`: copy the initial values of `.data` (mutable statics with a non-zero initial value) from flash to RAM.
   4. `reset_bss`: zero `.bss` (statics whose initial value is all zeros).
   5. Call `__rustos_main()`, which never returns.
5. **`__rustos_main` calls `main`.** The application's `pico2::entry!(main)` defines `__rustos_main` ([`lib.rs:401-406`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L401-L406)). It checks that `main` has the type `fn() -> !` and calls it.
6. **`main` claims the board and brings up drivers.** The first line is usually `Rp2350::take()`, which works once per boot ([`templates/pico2/src/main.rs:42-45`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/src/main.rs#L42-L45)). The boot path has set up no clocks: until `Rp2350Clocks::new` runs, the chip runs on the ring oscillator, about 11 MHz and drifting ([`clocks.rs:4-12`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L4-L12)). The boot path writes no GPIO and releases no peripheral from reset, so every pin keeps its reset state until `main` configures it. After a power-on reset, pads come out of reset isolated ([`gpio.rs:357-361`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/gpio/gpio.rs#L357-L361)). After a watchdog reset the picture differs: the reset does not clear the isolation latches, so a pad that software had configured (latch released) returns to its reset state, while a pad whose latch is still set keeps the output it had. The datasheet does not state the watchdog case outright, and this is unverified on hardware ([`watchdog.rs:151-183`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L151-L183); see [04 Drivers: core](04-drivers-core.md)). That is one more reason the keyer drives its key and PTT pins low before anything else in `main` ([`firmware/pico2-keyer/src/main.rs:15-18`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L15-L18) in cw-message-gateway). What keeps a radio's key line open during this window is the keyer box's hardware, not rustos. Both parts are never-loosen rules (see [13 Safety rules][gw-13]):
   - the required external 4.7 kΩ pull-down on the key pin. RP2350 erratum E9 means the chip's own pull-down can leave an undriven pad at about 2 V, enough to key the radio ([`docs/keyer.md:59-65`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/docs/keyer.md#L59-L65) in cw-message-gateway);
   - the key switch (a PC817 optocoupler, one transistor, or the photoMOS the doc names), which turns "pin low" into "line open": the box's key pin low means key open. **Never-loosen:** a Pico 2 pin is never wired straight to a radio's key line (the safety audit's NO-GO). The box's safe state is the pin driven low, which would key a radio that keys on a low line ([`docs/keyer.md:966-976`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/docs/keyer.md#L966-L976)).

**CPU state when `main` starts.** What rustos sets, and what it leaves as the bootrom left it:

| Item | State | Source |
|---|---|---|
| Security state | Secure: `BOOT_INFO` word 1 marks the image Secure | [`lib.rs:132`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L132) |
| Privilege | Not stated in the code. Inferred privileged: `OnReset` writes `CPACR` and `VTOR` in the private peripheral block, which "only allows privileged access", and the boot path has run on hardware | datasheet §3.7.4, [`rp2350-datasheet.md:9287-9288`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L9287-L9288) |
| Stack pointer | The main stack pointer (`MSP`), loaded with `_stack_top` from vector word 0. rustos never writes `CONTROL`, so it never switches Thread mode to the process stack (`PSP`). Which stack pointer the bootrom leaves selected is not stated in the code | [`lib.rs:63-67`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L63-L67) |
| `MSPLIM` | Here the Secure main stack limit, `MSPLIM_S`. rustos does not write it, and its value at bootrom exit is not documented in the datasheet's Table 450 | tutorial chapter 06 §6.9 |
| Interrupts and fault enables | rustos writes no `PRIMASK`, `BASEPRI`, `FAULTMASK`, NVIC, `SHCSR` or SAU register and has no `cpsid` or `cpsie`. Its only inline assembly is `dsb`, `isb` and `nop`, and the only Cortex-M33 registers it writes are `CPACR`, `VTOR` and the SysTick block (searched in `firmware/` and `api/` at `80bf599`). At reset no NVIC interrupt is enabled and the configurable faults are off (tutorial chapter 06 §6.3.2) | [`lib.rs:220-226`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L220-L226), [`usb/hw.rs:337-339`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/hw.rs#L337-L339), [`systick.rs:54`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/systick.rs#L54) |
| FPU and `VTOR` | FPU on; `VTOR` = `VECTOR_TABLE` | step 4 above |
| Clock | Ring oscillator, about 11 MHz, until the application sets up the clocks | step 6 above |

### The fault path

Every exception and interrupt goes through the vector table. Slot 3 (HardFault) holds `OnHardFault`. Every other slot that is not the stack pointer, the reset handler or a reserved zero holds `DefaultHandler` ([`lib.rs:508-518`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L508-L518)). Both handlers do the same two things:

1. Call `__rustos_safe_state()`, the application's safe state function named in `entry!` ([`lib.rs:453-457`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L453-L457), [`lib.rs:465-469`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L465-L469)).
2. Spin in `loop {}` forever.

They spin rather than return. Returning from a fault handler re-runs the instruction that faulted. Returning from an unknown interrupt that nobody cleared re-enters the handler at once. The core stays stopped until something resets it: the watchdog, if the application started it and once nothing feeds it ([04 Drivers: core](04-drivers-core.md)), a debugger, or a power cycle. With two cores (PR #6) the other core may still be feeding the watchdog, so the application must stop feeding it when either core faults (step 5 under [The fault path, step by step](#the-fault-path-step-by-step)). A watchdog reset as rustos's watchdog driver sets it up restarts both cores through the bootrom ([`watchdog.rs:45-46`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L45-L46)), so the next boot starts again at step 1 above and `OnReset` initialises `.data` and `.bss` again.

This exists for the keyer box. Its never-loosen rule is that the key line is open in fault handlers (rustos #3, merged; the safety audit's KB-9). The keyer firmware names its `safe_state` in `entry!`, and that function opens the key and PTT lines with one register store before it does anything else ([`firmware/pico2-keyer/src/main.rs:246-261`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L246-L261) in cw-message-gateway). See [08 Changes made for cw-message-gateway](08-changes-for-cw-gateway.md) and the gateway's [08 Keyer box: firmware and protocol][gw-08].

**A panic is not a fault.** `panic!`, a failed `unwrap`, an out-of-bounds index or (in a debug build) an arithmetic overflow calls the application's `#[panic_handler]`, not `OnHardFault`. rustos defines no panic handler; every application must define exactly one ([`templates/pico2/src/main.rs:26-36`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/src/main.rs#L26-L36)). The workspace and the template build with `panic = "abort"`: there is no unwinding ([`Cargo.toml:9-12`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/Cargo.toml#L9-L12), [`templates/pico2/Cargo.toml.liquid:14-18`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/Cargo.toml.liquid#L14-L18)). **The safe state does not run on a panic unless the application's panic handler calls it.** The keyer's panic handler does ([`firmware/pico2-keyer/src/main.rs:282-288`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L282-L288)); the template's does not.

**Two cores (PR #6).** Core 1 gets the same vector table, so a fault on either core runs the same handlers, and the safe state, on the core that faulted, while the other core carries on ([`lib.rs:397-400`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/lib.rs#L397-L400), [`multicore.rs:61-68`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L61-L68), PR #6). `pico2::multicore::core()` tells a safe state which core it is on. Details are in [06 Multicore](06-multicore.md).

### The memory map

![The RP2350 memory map as link.ld uses it: flash from 0x10000000 holds the vector table, the IMAGE_DEF block, code, read-only data and the load image of .data; SRAM from 0x20000000 holds .data, any alignment gap, .bss, an 8 kB stack reservation and the stack growing down from 0x20082000](diagrams/02-memory-map.svg)

*Figure: not to scale. Both columns put high addresses at the top. Notice that the `.stack` section is only a bookkeeping reservation; the real stack starts at the top of RAM and grows down towards `.bss` with nothing in rustos to stop it on core 0. `.bss` can start above the end of `.data` when one of its statics needs more than 4-byte alignment.*

`link.ld` declares two regions ([`link.ld:16-25`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L16-L25)):

- `FLASH`: `0x1000_0000`, 4 MB. This is the XIP window's cached view of the Pico 2's 4 MB flash chip (a Winbond W25Q32RV). The window itself is 16 MB, but a 4 MB chip appears four times inside it, so the length is the chip's size, not the window's ([`link.ld:18-22`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L18-L22)).
- `RAM`: `0x2000_0000`, 520 kB, all ten SRAM banks as one flat region.

Flash, from the bottom: `.vector_table` (272 bytes at `0x1000_0000`), `.boot_info` (20 bytes, at `0x1000_0110` in the shipping build), `.text` (code), `.rodata` (read-only data), then the bytes of `.data` (its load image, at `__sidata`). RAM, from the bottom: `.data`, `.bss`, an 8 kB `.stack` reservation, free space, and the core 0 stack descending from `_stack_top` = `0x2008_2000`. There is no heap: nothing in `link.ld` reserves one and rustos has no allocator ([`lib.rs:34-37`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L34-L37)). Large buffers are statics in `.bss`: the ADC's 16 kB ring ([`adc.rs:521-531`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L521-L531)) and, on PR #6, core 1's stack ([`multicore.rs:265-268`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L265-L268)).

`.bss` does not always start right at `__edata`. `link.ld` asks only for `ALIGN(4)`, but the linker aligns an output section to the largest alignment of its inputs, and some statics need more: the ADC ring is `align(16384)` ([`adc.rs:528`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L528)) and, on PR #6, `Core1Stack` is `align(32)` ([`multicore.rs:269`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L269)). In an image with an ADC ring, up to 16 kB of RAM below the ring can be alignment padding. That padding counts against the headroom `ASSERT`, which measures from `__ebss`. This is the linker's standard rule, not measured on a built image at `80bf599`.

The section and address tables are under [Interfaces and APIs](#the-linker-script-linkld), and the full RP2350 address space is under [The RP2350 memory map as rustos uses it](#the-rp2350-memory-map-as-rustos-uses-it).

## How it connects

Six kinds of neighbour meet this code: the chip (the bootrom and the CPU's exception entry), the build (Cargo, `build.rs` and the linker), the application, the drivers and the board singleton, the multicore driver on PR #6, and the cw-message-gateway firmware images with the keyer's CI. The figure shows each boundary and the contract on it; the table lists every contract with its source.

![What the boot code connects to: at build time the application's linker flag, pico2's build.rs and link.ld, the link, the image and the keyer's check_faults.py; at run time the bootrom and the CPU reading BOOT_INFO and VECTOR_TABLE, OnReset and the fault handlers calling the application, and PR #6's multicore using VECTOR_TABLE and enable_fpu](diagrams/02-connections.svg)

*Figure: build time at the top, run time below. Notice that the application defines both symbols the runtime calls, and that each cw-message-gateway image gets a rustos change only through its own pin.*

| From | To | Contract | Where |
|---|---|---|---|
| Bootrom (chip ROM) | `BOOT_INFO` | Reads five words in the first 4 kB of flash; must be a valid `IMAGE_DEF`; rustos ships the datasheet's minimum Arm one | [`lib.rs:152-160`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L152-L160), [`link.ld:87-90`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L87-L90) |
| Bootrom | `VECTOR_TABLE` words 0 and 1 | Loads SP and PC from the start of flash | [`link.ld:71-74`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L71-L74) |
| CPU, on every exception | `VECTOR_TABLE` through VTOR | Fetches the word at VTOR + 4 × exception number | [`lib.rs:266-271`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L266-L271), [`lib.rs:473-474`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L473-L474) |
| `link.ld` | `lib.rs` | Six symbols with an address and no value: `_stack_top`, `__sidata`, `__sdata`, `__edata`, `__sbss`, `__ebss`; two section names, `.vector_table` and `.boot_info`; `ENTRY(OnReset)` | [`lib.rs:166-185`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L166-L185), [`link.ld:31`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L31) |
| `build.rs` | Cargo and the linker | `cargo:rustc-link-search=<pico2 dir>` so `-Tlink.ld` resolves; relink when `link.ld` changes | [`build.rs:14-18`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/build.rs#L14-L18) |
| `OnReset` | Application | Calls `__rustos_main: extern "Rust" fn() -> !`, defined by `entry!` in the application | [`lib.rs:339-342`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L339-L342) |
| `OnHardFault`, `DefaultHandler` | Application | Call `__rustos_safe_state: extern "Rust" fn()`, defined by `entry!` | [`lib.rs:408-413`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L408-L413) |
| Rust's panic machinery | Application | The application's `#[panic_handler]`; rustos has none | [`templates/pico2/src/main.rs:33-36`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/src/main.rs#L33-L36) |
| `reset_bss` | Drivers and the board singleton | `.bss` statics read zero before `main`. `Rp2350::take()` depends on it: `BOARD_TAKEN` is an `AtomicBool` starting `false`, in `.bss` | [`api/src/device/mod.rs:253`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L253) |
| `multicore` (PR #6) | `lib.rs` | Uses `crate::VECTOR_TABLE` for core 1's VTOR and `crate::enable_fpu` on core 1 | [`multicore.rs:380-384`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L380-L384), [`multicore.rs:423-424`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L423-L424) |
| cw-message-gateway keyer firmware | `entry!`, the handlers | `pico2::entry!(main, safe_state = safe_state)`; its CI reads the built ELF and fails unless, in each of `OnHardFault`, `DefaultHandler` and the panic handler, the first store opens the key and PTT | [`firmware/pico2-keyer/src/main.rs:60`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L60), [`check_faults.py:1-23`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/check_faults.py#L1-L23), [`ci.yml:186`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/.github/workflows/ci.yml#L186) |
| cw-message-gateway decoder image (PR #30, open, not on main) | `entry!`, the handlers | `pico2::entry!(main, safe_state = safe_state)` | [`firmware/pico2-decoder/src/main.rs:45`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/src/main.rs#L45) |

The keyer firmware pins rustos to an exact commit, `8c91507` (master's merge of rustos #4) on cw-message-gateway main ([`firmware/pico2-keyer/Cargo.toml:21-31`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/Cargo.toml#L21-L31)). PR #30's decoder image pins `80bf599` ([`firmware/pico2-decoder/Cargo.toml:24-25`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/Cargo.toml#L24-L25), PR #30), and PR #31's one-Pico image pins `410dffa`, rustos PR #6's head ([`firmware/pico2-keyer/Cargo.toml:30-31`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/Cargo.toml#L30-L31), PR #31, draft). A change on this page reaches an image, and through it a radio, only when that image's pin moves, and that goes through the safety audit (see [Changing it](#changing-it)). The full pin history is on [08 Changes made for cw-message-gateway](08-changes-for-cw-gateway.md).

## Interfaces and APIs

### What an application uses

| Item | Signature or form | What it does | Defined at |
|---|---|---|---|
| `pico2::entry!` | `entry!(path)` or `entry!(path, safe_state = path)` | Declares the application's entry point and, optionally, its safe state. Exported at the crate root, so `pico2::entry!(..)` needs no `use`. | [`lib.rs:395-415`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L395-L415) |
| `pico2::no_safe_state` | `pub fn no_safe_state()` | Does nothing. `entry!(main)` names it as the safe state. | [`lib.rs:418`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L418) |
| `pico2::OnReset` | `#[unsafe(no_mangle)] pub extern "C" fn OnReset() -> !` | The reset handler. Vector slot 1 and the ELF entry point. An application never calls it. | [`lib.rs:436-445`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L436-L445) |
| `pico2::DefaultHandler` | `#[unsafe(no_mangle)] pub extern "C" fn DefaultHandler()` | Every exception without its own handler: safe state, then spin. | [`lib.rs:453-457`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L453-L457) |
| `pico2::OnHardFault` | `#[unsafe(no_mangle)] pub extern "C" fn OnHardFault()` | HardFault (exception 3): safe state, then spin. | [`lib.rs:465-469`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L465-L469) |
| `__rustos_main` | `#[unsafe(no_mangle)] pub extern "Rust" fn __rustos_main() -> !` | Emitted by `entry!` in the application; declared, not defined, in `pico2` | [`lib.rs:340`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L340), [`lib.rs:401-406`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L401-L406) |
| `__rustos_safe_state` | `#[unsafe(no_mangle)] pub extern "Rust" fn __rustos_safe_state()` | Emitted by `entry!`; called by both fault handlers | [`lib.rs:341`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L341), [`lib.rs:408-413`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L408-L413) |
| `#[panic_handler]` | `fn(&core::panic::PanicInfo) -> !` | Must be supplied by the application, exactly once | [`templates/pico2/src/main.rs:33-36`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/src/main.rs#L33-L36) |
| `pico2::multicore::core` (PR #6) | `pub fn core() -> Core` | Which core this runs on; one read of SIO `CPUID`; safe in a fault handler | [`multicore.rs:161-166`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L161-L166) |
| `pico2::multicore::Core1Stack<WORDS>` (PR #6) | `#[repr(C, align(32))] pub struct`; `new()`, `range()`, `MIN_WORDS = 128` | Core 1's stack as a static: `WORDS` eight-byte words, at least 128 (1 kB), a multiple of 4; `launch` checks both at compile time | [`multicore.rs:269-297`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L269-L297), [`multicore.rs:357-366`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L357-L366) |

#### `entry!` in detail

```rust
#[macro_export]
macro_rules! entry {
    ($f:path) => {
        $crate::entry!($f, safe_state = $crate::no_safe_state);
    };
    ($f:path, safe_state = $s:path) => {
        #[unsafe(no_mangle)]
        pub extern "Rust" fn __rustos_main() -> ! {
            let f: fn() -> ! = $f;   // rejects any other signature
            f()
        }
        #[unsafe(no_mangle)]
        pub extern "Rust" fn __rustos_safe_state() {
            let s: fn() = $s;        // rejects any other signature
            s()
        }
    };
}
```

(Quoted from [`lib.rs:395-415`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L395-L415), with comments shortened.)

- **Arguments.** Both arguments are paths to functions: a name such as `main`, or a path such as `crate::io::safe`. A closure is not a path, and the macro does not accept one.
- **Type checks.** The `let f: fn() -> ! = $f;` line makes the compiler prove the signature before the symbol exists. A `main` with arguments or a return value is a `mismatched types` error at the `entry!` line in the application crate ([`lib.rs:366-375`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L366-L375)). The safe state must coerce to `fn()`, so it takes no arguments and returns nothing. By Rust's coercion rules an `unsafe fn` does not coerce to `fn()`, so it is rejected too.
- **Why a macro.** A `no_std` binary cannot have an ordinary `fn main`. The application declares `#![no_main]`, and the runtime reaches it through a named symbol. A bare `extern` declaration would not be type-checked across the link; the macro is what adds the check ([`lib.rs:352-364`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L352-L364)).
- **Once per binary.** Both generated functions are `#[unsafe(no_mangle)]` (the crate is edition 2024, which requires the `unsafe(...)` form, [`firmware/pico2/Cargo.toml:4`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/Cargo.toml#L4)), so the final link must contain exactly one `entry!` expansion. With none, the link has an undefined `__rustos_main`. With two, there are two definitions of one symbol. (This is standard compiler and linker behaviour; the repository does not demonstrate it.)

#### Rules for a safe state function

From the `entry!` docs ([`lib.rs:380-394`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L380-L394)):

- It runs inside a fault handler, with the program in whatever state faulted it. It should only write the registers it needs: no locks, no allocation, no driver state.
- A fault inside it, when it was called from the HardFault handler, locks the processor up. Only a reset ends a lockup, as only a reset ends the handler's own loop.
- It may run twice, nested. Called from `DefaultHandler` for an interrupt or a configurable fault, or from the panic handler, a fault inside it escalates to HardFault, whose handler calls it again. A panic inside it calls the panic handler, which in the pattern below calls it again, until the stack runs out. Make it idempotent (plain stores) and unable to panic (no indexing, `unwrap` or arithmetic that can overflow). This is analysis from the Armv8-M priority rules, not tested.
- On PR #6 it may run on either core ([`lib.rs:397-400`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/lib.rs#L397-L400)).
- It may run before `main` has configured anything (a fault early in `main`). Code that touches a peripheral must cope with the peripheral still being held in reset. `pico2::common::reset::is_reset_done` exists for that ([`reset.rs:211-228`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/reset.rs#L211-L228)), and `Rp2350Pwm::silence_all` uses it ([`pwm.rs:709-728`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L709-L728)).
- Anything that keys a transmitter goes first, in one store, before anything that could delay it. That rule belongs to the keyer firmware and is checked by its CI, not by rustos. `silence_all`'s docs repeat it for rustos callers ([`pwm.rs:704-707`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L704-L707)).

### `BOOT_INFO`: the `IMAGE_DEF` block

`static BOOT_INFO: [u32; 5]`, `#[used]`, in section `.boot_info` ([`lib.rs:152-160`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L152-L160)). These are the five words of the minimum valid Arm `IMAGE_DEF`, copied verbatim from the datasheet, §5.9.5.1 (PDF p427) ([`lib.rs:126-135`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L126-L135)).

| Word | Value | Meaning |
|---|---|---|
| 0 | `0xffffded3` | `PICOBIN_BLOCK_MARKER_START` |
| 1 | `0x10210142` | Item `0x42` = `IMAGE_TYPE`, 1 word; flags `0x1021` = executable, Secure, Arm, RP2350 |
| 2 | `0x000001ff` | Item `0xff` = `LAST`; the other items total 1 word |
| 3 | `0x00000000` | Link to the next block; 0 links to itself (a loop of one block) |
| 4 | `0xab123579` | `PICOBIN_BLOCK_MARKER_END` |

The flags in word 1 decode as: bits 3:0 = 1 (executable), bits 5:4 = 2 (Secure), bits 10:8 = 0 (Arm), bits 14:12 = 1 (RP2350). If bits 10:8 were wrong, the bootrom would hand the image to the RISC-V cores, with no error. The tutorial's [chapter 05](../tutorials/rp2350_baremetal/05_boot_and_vectors.md) §5.3 has the bit-by-bit decode.

`#[used]` keeps rustc from dropping a static nothing reads. The section name must match `link.ld`, which keeps the section with `KEEP` so the linker's garbage collection (`--gc-sections`) cannot drop it ([`lib.rs:145-151`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L145-L151)).

### `VECTOR_TABLE`

`static VECTOR_TABLE: [Vector; 68]`, `#[used]`, in section `.vector_table`, built at compile time in a `const` block ([`lib.rs:506-518`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L506-L518)). `Vector` is a `#[repr(C)]` union of four word-sized fields, so each slot can hold the right kind of value ([`lib.rs:195-206`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L195-L206)):

| `Vector` field | Type | Used for |
|---|---|---|
| `handler` | `unsafe extern "C" fn()` | Ordinary exception and interrupt handlers |
| `reset` | `unsafe extern "C" fn() -> !` | Slot 1 only |
| `stack_top` | `*const u32` | Slot 0 only |
| `reserved` | `u32` | Reserved slots, which must read zero |

`unsafe impl Sync for Vector` is needed because of the raw pointer field. It is sound because the table is immutable, lives in flash and is only read by the hardware ([`lib.rs:208-212`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L208-L212)).

| Slot | Exception | Contents | Note |
|---|---|---|---|
| 0 | (initial SP) | `&raw const _stack_top` = `0x2008_2000` | Not a handler |
| 1 | Reset | `OnReset` | |
| 2 | NMI | `DefaultHandler` | Nothing routes to NMI at reset: `NMI_MASK0` resets to 0 (datasheet Table 362, tutorial chapter 06 §6.3.2) |
| 3 | HardFault | `OnHardFault` | |
| 4-7 | MemManage, BusFault, UsageFault, SecureFault | `DefaultHandler` | Disabled at reset, so they escalate to HardFault instead (see Known bugs) |
| 8-10 | reserved | `0` | |
| 11, 12 | SVCall, DebugMonitor | `DefaultHandler` | |
| 13 | reserved | `0` | |
| 14, 15 | PendSV, SysTick | `DefaultHandler` | rustos's SysTick driver runs with no interrupt |
| 16-67 | IRQ0-IRQ51 | `DefaultHandler` | 52 RP2350 interrupt inputs (datasheet §3.2); IRQ46-51 are `SPAREIRQ_IRQ_0..5`, never fired by hardware |

Sources: the table doc ([`lib.rs:476-495`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L476-L495)), the construction ([`lib.rs:509-516`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L509-L516)) and, for SysTick, [`systick.rs:26-32`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/systick.rs#L26-L32). Rust function pointers carry the Thumb bit (bit 0 set) automatically, which every populated Armv8-M vector needs (tutorial chapter 05 §5.6.4).

No rustos driver on master enables an interrupt: the SysTick, DMA, ADC, PWM and USB drivers each say they leave theirs off ([`systick.rs:26-32`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/systick.rs#L26-L32), [`dma.rs:53`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/dma.rs#L53), [`adc.rs:138`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L138), [`pwm.rs:194`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L194), [`usb/mod.rs:216`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L216)), and no code writes the interrupt controller (NVIC). So slots 11-67 (SVCall, DebugMonitor, PendSV, SysTick and the 52 interrupts) are reached only if an application raises or enables one of them itself. There is no API to install a handler: the table is a fixed `static` in flash, so a real interrupt handler means editing `VECTOR_TABLE` (see [Adding an interrupt handler](#adding-an-interrupt-handler)).

### Registers the boot path writes

| Register | Address | Write | Constant | Defined at | Datasheet |
|---|---|---|---|---|---|
| `CPACR` (Coprocessor Access Control) | `0xE000_ED88` (`PPB_BASE + 0x0ED88`) | Read-modify-write: `|= 0x00F0_0000`, full access for CP10 and CP11 (the FPU); other fields kept | `CPACR_FPU_FULL = (0b11 << 20) \| (0b11 << 22)` | [`lib.rs:223`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L223), [`lib.rs:231`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L231) | §3.7, Table 229, [`rp2350-datasheet.md:13941-13958`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L13941-L13958) |
| `VTOR` (Vector Table Offset) | `0xE000_ED08` (`PPB_BASE + 0x0ED08`) | Plain write: `&VECTOR_TABLE` (`0x1000_0000`) | | [`lib.rs:226`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L226) | Table 201, `TBLOFF` bits 31:7, [`rp2350-datasheet.md:13021-13042`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L13021-L13042) |

`PPB_BASE = 0xE000_0000` is the Cortex-M33 private peripheral block, the same on every Cortex-M part (datasheet §3.7.5, [`rp2350-datasheet.md:10412-10413`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L10412-L10413); [`lib.rs:214-220`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L214-L220)). These registers are per core: on PR #6, core 1 writes its own `CPACR` ([`multicore.rs:52-55`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L52-L55)).

### The linker script, `link.ld`

#### Regions and directives

| Item | Value | Meaning | Line |
|---|---|---|---|
| `FLASH (rx)` | `ORIGIN = 0x10000000, LENGTH = 4M` | The XIP cached window over the 4 MB flash chip. 4M is the device size, not the 16 MB window | [`link.ld:23`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L23) |
| `RAM (rwx)` | `ORIGIN = 0x20000000, LENGTH = 520K` | SRAM0-9 as one region; ends at `0x2008_2000` | [`link.ld:24`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L24) |
| `ENTRY(OnReset)` | | The ELF entry point, and the root the linker's garbage collection traces from. Without it the linker deletes the program | [`link.ld:31`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L31) |
| `_min_stack_size` | `8K` | The smallest stack the build accepts | [`link.ld:35`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L35) |

The `(rx)` and `(rwx)` attributes enforce nothing. They only steer where unclaimed sections go ([`link.ld:13-14`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L13-L14)).

#### Sections, in address order

The order in `SECTIONS` is the order in flash ([`link.ld:39-42`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L39-L42)).

| Section | Region (run / load) | Input | Align | Size or start | Notes | Lines |
|---|---|---|---|---|---|---|
| `.vector_table` | FLASH | `KEEP(*(.vector_table))` | 512 | At `ORIGIN(FLASH)`; 68 × 4 = 272 bytes (`0x110`) | The Armv8-M VTOR alignment rule needs 512 (entries rounded up to 128, × 4); the RP2350's VTOR alone would accept 128 | [`link.ld:71-74`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L71-L74) |
| `.boot_info` | FLASH | `KEEP(*(.boot_info))` | 4 | 20 bytes, at `0x1000_0110` in the shipping build | Must start in the first 4 kB | [`link.ld:87-90`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L87-L90) |
| `.text` | FLASH | `*(.text .text.*)` | 4 at the end | From `0x1000_0124` in the shipping build | Code, run in place over XIP. Rust puts each function in its own `.text.<name>` section | [`link.ld:105-108`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L105-L108) |
| `.rodata` | FLASH | `*(.rodata .rodata.*)` | 4 at the end | | Read-only statics, string literals, panic messages. A stray write to flash is silently dropped, not faulted | [`link.ld:122-125`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L122-L125) |
| `.data` | RAM / FLASH (`> RAM AT > FLASH`) | `*(.data .data.*)` | 4 | Runs at `0x2000_0000`; loads at `__sidata` | Mutable statics with a non-zero initial value; `reset_data` copies them | [`link.ld:152-158`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L152-L158) |
| `.bss` | RAM, `NOLOAD` | `*(.bss .bss.*)` | 4 in `link.ld`; more if an input needs it | After `.data`, at the next multiple of the largest input alignment (16 kB if an `AdcRing` is present; see [The memory map](#the-memory-map)) | Zero-initialised statics; no bytes in the image; `reset_bss` zeroes them | [`link.ld:160-165`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L160-L165) |
| `.stack` | RAM, `NOLOAD` | none: `. = . + _min_stack_size` | 8 | 8 kB, right after `.bss` | Bookkeeping only. It makes `--print-memory-usage` and the region check count the stack. The real stack is at the top of RAM | [`link.ld:167-176`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L167-L176) |
| `/DISCARD/` | | `*(.ARM.exidx .ARM.exidx.*)` | | | Stack-unwinding tables, useless with `panic = "abort"` | [`link.ld:177`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L177) |

The "shipping build" addresses are the tutorial's measured output for the blinky application ([chapter 04](../tutorials/rp2350_baremetal/04_the_linker_script.md) §4.11.2). The start of `.boot_info` and `.text` follows from the sizes above. Everything after `.text` moves with code size.

#### Symbols

These symbols have an address and no value. Declare them in Rust as `unsafe extern "C" { static NAME: u32; }` and only ever take `&raw const NAME`; reading one as a value reads whatever bytes are at that address ([`lib.rs:162-165`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L162-L165), [`link.ld:148-150`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L148-L150)).

| Symbol | Definition | Value | Used by |
|---|---|---|---|
| `_stack_top` | `ORIGIN(RAM) + LENGTH(RAM)` ([`link.ld:183`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L183)) | `0x2008_2000`, one past the last RAM byte; never dereferenced | `VECTOR_TABLE[0]` ([`lib.rs:171`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L171)) |
| `__sidata` | `LOADADDR(.data)` ([`link.ld:158`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L158)) | Where `.data`'s initial bytes sit in flash | `reset_data` source ([`lib.rs:176`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L176)) |
| `__sdata`, `__edata` | `.` before and after `.data`'s input ([`link.ld:153-156`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L153-L156)) | `.data`'s bounds in RAM | `reset_data` destination and length ([`lib.rs:178-180`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L178-L180)) |
| `__sbss`, `__ebss` | `.` before and after `.bss`'s input ([`link.ld:161-164`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L161-L164)) | `.bss`'s bounds in RAM | `reset_bss` ([`lib.rs:182-184`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L182-L184)); also the keyer's core 0 stack limit on cw-message-gateway PR #31 (draft), see [Programming against it](#bound-the-core-0-stack) |
| `_min_stack_size` | `8K` | `0x2000` | The `.stack` section and the last `ASSERT` |

#### Assertions

Each one stops the build instead of producing a board that boots to nothing ([`link.ld:186-195`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L186-L195)):

| Assertion | Message | Guards against |
|---|---|---|
| `ADDR(.vector_table) == ORIGIN(FLASH)` | vector table must be at flash base (datasheet 5.9.3.3) | The bootrom reading something else as SP and PC |
| `ADDR(.vector_table) % 512 == 0` | vector table violates VTOR alignment | A table VTOR cannot point at |
| `ADDR(.boot_info) - ORIGIN(FLASH) < 4096` | IMAGE_DEF must be in the first 4kB (datasheet 5.9) | The bootrom not finding the block and refusing the image |
| `ADDR(.boot_info) % 4 == 0` | boot block must start word-aligned (datasheet 5.9.1) | A malformed block |
| `_stack_top - __ebss >= _min_stack_size` | insufficient headroom between .bss and the stack | `.bss` growing into the stack's 8 kB budget |

### `build.rs`

Runs on the host before `pico2` compiles and prints two Cargo directives ([`build.rs:14-18`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/build.rs#L14-L18)):

| Directive | Effect |
|---|---|
| `cargo:rustc-link-search=$CARGO_MANIFEST_DIR` | Adds `firmware/pico2/` to the linker search path. Search paths from a dependency's build script also apply when the final binary links, so an application's `-C link-arg=-Tlink.ld` finds this file |
| `cargo:rerun-if-changed=link.ld` | Rebuilds and relinks when `link.ld` changes, even if no Rust source changed |

### Build settings an application needs

`pico2` supplies the linker script; the application supplies the flags. The template has them ([`templates/pico2/.cargo/config.toml:4-13`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/.cargo/config.toml#L4-L13)):

| Setting | Value | Why |
|---|---|---|
| `[build] target` | `thumbv8m.main-none-eabihf` | Cortex-M33 with hardware floating point |
| `rustflags` | `-C link-arg=-Tlink.ld` | Use rustos's linker script |
| | `-C link-arg=--print-memory-usage` | Print the FLASH and RAM report on every build |
| | `-C link-arg=-Map=firmware.map` | Write a map file |
| `runner` | `picotool load -u -x -t elf` | `cargo run` flashes and starts the image |
| `[profile.*] panic` | `"abort"` | There is no unwinder ([`templates/pico2/Cargo.toml.liquid:14-18`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/Cargo.toml.liquid#L14-L18)) |
| crate attributes | `#![no_std]`, `#![no_main]` | Required ([`templates/pico2/src/main.rs:15-16`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/src/main.rs#L15-L16)) |

Inside the rustos workspace itself there is no application and no link. `cargo pico2` compiles the library for the target ([`.cargo/config.toml:6-7`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/.cargo/config.toml#L6-L7)), and a plain `cargo build` builds only `api` for the host ([`Cargo.toml:3-7`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/Cargo.toml#L3-L7)).

### `common/`: what `main` reaches first

`firmware/pico2/src/common/` holds facts every driver needs. They do not run at boot, but the first lines of every `main` use them.

| Item | What it is | Defined at | Covered in |
|---|---|---|---|
| `Rp2350` (with `Rp2350Pins`), `Rp2350::take() -> Option<Rp2350>` | The board singleton made by `api::define_board!`: one `PinHandle<N>` per Pico 2 pin and one `DeviceHandle<T>` per driver. `take` succeeds once per boot | [`board.rs:97-146`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/board.rs#L97-L146) | [03 The api crate](03-api-crate.md) |
| `RegAddr` | Base addresses of every peripheral block rustos uses | [`reg.rs:57`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/reg.rs#L57) | [04 Drivers: core](04-drivers-core.md) |
| `clr_reset_reg`, `set_reset_reg`, `wait_for_reset_done`, `is_reset_done`, `cycle_reset` | The `RESETS` controller. Almost every peripheral starts held in reset | [`reset.rs:129-251`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/reset.rs#L129-L251) | [04 Drivers: core](04-drivers-core.md) |
| `MAX_GPIO_PIN = 30` | GPIO count of the RP2350A package on the Pico 2 | [`mod.rs:40`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/mod.rs#L40) | [05 Drivers: GPIO, USB, PWM, ADC, DMA](05-drivers-io.md) |

One fact from `reset.rs` belongs on this page because it can stop the boot image dead: `RESETS` bits 7 (`IO_QSPI`) and 10 (`PADS_QSPI`) must never be set on an image running from flash. Those blocks drive the flash pins that XIP fetches every instruction over, so asserting them stops the CPU mid-fetch with no fault ([`reset.rs:157-164`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/reset.rs#L157-L164)).

### The RP2350 memory map as rustos uses it

| Range | What | rustos uses it for | Source |
|---|---|---|---|
| `0x0000_0000` | Boot ROM, 32 kB of mask ROM | The bootrom; rustos never calls into it on master | the size: datasheet chapter 5, [`rp2350-datasheet.md:26256`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L26256); the address: tutorial [chapter 03](../tutorials/rp2350_baremetal/03_memory_map.md) §3.1 |
| `0x1000_0000`-`0x103F_FFFF` | XIP, cached view of the 4 MB flash | `FLASH` region: all code and constants | [`link.ld:23`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L23) |
| `0x2000_0000`-`0x2003_FFFF` | SRAM0-3, 256 kB, striped across 4 banks | `RAM` region (`.data`, `.bss`, free) | datasheet §2.2.3 Table 10 |
| `0x2004_0000`-`0x2007_FFFF` | SRAM4-7, 256 kB, striped | `RAM` region | datasheet §2.2.3 Table 10 |
| `0x2008_0000`-`0x2008_0FFF` | SRAM8, 4 kB, not striped | `RAM` region; the deeper part of an 8 kB core 0 stack | datasheet §2.2.3 Table 11 |
| `0x2008_1000`-`0x2008_1FFF` | SRAM9, 4 kB, not striped | `RAM` region; the first 4 kB of the core 0 stack | datasheet §2.2.3 Table 11 |
| `0x2008_2000` | `SRAM_END`, unmapped | `_stack_top`; a read or write here would be a bus fault | [`link.ld:179-183`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L179-L183) |
| `0x4000_0000`... | APB peripherals (`SYSCFG`, `CLOCKS`, `PSM`, `RESETS`, `IO_BANK0`, `PADS_BANK0`, `XOSC`, PLLs, `ADC`, `PWM`, `TIMER0`, `WATCHDOG`, `POWMAN`, `TICKS`) | Drivers | [`reg.rs:57-126`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/reg.rs#L57-L126) |
| `0x5000_0000`... | AHB peripherals (`DMA`, `USBCTRL_DPRAM`, `USBCTRL_REGS`) | Drivers | [`reg.rs:128-141`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/reg.rs#L128-L141) |
| `0xD000_0000` | SIO, reached by each core over its own port: GPIO set/clear (shared by both cores), each core's ends of the inter-core FIFOs, `CPUID` (differs per core) | GPIO driver; PR #6 multicore | [`reg.rs:143-154`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/reg.rs#L143-L154), [`multicore.rs:110-115`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L110-L115) (PR #6) |
| `0xE000_0000` | Cortex-M33 private peripheral block (PPB), per core | `CPACR`, `VTOR`, SysTick | [`lib.rs:220`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L220) |

Instructions can be fetched only from ROM, XIP and SRAM, so those are the only places a program can live, and the reason `link.ld` has exactly two regions (tutorial chapter 03 §3.1). The two 4 kB banks at the top are the ones the datasheet suggests for stacks, and rustos's core 0 stack starts there because `_stack_top` is the top of RAM.

## Algorithms and processing

### `OnReset`, step by step

| Step | Code | Exactly what it does | Must happen before | Line |
|---|---|---|---|---|
| 1 | `enable_fpu()` | `v = CPACR.read_volatile(); CPACR.write_volatile(v \| 0x00F0_0000); dsb; isb` | Any FPU instruction. Before this, an FPU instruction raises a UsageFault (`NOCP`), which escalates to HardFault | [`lib.rs:257-264`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L257-L264) |
| 2 | `reset_vtor()` | `VTOR.write_volatile(&raw const VECTOR_TABLE as u32); dsb; isb` | Any exception. The bootrom entered through the table but does not necessarily leave VTOR pointing at it | [`lib.rs:284-289`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L284-L289) |
| 3 | `reset_data()` | `count = (&__edata − &__sdata) / 4`; `copy_nonoverlapping(&__sidata, &__sdata, count)` (whole `u32` words) | Any read of an initialised static | [`lib.rs:307-313`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L307-L313) |
| 4 | `reset_bss()` | `count = (&__ebss − &__sbss) / 4`; `(&__sbss).write_bytes(0, count)` | Any read of a zero-initialised static (`Rp2350::take` reads one) | [`lib.rs:327-332`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L327-L332) |
| 5 | `__rustos_main()` | Calls the application; type `-> !`, so `OnReset` needs no trailing `loop {}` | | [`lib.rs:443`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L443) |

Points that matter when reading or changing this code:

- **Volatile accesses.** `read_volatile` and `write_volatile` make the compiler perform each access exactly as written ([`lib.rs:242-246`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L242-L246)).
- **Barriers.** The `dsb`/`isb` pair after each control-register write makes sure the write has taken effect before the next instruction is fetched ([`lib.rs:247-250`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L247-L250)).
- **Read-modify-write against plain write.** `CPACR` is read-modify-write because it holds other coprocessors' fields (CP0, CP4, CP5, CP7). `VTOR` is a plain write because the whole register is the address.
- **The copy count comes from the RAM pair.** `__edata − __sdata` is the length. Using `__edata − __sidata` would subtract a flash address from a RAM address and copy about 268 MB over everything (tutorial chapter 06 §6.6).
- **Word arithmetic.** The counts are in `u32` elements, so both divide by 4. The `ALIGN(4)`s in `link.ld` make that division exact. The plain `usize` subtraction is deliberate: `offset_from` with `unwrap` would put a panic path (and about 3.6 kB of formatting code) into the reset handler (tutorial chapter 06 §6.6).
- **Empty sections.** The tutorial's chapter 04 §4.11.2 listing shows `.data` and `.bss` both empty. Its chapters 06 §6.5.1 and 08 report the finished blinky's `.bss` holding the 4-byte board flag (RAM 8200 B, also in [`README.md:298-300`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/README.md#L298-L300) at `80bf599`). At `80bf599`, any application that calls `Rp2350::take()` has `BOARD_TAKEN` in `.bss` ([`api/src/device/mod.rs:253`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L253)), so `reset_bss` always zeroes at least one word. `.data` is empty unless the application has a mutable static with a non-zero initial value.
- **Inlining.** All four helpers are `#[inline]`. In a release build they fold into `OnReset`. A debug build keeps them as functions and adds overflow and precondition checks that could panic before `.bss` is zeroed (tutorial chapter 06 §6.8). Use `--release` for bring-up.

### The vector table at compile time

The table is a `static` initialised by a `const` block: fill all 68 slots with `DefaultHandler`, then overwrite slots 0, 1 and 3 and zero slots 8, 9, 10 and 13 ([`lib.rs:509-516`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L509-L516)). The compiler emits it as plain flash contents; nothing runs to build it. `&raw const _stack_top` is resolved by the linker, so the source never hard-codes `0x2008_2000`.

### The stack

- **Core 0's stack** is the main stack. It starts at `_stack_top` = `0x2008_2000` and grows down. The Cortex-M33 stack is full-descending: SP is decremented before each store, so the first push writes `0x2008_1FFC` and the unmapped address `0x2008_2000` is never touched ([`link.ld:179-183`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L179-L183)).
- **The `.stack` section** books 8 kB directly above `.bss` so the linker's region check and memory report count a stack ([`link.ld:167-170`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L167-L170)). Together with the last `ASSERT`, it means a build whose statics leave less than 8 kB fails to link. It reserves the right amount in the wrong place: the stack's real position is at the top of RAM, and the bytes between `.stack` and the stack pointer are shared, unguarded space.
- **No guard on core 0 in rustos.** rustos never writes core 0's `MSPLIM`. A stack deeper than the free space walks down into `.bss` and corrupts statics with no fault (tutorial chapter 04 §4.7.2). The keyer firmware on cw-message-gateway PR #31 (draft) sets core 0's `MSPLIM` itself, from `main`; what an overflow then does is under [Bound the core 0 stack](#bound-the-core-0-stack).
- **Alignment.** `.stack` is `ALIGN(8)` because the Arm procedure call standard (AAPCS) needs an 8-byte-aligned SP at public interfaces ([`link.ld:171-173`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L171-L173)); `0x2008_2000` is 8-aligned.
- **Core 1's stack (PR #6)** is an application `static` of type `Core1Stack<WORDS>`, so it lives in `.bss` and the headroom `ASSERT` counts it. `launch` sends the bootrom the stack's top as core 1's initial SP. Core 1's first code, `core1_start`, is a naked function (no prologue, so no push). It loads the stack's bottom into `MSPLIM` before anything touches the stack, so a core 1 overflow faults instead of writing below it ([`multicore.rs:398-418`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L398-L418)). Then `core1_run` enables core 1's FPU and calls the application's core 1 function ([`multicore.rs:420-430`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L420-L430)). `.data` and `.bss` are not set up again on core 1 ([`multicore.rs:58-59`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L58-L59)).

### The fault path, step by step

1. An exception becomes active: a fault, NMI or an enabled interrupt.
2. The CPU pushes an exception frame on the current stack and fetches the handler address at VTOR + 4 × the exception number.
3. MemManage, BusFault, UsageFault and SecureFault are disabled at reset (`SHCSR` bits 16-19), so they escalate to HardFault (tutorial chapter 06 §6.9). In practice almost every fault arrives at `OnHardFault`.
4. The handler calls `__rustos_safe_state()`, which calls the function named in `entry!` (or `no_safe_state`).
5. The handler spins in `loop {}`. The core stays there until a reset. The watchdog, if started, resets the chip once nothing feeds it. With two cores (PR #6) the other core may still be feeding it; the application must stop feeding when one core faults. The keyer on cw-message-gateway PR #31 does, through a fault flag its safe state sets and core 0 reads ([`firmware/pico2-keyer/src/main.rs:343-354`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L343-L354), PR #31, draft).
6. On PR #6 with core 1 running, the other core keeps running. The application decides what it does about it. The keyer firmware on cw-message-gateway PR #31 has core 0 trip the box when core 1 faults ([`firmware/pico2-keyer/src/core1.rs:30-35`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/core1.rs#L30-L35)); see the gateway's [11 Decoder box and the one-Pico image][gw-11].

**Telling handlers apart.** `DefaultHandler` and `OnHardFault` have identical bodies. The tutorial measured that a release build folds such identical functions into one address, so a debugger stopped in either shows the same PC (tutorial chapter 05 §5.8). That measurement predates the safe-state call. The two bodies are still identical in source (both call `__rustos_safe_state()` then loop), so the same fold is expected; this has not been re-measured at `80bf599`. To find which exception fired and why, see [Debugging a stopped handler](#debugging-a-stopped-handler).

### Debugging a stopped handler

With a debugger halted in a handler's `loop {}`, these registers say what happened. They are in the private peripheral block and are per core. rustos reads none of them; the addresses and fields are the datasheet's.

| Register | Address | Read | Datasheet |
|---|---|---|---|
| `ICSR` | `0xE000_ED04` | `VECTACTIVE`, bits 8:0: the number of the exception now running. 3 is HardFault; 16 + n is IRQ n. The core's `IPSR` holds the same number | [`rp2350-datasheet.md:10786`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L10786), [`rp2350-datasheet.md:13018`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L13018) |
| `HFSR` | `0xE000_ED2C` | `FORCED`, bit 30: a configurable-priority fault was escalated to HardFault. With the configurable faults disabled (step 3 above) that is the usual case | [`rp2350-datasheet.md:10817`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L10817), [`rp2350-datasheet.md:13451-13465`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L13451-L13465) |
| `CFSR` | `0xE000_ED28` | The cause of the escalated fault: `UFSR` (bits 31:16; for example `STKOF`, bit 20, a stack-limit violation, and `NOCP`, bit 19, the FPU used while off), `BFSR` (bits 15:8, bus faults) and `MMFSR` (bits 7:0, memory-protection faults) | [`rp2350-datasheet.md:10806-10815`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L10806-L10815), [`rp2350-datasheet.md:13375-13405`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L13375-L13405) |

The stacked return address says where the program was. On entry the CPU pushed its State context on the stack in use, the main stack unless something selected the process stack (rustos never writes `CONTROL`; see the CPU state table under [Power-on to `main`](#power-on-to-main)): `R0`, `R1`, `R2`, `R3`, `R12`, `LR`, `ReturnAddress`, `RETPSR`, from the lowest address up. The return address is therefore the 7th word, offset `0x18` from the frame's start ([`DDI0553B_z_armv8m_arm.md:4747-4748`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/DDI0553B_z_armv8m_arm.md#L4747-L4748), rule RWYWW, and the frame figure at [`DDI0553B_z_armv8m_arm.md:4875-4895`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/DDI0553B_z_armv8m_arm.md#L4875-L4895)). A floating-point context, when stacked, sits above those eight words. An additional state context, which would sit below them, is stacked only for some Security-state transitions (rule RWYWW); this image runs Secure only, so none is expected (analysis, not measured). The frame starts at the stack pointer the handler was entered with. The handler's prologue pushes below that, so add the size of those pushes to the `SP` you read. `llvm-objdump -d --disassemble-symbols=OnHardFault` shows them; the release build the tutorial measured began with `push {r7, lr}`, 8 bytes (tutorial chapter 05 §5.8, measured before the safe-state call was added).

## Programming against it

### A minimal application

The template is the smallest complete program. This is [`templates/pico2/src/main.rs:15-60`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/src/main.rs#L15-L60) with its comments removed:

```rust
#![no_std]
#![no_main]

use core::hint::spin_loop;
use core::panic::PanicInfo;

use api::gpio::Gpio;
use api::common::Write;
use pico2::common::board::Rp2350;
use pico2::gpio::gpio::Rp2350Gpio;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

pico2::entry!(main);

fn main() -> ! {
    let board = Rp2350::take().unwrap();
    let mut gpio = Rp2350Gpio::new(board.gpio);
    let mut led = gpio.output_from_handle(board.pins.led).unwrap();
    loop {
        led.write(true);
        delay();
        led.write(false);
        delay();
    }
}

fn delay() {
    for _ in 0..5_000_000 {
        spin_loop();
    }
}
```

The template starts neither the clocks nor the timer, so `delay` is a busy-wait calibrated by eye. Without it the LED toggles too fast to see as a blink. The template's comment "There is no configured timer yet" ([`templates/pico2/src/main.rs:54-55`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/src/main.rs#L54-L55)) predates `Rp2350Timer`, which `pico2` has had since rustos #1 (item W5 on [09 Work left and known issues](09-work-left-known-issues.md)). For a timed delay, start `Rp2350Clocks` and `Rp2350Timer` and call `timer.delay_ms(..)` ([`firmware/pico2/src/timer.rs:232-252`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/timer.rs#L232-L252)); the fuller program on [01 Overview](01-overview.md) starts both.

Generate a project with `cargo generate --git https://github.com/robinonsay/rustos templates/pico2` ([`templates/README.md:11-23`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/README.md#L11-L23)). Then hold BOOTSEL while plugging the Pico 2 in and run `cargo run --release`, which hands the ELF to `picotool load -u -x -t elf` ([`templates/pico2/.cargo/config.toml:13`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/.cargo/config.toml#L13)). The template's comment also offers "have picotool reboot a running image" ([`templates/pico2/src/main.rs:8-13`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/src/main.rs#L8-L13)), but a rustos image has no picotool reset interface: the blinky starts no USB, and rustos's USB device has only its two CDC interfaces ([`api/src/usb/descriptor.rs:52-57`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L52-L57)); see [01 Overview](01-overview.md). Use a spare Pico 2 wired to nothing, never the keyer box or any Pico 2 wired to a radio; only the audit-cleared CI UF2 goes on the box (the gateway's [13 Safety rules][gw-13], rules 7.1 and 7.2). To build against a local rustos checkout, add the `[patch]` section in [`templates/README.md:42-46`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/README.md#L42-L46).

### Adding a safe state

Name a `fn()` in `entry!` and call the same function from the panic handler, because rustos does not call it on a panic. Write registers directly; touch no driver. This is the keyer's pattern, trimmed ([`firmware/pico2-keyer/src/main.rs:246-288`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L246-L288) in cw-message-gateway). The keyer opens the key, GP16, and the PTT, GP17, in one store, `(1 << KEY_GPIO) | (1 << PTT_GPIO)` ([`main.rs:260`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L260)), and its CI fails the build unless that first store carries both bits ([`check_faults.py:37-40`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/check_faults.py#L37-L40)). The trim keeps that store and drops the keyer's `tone_low`:

```rust
pico2::entry!(main, safe_state = safe_state);

const KEY_GPIO: usize = 16;
const PTT_GPIO: usize = 17;

fn safe_state() {
    // FIRST: one store, no driver, no lock. SIO GPIO_OUT_CLR drives the pins low
    // if they are outputs, and does nothing if they are not configured yet.
    // The keyer's CI (check_faults.py) requires both bits in this first store.
    const SIO_GPIO_OUT_CLR: *mut u32 = 0xd000_0020 as *mut u32;
    unsafe { SIO_GPIO_OUT_CLR.write_volatile((1 << KEY_GPIO) | (1 << PTT_GPIO)) };
    // Then anything slower. silence_all writes nothing if PWM is still in reset.
    unsafe { pico2::pwm::Rp2350Pwm::silence_all() };
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    safe_state();
    loop { core::hint::spin_loop(); }
}
```

What "safe" means is the application's decision. For the keyer box, "key open" is GPIO low only because the box's key switch turns "pin low" into "line open". **Never-loosen:** a Pico 2 pin is never wired straight to a radio's key line; with the pin driven low as the safe state, a radio that keys on a low line would transmit ([`docs/keyer.md:966-976`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/docs/keyer.md#L966-L976) in cw-message-gateway). Do not copy the pin numbers or the polarity without checking the hardware; see the gateway's [13 Safety rules][gw-13].

### Telling the cores apart in a safe state (PR #6)

```rust
use pico2::multicore::{core, Core};

fn safe_state() {
    outputs_open();                 // first, on whichever core faulted
    if core() == Core::Core1 {
        // e.g. raise a flag in shared memory that core 0 polls
    }
}
```

`core()` is one volatile read of SIO `CPUID` and is safe in a handler ([`multicore.rs:158-166`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L158-L166), PR #6).

### Giving core 1 a stack (PR #6)

```rust
use pico2::multicore::Core1Stack;

// 16 kB: 2048 eight-byte words. At least Core1Stack::MIN_WORDS (128), a multiple of 4.
static CORE1_STACK: Core1Stack<2048> = Core1Stack::new();
```

It goes in `.bss` and counts against the headroom `ASSERT`. Both size rules are compile-time checks in `launch` ([`multicore.rs:357-366`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L357-L366)). The keyer on cw-message-gateway PR #31 uses a 64 kB core 1 stack, `Core1Stack<8192>` ([`firmware/pico2-keyer/src/core1.rs:65`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/core1.rs#L65)).

### Bound the core 0 stack

rustos does not set core 0's `MSPLIM`. An application that wants a stack overflow to fault can do what the keyer on cw-message-gateway PR #31 (draft) does early in `main`, right after it drives the key and PTT low ([`firmware/pico2-keyer/src/main.rs:357-371`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L357-L371)):

```rust
fn limit_stack() {
    unsafe extern "C" { static __ebss: u32; }
    // MSPLIM keeps bits 31:3, so round the end of .bss up to 8 bytes.
    let bottom = ((&raw const __ebss) as usize).next_multiple_of(8);
    unsafe { core::arch::asm!("msr MSPLIM, {}", in(reg) bottom, options(nomem, nostack, preserves_flags)) };
}
```

A push below the limit then raises a UsageFault (`STKOF`), escalated to HardFault, and nothing below the limit is written. **Whether the safe state then runs is not established.** Entering HardFault pushes an exception frame on the same stack; where the frame does not fit, the stack pointer is set to the limit and no push below it is performed ([`DDI0553B_z_armv8m_arm.md:5315-5328`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/DDI0553B_z_armv8m_arm.md#L5315-L5328)). The handler then needs stack of its own: the release `OnHardFault` the tutorial measured began with `push {r7, lr}` (tutorial chapter 05 §5.8, measured before the safe-state call was added). rustos never writes `CCR`, so, assuming the bootrom leaves it at its reset value (not verified), `CCR.STKOFHFNMIGN` is 0, "Stack limit faults not ignored" ([`DDI0553B_z_armv8m_arm.md:71868-71876`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/DDI0553B_z_armv8m_arm.md#L71868-L71876)). A fault that cannot be taken at HardFault priority puts the core in lockup ([`DDI0553B_z_armv8m_arm.md:6713-6717`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/DDI0553B_z_armv8m_arm.md#L6713-L6717)). The safety audit read the keyer image the same way: "a core 0 stack overflow locks up at MSPLIM (OnHardFault pushes first) and the watchdog opens the key within 0.5 s" (project file `audit/report.md` §3, PR #31 entry at `8d5fcf6`). So the bound stops an overflow from corrupting `.bss`, and the keyer counts on its watchdog, not its safe state, to open the key after one. This is analysis from the Arm rules, untested on hardware; [06 Multicore](06-multicore.md) gives the same analysis for core 1.

### Checking a build

Run these from a generated application after any change to `link.ld` or `lib.rs`. They come from the tutorial ([chapter 04](../tutorials/rp2350_baremetal/04_the_linker_script.md) §4.11.2, [chapter 05](../tutorials/rp2350_baremetal/05_boot_and_vectors.md) §5.5 and §5.6.4). `llvm-objdump` and `llvm-nm` come with `rustup component add llvm-tools`, which installs them into the toolchain's sysroot, not on `PATH`. The first line puts them on `PATH`, as tutorial [chapter 01](../tutorials/rp2350_baremetal/01_setup_and_workspace.md) §1.7 does.

```sh
export PATH="$PATH:$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin"   # llvm-tools is not on PATH by default
ELF=target/thumbv8m.main-none-eabihf/release/<app>
llvm-objdump --section-headers $ELF        # .vector_table 0x110 bytes at 0x10000000, then .boot_info, .text, .rodata
llvm-nm -n $ELF | grep -E '_stack_top|__s(i)?data|__edata|__sbss|__ebss'
                                          # _stack_top must be 20082000; read symbols here, not from firmware.map
llvm-objdump -s -j .vector_table $ELF      # word 0 = 00200820 (0x20082000), word 1 = OnReset + 1
cp $ELF app.elf && picotool info -a app.elf  # "image type: ARM Secure", one image def block
```

The tutorial's checklist after a `link.ld` change: `.vector_table` is `0x110` bytes at `0x1000_0000`; `.boot_info` is 4-aligned, non-empty and inside the first 4 kB; a non-empty `.data` has two different addresses and `__sidata` equals its LMA; `.bss` follows `.data` (after any alignment gap) and `.stack` follows `.bss`; `.ARM.exidx` is absent; `_stack_top` is `0x2008_2000` (read with `llvm-nm`; lld's map file prints the location counter, not the symbol's value).

## Changing it

### Which files to touch

| Change | Files | Also |
|---|---|---|
| A real handler for an exception or interrupt | `lib.rs`: the `VECTOR_TABLE` construction ([`lib.rs:509-516`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L509-L516)); the handler as `#[unsafe(no_mangle)] pub extern "C" fn` (edition 2024 requires the `unsafe` form, as at [`lib.rs:465-466`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L465-L466)). Steps: [Adding an interrupt handler](#adding-an-interrupt-handler) | A peripheral interrupt handler must clear its source and return. Only the catch-all and fault handlers spin. Any new fault handler must call `__rustos_safe_state()` first |
| A new linker section (for example, uninitialised RAM) | `link.ld` `SECTIONS`, in address order; a matching `#[unsafe(link_section = "...")]` in Rust | Add an `ASSERT` if a mistake would be silent; re-run the build checks above |
| The minimum stack | `_min_stack_size` ([`link.ld:35`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L35)) | It is a link-time budget only, not a guard |
| A different flash chip size | `LENGTH` of `FLASH` ([`link.ld:23`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/link.ld#L23)) | Use the chip's size, never the 16 MB window |
| The boot metadata (for example, a `VECTOR_TABLE` or `ENTRY_POINT` item, signing) | `BOOT_INFO` ([`lib.rs:152-160`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L152-L160)) | The datasheet's §5.9 is the reference. Keep the block word-aligned, in the first 4 kB, and its link word a closed loop |
| Startup work before `main` | `OnReset` ([`lib.rs:436-445`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L436-L445)) | Nothing before `enable_fpu` may use floating point; nothing before `reset_bss` may read a static; nothing in `OnReset` may panic |

### Adding an interrupt handler

No rustos code on master takes an interrupt, so this has not been done in the repository. The steps follow the table's construction and the datasheet, and are untested.

1. **Find the IRQ number.** The datasheet's §3.2, Table 94, lists all 52 ([`rp2350-datasheet.md:5724-5760`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L5724-L5760)); for example `TIMER0_IRQ_0` is 0 and `ADC_IRQ_FIFO` is 35. The vector slot is 16 + n ([`lib.rs:489`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L489)).
2. **Add the handler to `pico2`.** In `firmware/pico2/src/lib.rs`, write it as `#[unsafe(no_mangle)] pub extern "C" fn <Name>()` and add `t[16 + n] = Vector { handler: <Name> };` to the `const` block that builds `VECTOR_TABLE` ([`lib.rs:508-518`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L508-L518)). An application cannot do this from its own crate, because the table is a fixed `static` inside `pico2`. It needs a rustos change, or a local checkout through the template's `[patch]` section ([`templates/README.md:42-46`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/README.md#L42-L46)).
3. **Enable it twice, on the right core.** First enable the source in the peripheral's own interrupt-enable register (each block's register list names it). Then set bit n mod 32 of `NVIC_ISER0` (IRQs 0-31) or `NVIC_ISER1` (IRQs 32-51), at `PPB_BASE + 0x0E100` and `+ 0x0E104` ([`rp2350-datasheet.md:10712`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L10712), Table 192 at [`rp2350-datasheet.md:12814-12824`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L12814-L12824)); `NVIC_ICER0` and `NVIC_ICER1`, at `+ 0x0E180` and `+ 0x0E184`, disable ([`rp2350-datasheet.md:10716`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L10716)). Each core has its own NVIC, and an interrupt is taken by the core whose NVIC enables it. The datasheet says to enable an interrupt that is not core-local in one core's NVIC only ([`rp2350-datasheet.md:5720-5722`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/extracted/rp2350-datasheet.md#L5720-L5722)). On PR #6, core 1 starts with none enabled ([`multicore.rs:67-68`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L67-L68), PR #6).
4. **Clear the source and return.** A peripheral handler clears its interrupt in the peripheral and returns; it never spins. Share data with `main` only through atomics: rustos has no critical-section helper and no code that masks interrupts (`cpsid` is not found in the code at `80bf599`).
5. **Leave the fault path alone.** Every fault slot keeps a handler that calls `__rustos_safe_state()` first, and `OnHardFault` and `DefaultHandler` keep their names, which cw-message-gateway's `check_faults.py` finds them by ([`check_faults.py:41`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/check_faults.py#L41)). An interrupt enabled in the NVIC whose slot still holds `DefaultHandler` runs the safe state and stops the core.
6. **Reaching a radio.** The keyer box gets the change only when its rustos pin moves, and a pin move needs a fresh safety audit and a new hash before anything is flashed (see [Safety gates and the audit](#safety-gates-and-the-audit)).

Interrupt support is part of item W9 on [09 Work left and known issues](09-work-left-known-issues.md).

### Tests

No automated test covers the boot path. rustos has no CI, and its host tests cover only the `api` crate. Verification is by inspecting the built image (the commands under [Checking a build](#checking-a-build)) and by running it on a Pico 2. cw-message-gateway's CI runs `check_faults.py` on the built keyer image, and that is the only automated check on the fault handlers. It fails if, in `OnHardFault`, `DefaultHandler` or the panic handler, the first store (following any calls) is not the key-open store ([`check_faults.py:1-23`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/check_faults.py#L1-L23)). It finds the two fault handlers by those symbol names ([`check_faults.py:41`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/check_faults.py#L41)) and the panic handler as `rust_begin_unwind` or, failing that, `panic_fmt` ([`check_faults.py:60-70`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/check_faults.py#L60-L70)). Renaming either fault handler breaks that check.

CI runs that check only after a pin has moved. To run it on a rustos commit before proposing a pin move:

1. Push the rustos commit under test to a branch on GitHub; the pin is a `git` URL, so cargo fetches the commit from there.
2. In a scratch branch of a cw-message-gateway clone, set `rev` for both `api` and `pico2` in `firmware/pico2-keyer/Cargo.toml` to that commit ([`firmware/pico2-keyer/Cargo.toml:30-31`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/Cargo.toml#L30-L31)).
3. Let cargo rewrite `firmware/pico2-keyer/Cargo.lock` to match (any cargo build there without `--locked` does), because `build.sh` builds with `--locked` ([`build.sh:34`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/build.sh#L34)). Commit both files: `build.sh` builds the committed tree, not the working tree ([`build.sh:1-14`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/build.sh#L1-L14)).
4. Run `sh firmware/pico2-keyer/build.sh`, then `python3 firmware/pico2-keyer/check_faults.py firmware/pico2-keyer/pico2-keyer.elf`. These are the two steps CI runs, from inside `firmware/pico2-keyer` ([`ci.yml:177-186`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/.github/workflows/ci.yml#L177-L186)).

The UF2 this makes is not an audited file and must never be flashed.

### Safety gates and the audit

- **Never loosen.** The fault handlers call the application's safe state before anything else. The keyer box's rule that the key line is open in fault handlers depends on it. Do not add work before the `__rustos_safe_state()` call in either handler. Do not rename `OnHardFault` or `DefaultHandler`. Do not route a fault slot to a handler that skips the safe state. The full list of limits and gates is the gateway's [13 Safety rules][gw-13].
- **Every rustos change that reaches the radio goes through the audit.** The keyer firmware pins rustos to an exact commit. Moving that pin can change the flashed file even with no source change ([`firmware/pico2-keyer/Cargo.toml:21-29`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/Cargo.toml#L21-L29)). A new pin means a new firmware head, a fresh safety audit and a new hash before anything is flashed. See the gateway's [14 Audit, bench and flashing][gw-14].

### What must never change

- `VECTOR_TABLE` at `ORIGIN(FLASH)`, 512-aligned, word 0 the stack top and word 1 `OnReset`.
- A valid `IMAGE_DEF` in the first 4 kB. Bits 10:8 of the image type select Arm.
- `KEEP()` on `.vector_table` and `.boot_info`, and `#[used]` on both statics.
- `ENTRY(OnReset)`.
- In `OnReset`: the FPU is on before any floating-point instruction; `.data` is copied and `.bss` zeroed before any static is read; the application is called last. `reset_vtor` may move ahead of `enable_fpu`, and `cpsid i` may go first (tutorial chapter 06 §6.9, and [Work left](#work-left)), provided neither emits a floating-point instruction (check with `llvm-objdump -d --disassemble-symbols=OnReset`).
- `reset_data` takes its length from `__edata − __sdata`.
- `FLASH` `LENGTH` is the flash chip's size.
- No code running from flash ever asserts reset on `IO_QSPI` (bit 7) or `PADS_QSPI` (bit 10).
- The fault handlers call the safe state first.

## Known bugs, limits and flakes

| Item | Effect | Source |
|---|---|---|
| Core 0 has no stack limit in rustos | A stack overflow silently corrupts `.bss` instead of faulting. The `.stack` reservation and the `ASSERT` check only the static budget | tutorial chapter 04 §4.7.2 and chapter 06 §6.9; the keyer sets one itself on PR #31 |
| With `MSPLIM` set, an overflow may lock the core up before the safe state runs | Nothing below the stack is written, but the key-open store may never run; only the watchdog or a reset ends it (analysis, untested) | [Bound the core 0 stack](#bound-the-core-0-stack); [06 Multicore](06-multicore.md) |
| Configurable faults are not enabled | `SHCSR` bits 16-19 stay 0, so every fault escalates to HardFault and the specific cause is only in the fault status registers (see [Debugging a stopped handler](#debugging-a-stopped-handler)). `DefaultHandler` in slots 4-7 is never reached | tutorial chapter 06 §6.9 |
| `DefaultHandler` and `OnHardFault` are identical | In a release build they are expected to share one address, so the PC does not say which fired. NMI also lands in `DefaultHandler` | tutorial chapter 05 §5.8 (measured before the safe-state call was added; not re-measured) |
| No `cpsid i` at the top of `OnReset`; `enable_fpu` runs before `reset_vtor` | Harmless on a cold boot, where no interrupt is enabled. A few instructions run with VTOR wherever the bootrom left it | tutorial chapter 06 §6.9 |
| The safe state is not called on a panic | An application whose panic handler does not call it leaves its outputs as they were. The template's panic handler does not call it | [`templates/pico2/src/main.rs:33-36`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/src/main.rs#L33-L36) |
| A fault inside the safe state locks the core up | Only a reset ends it | [`lib.rs:392-394`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L392-L394) |
| Debug builds can panic inside `OnReset` | Precondition and overflow checks in the reset helpers route to the panic handler before `.bss` is zeroed | tutorial chapter 06 §6.8 |
| `link.ld` has no `ASSERT(SIZEOF(.boot_info) == 20)`, no orphan-section guard, no `__sidata % 4` check | A mistyped `link_section` name or a stray section is placed silently | tutorial chapter 04 §4.12 |
| The `/* RP2350-E25 */` comments in `link.ld` overstate that erratum | The `ALIGN(4)`s are good practice, not an E25 workaround (E25 concerns `LOAD_MAP` items, which this image has none of) | tutorial chapter 04 §4.12 |
| Stale references to a `demo` crate | `demo/`, `blinky.elf` and `blinky.uf2` are not in the tree (the demo moved to `templates/pico2`). The README's repository layout and build sections, the crate doc ([`lib.rs:11`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L11), [`lib.rs:40-46`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L40-L46)), [`build.rs:6-10`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/build.rs#L6-L10) and the tutorial still describe it. They say the workspace's `.cargo/config.toml` sets the target and `-Tlink.ld`. At `80bf599` it sets only the `pico2` alias, and `cargo build --release` at the root builds only `api` for the host | [`README.md:64-77`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/README.md#L64-L77), [`README.md:286-301`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/README.md#L286-L301) |
| `enable_fpu`'s "exactly once" safety note is stale on PR #6 | Core 1 calls it too, once per core; `CPACR` is per core. The note still says it runs once, early in `OnReset` | [`lib.rs:252-255`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L252-L255); [`multicore.rs:423-424`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L423-L424) (PR #6); [06 Multicore](06-multicore.md) |
| The crate doc says "this crate must supply the `#[panic_handler]`" | In context it means the application crate; `pico2` defines none | [`lib.rs:34-38`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L34-L38) |

**Hardware status.** The boot path has run on Pico 2 boards: the tutorial ends with the blinky running, and the project's notes record the keyer box flashed with a rustos-based image on the handheld bench. That image, build `eef8c126`, builds rustos `c985211`. Between `c985211` and `80bf599`, `lib.rs` gains only `pub mod dma; pub mod adc;` and their mention in the crate doc, and `link.ld` and `build.rs` are unchanged, so the boot path that ran is this page's boot path (`git diff c985211 80bf599`; see [08 Changes made for cw-message-gateway](08-changes-for-cw-gateway.md)). No record was found of the fault path (a real fault reaching the safe state) being exercised on hardware. The keyer relies on the static check in its CI for that. Everything on PR #6 (core 1's stack, `MSPLIM` on core 1, shared handlers) is untested on hardware.

## Work left

- **Merge rustos PR #6.** It adds core 1's stack and launch, and the shared-handler note. It waits on Robin (the project owner) merging it. After it merges, the keyer firmware moves its rustos pin to master's merge, which needs a fresh audit and hash ([06 Multicore](06-multicore.md)).
- **Set core 0's `MSPLIM` in `OnReset`** (for example to `__ebss` rounded up to 8), so every application gets an overflow bound and not only the keyer. On its own that stops the corruption but may lock the core up instead of running the safe state ([Bound the core 0 stack](#bound-the-core-0-stack)). A fault path that survives an overflow needs more, for example a first store that needs no stack. `MSPLIM`'s bound stays: setting `CCR.STKOFHFNMIGN` would let the handler write below the stack, and [06 Multicore](06-multicore.md) records that it is not an option. Either change touches the keyer's audited fault path and goes through the audit.
- **Make faults identifiable:** enable the configurable faults in `SHCSR`, or give the handlers distinct bodies (for example, read `IPSR` into a distinct value) so the release build does not fold them.
- **Harden `OnReset`:** `cpsid i` first, and swap `enable_fpu` and `reset_vtor` (tutorial chapter 06 §6.9).
- **Add the missing `ASSERT`s** listed in tutorial chapter 04 §4.12.
- **Fix the stale docs:** the `demo` references, `.cargo/config.toml`'s described contents, the stale `enable_fpu` note, and the tutorial's `BOARD_CREATED`, which the code calls `BOARD_TAKEN` ([`api/src/device/mod.rs:253`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L253)).

The full list for rustos is [09 Work left and known issues](09-work-left-known-issues.md).

## Sources

- rustos at `80bf5998a60a41b3741041062ff7e9c59caf95b2`: `firmware/pico2/src/lib.rs` (and its diff from `c985211`), `firmware/pico2/link.ld`, `firmware/pico2/build.rs`, `firmware/pico2/Cargo.toml`, `firmware/pico2/src/common/{mod,reg,reset,board}.rs`, `firmware/pico2/src/{clocks,watchdog,pwm,adc,dma,systick}.rs`, `usb/mod.rs`, `usb/hw.rs` and `gpio/gpio.rs` (the parts cited), `api/src/device/mod.rs` (`define_board!`), `Cargo.toml`, `.cargo/config.toml`, `templates/` (`README.md`, `pico2/.cargo/config.toml`, `pico2/Cargo.toml.liquid`, `pico2/src/main.rs`), `README.md`.
- rustos PR #6 at `410dffa091e247e6e7546c7f2ff465f0af499d5a`: the diff of `firmware/pico2/src/lib.rs`, `firmware/pico2/src/common/board.rs` and `README.md` against master, and `firmware/pico2/src/multicore.rs`.
- rustos commit `02b762b` (rustos #3), which added the safe state to `entry!` and the handlers.
- The bare-metal tutorial: [index](../tutorials/rp2350_baremetal/index.md) and chapters [03](../tutorials/rp2350_baremetal/03_memory_map.md), [04](../tutorials/rp2350_baremetal/04_the_linker_script.md), [05](../tutorials/rp2350_baremetal/05_boot_and_vectors.md) and [06](../tutorials/rp2350_baremetal/06_reset_handler.md).
- The RP2350 datasheet, [`docs/rp2350-datasheet.pdf`](../rp2350-datasheet.pdf), and its extracted text, [`docs/extracted/rp2350-datasheet.md`](../extracted/rp2350-datasheet.md). This page also cites chapter 5's opening (32 kB mask ROM), §3.2 Table 94 (IRQ numbers), the NVIC, `ICSR`, `CFSR` and `HFSR` registers, and §3.7.4 (the PPB is privileged only). The code cites: §5.9.5.1 PDF p427 (minimum Arm `IMAGE_DEF`, vector table fallback); §5.9.3.3 (`VECTOR_TABLE` item, the default at image start); §5.9 and §5.9.1 (blocks in the first 4 kB, word alignment, block loops); PDF p357 (marker values unlikely in code); §3.7.5 (`PPB_BASE`); §3.7 and Table 229 (`CPACR`); Table 201 (`VTOR` `TBLOFF` bits 31:7); §3.2 (52 interrupt inputs); §2.2.2 Table 9 and §2.2.3 Tables 10 and 11 (XIP and SRAM address maps); §12.14.1 (QMI chip-select 0 window); Table 439 (`XIP_CTRL.CTRL`, writes downgraded to reads); §10.2.2 (decoded SRAM range); erratum RP2350-E25; PDF p14 (RISC-V cores drive the same registers).
- The Armv8-M Architecture Reference Manual as extracted text, [`docs/extracted/DDI0553B_z_armv8m_arm.md`](../extracted/DDI0553B_z_armv8m_arm.md): exception entry and the stack frame (B3), stack limit checks (B3.21), priority escalation and lockup (B3.31), `CCR.STKOFHFNMIGN`.
- The tutorial's chapters [01](../tutorials/rp2350_baremetal/01_setup_and_workspace.md) (§1.7, `llvm-tools` on `PATH`) and [08](../tutorials/rp2350_baremetal/08_first_blink.md) (the finished blinky's `.bss`).
- cw-message-gateway at `44185fa9a00369c379d9d8af40ca307e3867e860`: `firmware/pico2-keyer/src/main.rs`, `firmware/pico2-keyer/check_faults.py`, `firmware/pico2-keyer/build.sh`, `firmware/pico2-keyer/Cargo.toml`, `.github/workflows/ci.yml`, `docs/keyer.md` (the pull-down and erratum E9, the key switch). At PR #30 `3efaf842e1aab75354e5e19d0384a703d4d786be`: `firmware/pico2-decoder/src/main.rs` and `firmware/pico2-decoder/Cargo.toml`. At PR #31 `bc5062ae4f56ae1f9949b3b067218c7e675d88f8`: `firmware/pico2-keyer/src/main.rs` (`limit_stack`, the fault flag), `firmware/pico2-keyer/src/core1.rs` and `firmware/pico2-keyer/Cargo.toml`.
- Project file `audit/report.md` (the safety audit), §3, the PR #31 entry at `8d5fcf6`, for its reading of a core 0 stack overflow. Project files are not in either repository.
- Project notes `MEMORY.md` (the project's memory notes, working notes kept outside both repositories), for which keyer image has been flashed and the PR states. Checked against the code where the code says anything.

[gw-08]: https://github.com/robinonsay/cw-message-gateway/blob/main/docs/handover/08-keyer-box-firmware.md
[gw-11]: https://github.com/robinonsay/cw-message-gateway/blob/main/docs/handover/11-decoder-box-one-pico.md
[gw-13]: https://github.com/robinonsay/cw-message-gateway/blob/main/docs/handover/13-safety-rules.md
[gw-14]: https://github.com/robinonsay/cw-message-gateway/blob/main/docs/handover/14-audit-bench-flashing.md
[gw-17]: https://github.com/robinonsay/cw-message-gateway/blob/main/docs/handover/17-glossary.md
