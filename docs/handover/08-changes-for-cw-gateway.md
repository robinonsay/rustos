# 08 Changes made for cw-message-gateway

> Every driver in rustos except GPIO was written for cw-message-gateway, the project whose keyer box and decoder box are Pico 2 boards running rustos firmware. Six rustos pull requests carry that work. #1 to #5 are merged on `master` (`80bf599`); #6 (multicore and `Rp2350Usb::room()`) is open at `410dffa`. This page tells, PR by PR, what each added and why the gateway's firmware needed it; which firmware image uses which driver; what has and has not run on a real Pico 2; how the gateway pins rustos by commit and why moving that pin is a firmware change; and which rustos behaviours the keyer's safety depends on. It describes a set of changes rather than one subsystem, so it adapts the page shape: "How it works" is the story of the six PRs, "How it connects" maps drivers to firmware images and explains the pin, "Interfaces and APIs" lists the public items each PR added, and "Algorithms and processing" walks through the safety-relevant behaviours. Each driver's full description is on [04 Drivers: core](04-drivers-core.md), [05 Drivers: I/O](05-drivers-io.md) and [06 Multicore](06-multicore.md).

Terms used on this page, in plain words (the shared glossary is [17 Glossary][gw-17] in the cw-message-gateway handover; the chip terms are also on [01 Overview](01-overview.md)):

- **Keyer box**: a Pico 2 that turns a radio's transmitter on and off for the gateway's host program `hfnode`, by closing the radio's **key line** through a switch ("key down" = transmitting). It also drives an FM handheld radio's **PTT** (push-to-talk) line and plays **MCW** (Morse sent as an audio tone into the handheld's microphone). **Decoder box**: a separate Pico 2 that listens to a radio's audio and decodes the Morse (cw-message-gateway PR #30, open). **One-Pico image**: one firmware image that runs the keyer on core 0 and the decoder on core 1 of the same Pico 2 (PR #31, draft).
- **Firmware image**, **UF2**: the file flashed onto the board. **Build id**: the first eight characters of the commit CI built from, compiled into the image and reported by the box, so `hfnode` can check it runs the cleared file.
- **Safety audit**: a separate review that clears one exact UF2, by its SHA-256 hash, before it may be flashed or used to key a radio. Its keyer-box findings are numbered KB-1 to KB-14.
- **Pin** (of a dependency): the `rev = "<commit>"` in the firmware's `Cargo.toml` that fixes which rustos commit it builds against.
- **Safe state**: the state of every output that cannot key a radio: key and PTT open, tone off. The keyer reaches it by driving GP16 (key) and GP17 (PTT) low, and low means open only because of the box's wiring: each pin drives an inverting switch (a PC817 optocoupler, or one transistor), and a required external 4.7 kΩ pull-down from the pin to ground holds it low while nothing drives it. A bare Pico pin wired straight to a radio's key contact is graded NO-GO by the safety audit, because every safe state of the box would then key the radio ([`docs/keyer.md:966-976`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/docs/keyer.md#L966-L976), [`docs/keyer.md:981-985`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/docs/keyer.md#L981-L985)). **Never loosen** either part ([13 Safety rules][gw-13]).
- **Watchdog**: a hardware timer that resets the chip unless the program "feeds" it in time. **Scratch registers**: watchdog registers whose contents survive a watchdog reset but not a power-up. **Bootrom**: the chip's built-in first-stage code, which runs before the application after every reset.
- **PWM slice**: one of the RP2350's twelve square-wave generators; `TOP` sets its period and `CC` (compare) the point in each period where the output goes low.
- **DMA ring**: a buffer in RAM that the DMA block fills in a circle without the processor. **Sentinel**: a value the hardware never writes, used to mark a slot as read. **DREQ**: a peripheral's request line, which paces a DMA channel to the peripheral's rate.
- **SIO**: the RP2350's single-cycle I/O block; its `GPIO_OUT_CLR` register drives any set of pins low with one store. **Pad bits**: each pin's pad has `OD` (output disable), `IE` (input enable) and `ISO` (an isolation latch that holds the pin's state); its `IO_BANK0` control register has `FUNCSEL` (which peripheral drives the pin) and `OUTOVER` (an override that can force the output low).
- **PLL**, **VCO**: a phase-locked loop multiplies the crystal's frequency up; the VCO is its internal oscillator, which is then divided down. **NAK**: the USB answer "not now, try again", which makes the host wait without losing data.
- **HardFault**, **lockup**: the fault handler of last resort, and the stopped state a core enters when even that handler faults; only a reset ends a lockup. **Secure Privileged**, **Secure Unprivileged**: Arm security and privilege levels; a register that requires a higher level than the code reading it answers with a bus fault.
- **Erratum**: a documented chip bug, numbered in the RP2350 datasheet's errata list (E5, E9 and E12 appear below). **`hangtest`**, **`stucktest`**, **`linktest`**: the gateway's bench tests that make a box hang, hold its key, or lose its host on purpose ([14 Audit, bench and flashing][gw-14]).
- **`MSPLIM`**, **MPU**: a core's stack-limit register and its memory protection unit; see [06 Multicore](06-multicore.md).

## Where it lives

All six PRs have base `master`. GitHub merged #1 to #5 with merge commits, and for each of them the merge commit's tree is identical to the PR head's tree (checked with `git rev-parse <commit>^{tree}`), so the head and the merge hold the same sources. #6 is open at `410dffa`.

| PR | Title | Branch | State and commits | Files added (A) or changed (C) | Diff |
|---|---|---|---|---|---|
| #1 | Pico 2: clocks, timer, watchdog and USB serial drivers | `claude/any-radio-key-jack-wijoc6` | merged 2026-10-06 as `cfc2b14`; head `5bb9199` | A: `api/src/usb/{mod,setup,descriptor,cdc_acm,control,ring}.rs`, `firmware/pico2/src/{clocks,timer,watchdog}.rs`, `firmware/pico2/src/usb/{mod,hw}.rs`. C: `common/{board,reg,reset}.rs`, both crates' `lib.rs`, `README.md` | 17 files, +4,881 −15 |
| #2 | pico2: add PWM square-wave outputs | `claude/any-radio-key-jack-wijoc6` | merged 2026-10-08 as `fcde02e`; head `c1c82c9` | A: `firmware/pico2/src/pwm.rs`. C: `common/{board,reg,reset}.rs`, `gpio/mod.rs`, `lib.rs`, `README.md` | 7 files, +959 −11 |
| #3 | pico2: a timer no debugger pauses, SysTick, app scratch registers, a fault safe state | `claude/keyer-unattended-d05o6i` | merged 2026-10-08 as `5fc0c2d`; head `c985211` | A: `firmware/pico2/src/systick.rs`. C: `timer.rs`, `watchdog.rs`, `lib.rs`, `common/board.rs`, `README.md` | 6 files, +256 −25 |
| #4 | templates: name the pico2 template's manifest Cargo.toml.liquid | `claude/project-thread-ask7sb` | merged 2026-10-09 as `8c91507`; head `6b02215` | renamed `templates/pico2/Cargo.toml` to `Cargo.toml.liquid`; C: `templates/README.md` | 2 files, +7 −1 |
| #5 | pico2 adc, dma: sample an analog pin endlessly into a DMA ring | `claude/pico-decoder-u9bum5` | merged 2026-10-11 as `80bf599` (master); head `e3d8369` | A: `firmware/pico2/src/{adc,dma}.rs`. C: `clocks.rs`, `common/{board,reg}.rs`, `lib.rs`, `README.md` | 7 files, +1,133 −15 |
| #6 | pico2 multicore: start core 1 on its own stack (and usb room) | `claude/pico-decoder-u9bum5` | **open**; commits `3d8af9c`, `422320d`, `410dffa` (head) | A: `api/src/multicore.rs`, `firmware/pico2/src/multicore.rs`. C: `usb/mod.rs`, `common/board.rs`, both crates' `lib.rs`, `README.md` | 7 files, +1,007 −6 |

The modules the six PRs added (`clocks`, `timer`, `systick`, `watchdog`, `usb`, `pwm`, `dma`, `adc`) are declared at [`firmware/pico2/src/lib.rs:110-117`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L110-L117) (master) and [`firmware/pico2/src/lib.rs:118`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/lib.rs#L118) (`multicore`, PR #6 only). The board's device handles they added sit in the `devices` block at [`firmware/pico2/src/common/board.rs:136-143`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/board.rs#L136-L143); PR #6 adds `core1` at [`firmware/pico2/src/common/board.rs:147`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/common/board.rs#L147). GPIO (`firmware/pico2/src/gpio/`) was written on 2026-08-29 and 2026-08-30, before this project. Its only change since is PR #2's commit `a123f90` (2026-10-07), which made the GPIO register types `IoBank`, `PadsBank` and `GpioRegs` `pub(crate)` for the PWM driver. It is described here only where the gateway relies on it.

## How it works

The gateway's firmware grew in steps, and each step needed something rustos did not yet have. Each rustos PR was opened from the gateway thread that needed it, and the gateway's firmware was pinned to that PR's head and then to master's merge of it. The figure shows the order.

![Timeline of rustos PRs #1 to #6, each beside the cw-message-gateway change that first used it: #1 for PR #11, #2 for PR #15, #3 for PR #16, #4 for PR #24, #5 for PR #30 and #6 for PR #31](diagrams/08-pr-timeline.svg)

*Figure: time runs down. Each rustos PR is on the left and the gateway change that first pinned it is on the right; dashed purple marks PRs that are still open.*

### #1: clocks, timer, watchdog and USB serial (for the keyer box, gateway PR #11)

The first keyer box (gateway PR #11, "Any radio: key jack keyer box (Pico 2) and headphone audio", merged 2026-10-07) is a Pico 2 that takes text lines from `hfnode` over a USB serial port and keys a radio's key line on GP16. rustos then had only GPIO, so PR #1 added what such a box needs:

- **Clocks** (`clocks.rs`): the 12 MHz crystal, `PLL_SYS` at 150 MHz for the processors and `PLL_USB` at 48 MHz for USB, with every frequency a compile-time constant checked by `const` assertions ([`firmware/pico2/src/clocks.rs:91-118`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L91-L118), [`firmware/pico2/src/clocks.rs:177-187`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L177-L187)). `Rp2350Clocks` is a zero-sized proof that this ran; every driver that needs a frequency takes `&Rp2350Clocks` ([`firmware/pico2/src/clocks.rs:363-372`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L363-L372)).
- **Timer** (`timer.rs`): `TIMER0` counting 1 µs ticks from the crystal, read without a race between its two halves ([`firmware/pico2/src/timer.rs:207-224`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/timer.rs#L207-L224)). The keyer's software time limits (key-down, run, PTT, link, rest and the duty budget) are all measured on it ([`crates/keyer-core/src/lib.rs:44-101`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/keyer-core/src/lib.rs#L44-L101)); the 500 ms watchdog counts in the watchdog block itself, and the clock check compares the timer with SysTick ([`crates/keyer-core/src/control.rs:86-137`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/keyer-core/src/control.rs#L86-L137)).
- **Watchdog** (`watchdog.rs`): start, feed and force a reset, with a reset scope that resets everything but the oscillators ([`firmware/pico2/src/watchdog.rs:26-63`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L26-L63)), its debugger pause bits cleared, and a decoded reason for the last reset ([`firmware/pico2/src/watchdog.rs:65-127`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L65-L127)). The keyer runs it at 500 ms.
- **USB serial** (`usb/`, and `api::usb`): a polled USB CDC-ACM device (the serial-port class that computers open without a driver install). The chip-independent half (SETUP parsing, descriptors, the endpoint-0 state machine, CDC line coding, a byte ring) is in the `api` crate with 35 host unit tests; the register half is `pico2::usb` ([`api/src/usb/mod.rs:98-102`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/mod.rs#L98-L102)). It reports link loss as an epoch counter, which the keyer uses to end a run the moment USB goes away ([`firmware/pico2/src/usb/mod.rs:164-194`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L164-L194)). That path has not run on a chip: bench `linktest` has not run, an unplug is reported as `Suspended` because the board cannot sense VBUS, and whether `SIE_STATUS.CONNECTED` ever falls is unverified ([`firmware/pico2/src/usb/mod.rs:177-190`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L177-L190)).
- **Support**: `common::reset::cycle_reset`, which puts a block through a full reset so a driver starts from reset values even after a debugger restarted only the processors ([`firmware/pico2/src/common/reset.rs:230-251`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/reset.rs#L230-L251)); and the register addresses in `RegAddr` ([`firmware/pico2/src/common/reg.rs:57-155`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/reg.rs#L57-L155)).

### #2: PWM square waves (for the handheld's MCW tone, gateway PR #15)

Gateway PR #15 ("Keyer box: drive the UV-K1 handheld through its K-plug (PTT, MCW tone, PTT line)") keys an FM handheld by holding its PTT on GP17 and playing a 700 Hz tone on GP18 into its microphone. PR #2 added the PWM driver for that tone: a fixed-frequency 50 % square wave per pin, switched on and off with `Write<bool>` ([`firmware/pico2/src/pwm.rs:774-804`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L774-L804)). Its three commits show the safety thinking: `a123f90` the driver; `0073788` turned tones off on `Drop` and added `silence_all` for panic and fault handlers, the `NoWrap` error, and disconnecting pins a previous run left on the PWM; `c1c82c9` made `silence_all` write nothing while the PWM block is held in reset ([`firmware/pico2/src/pwm.rs:709-713`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L709-L713)). It also added `common::reset::is_reset_done` ([`firmware/pico2/src/common/reset.rs:211-228`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/reset.rs#L211-L228)) and made the GPIO register structs `pub(crate)` so the PWM driver can route a pin the same way the GPIO driver does.

### #3: a timer no debugger pauses, SysTick, scratch registers, a fault safe state (for the audit's KB-5, KB-7 and KB-9, gateway PR #16)

The safety audit's keyer-box review (project file `audit/req-keyer-box-fixes.md`) found three gaps that needed rustos changes. Gateway PR #16 fixed them on the firmware side, and rustos PR #3 (commit `02b762b`, whose message names "the hardware safety audit's KB-5, KB-7 and KB-9"; the gateway repository was then called `ic-7300-hf-server`) supplied the parts:

| Audit item | The gap | What rustos #3 added | What the gateway firmware does with it |
|---|---|---|---|
| KB-5, "One clock for every limit" | `TIMER0` paused while a debugger halted either core (reset value `0b110`), so halting core 1 froze every keyer limit while core 0 ran on and fed the watchdog. And every time source counted the same tick, so nothing could notice it stopping. | `Rp2350Timer::new` writes `DBGPAUSE` = 0 ([`firmware/pico2/src/timer.rs:194`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/timer.rs#L194)); a new `Rp2350SysTick`, the core's 24-bit counter run free on the 150 MHz processor clock, a path apart from the tick ([`firmware/pico2/src/systick.rs:1-46`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/systick.rs#L1-L46)). | `ClockCheck` compares the two every 50 ms and trips the box (`CLOCK`) if they disagree ([`crates/keyer-core/src/control.rs:86-137`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/keyer-core/src/control.rs#L86-L137)). |
| KB-7, "Restarts and trips are not latched" | A trip lived in RAM, so a watchdog reset cleared it. | `Scratch`, `scratch()` and `set_scratch()` for `SCRATCH0` and `SCRATCH1`, the two scratch registers the bootrom does not use ([`firmware/pico2/src/watchdog.rs:129-149`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L129-L149)). | Every pass saves the trip, whether the transmitter was keyed, and the duty budget there ([`crates/keyer-core/src/keyer.rs:222-239`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/keyer-core/src/keyer.rs#L222-L239)); a watchdog boot comes up tripped ([`crates/keyer-core/src/keyer.rs:406-409`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/keyer-core/src/keyer.rs#L406-L409)). |
| KB-9, "Fault handlers do not open the key" | `OnHardFault` and `DefaultHandler` were `loop {}`, leaving the key as it was until the watchdog reset the chip. | `entry!(main, safe_state = f)`: both handlers call `f` first ([`firmware/pico2/src/lib.rs:380-394`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L380-L394)); `entry!(main)` alone names `no_safe_state`, which does nothing. | `safe_state` opens key and PTT with one store, then `silence_all`, then holds the tone pin low; `check_faults.py` checks the order in the built image ([`firmware/pico2-keyer/src/main.rs:236-252`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L236-L252)). |

The commit message of `02b762b` says "Nothing here has run on a Pico 2". PR #3's branch merged PR #2's branch in twice (`aab6a5d`, `c985211`), so its head `c985211` holds #1, #2 and #3, which is why the keyer image could pin `c985211` before either PR was merged.

### #4: the template's manifest renamed (for every firmware build, gateway PR #24)

cargo reads every `Cargo.toml` in a git dependency's repository. rustos's application template had one with `name = "{{project-name}}"`, so every firmware build printed `invalid character '{' in package name` at error level and then built fine. PR #4 renamed it `Cargo.toml.liquid`, which cargo-generate 0.23.4 or later turns back into `Cargo.toml` when it generates an application ([`templates/README.md:31-35`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/README.md#L31-L35), [`templates/README.md:8-9`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/README.md#L8-L9)). Nothing in `api` or `pico2` changed: the audit recorded that "rustos c9852112..8c91507 changes only `templates/`" (project file `audit/report.md` §3). Gateway PR #24 moved the keyer's pin to `8c91507`; that move is where the lesson in [Moving the pin can change the UF2](#moving-the-pin-can-change-the-uf2) was learned.

### #5: ADC and DMA (for the decoder box, gateway PR #30)

The decoder box listens to a radio's audio on GP26 at 32,000 samples per second ([`crates/decoder-core/src/lib.rs:39-40`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/crates/decoder-core/src/lib.rs#L39-L40)). The ADC's own FIFO holds eight results, 250 µs at that rate, far too little to poll beside USB. PR #5 added a DMA driver that resets the DMA block and hands out its sixteen channels as owned tokens, and an ADC driver that runs one channel endlessly from the ADC FIFO into a 16 kB ring of 8192 samples (256 ms at 32 kHz), with overrun detection by sentinel ([`firmware/pico2/src/adc.rs:45-88`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L45-L88)). It added `CLK_ADC_HZ` and `start_clk_adc` to the clocks module, leaving `Rp2350Clocks::new` unchanged so that an image without the ADC runs the same clock sequence ([`firmware/pico2/src/clocks.rs:109-112`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L109-L112), [`firmware/pico2/src/clocks.rs:427-443`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L427-L443)). The audit classified it with no separate hardware review: "It is reviewed with whichever firmware image pins it" (project file `audit/report.md` §3). The commit message of `5123db5` says the ring logic was checked in a model (400 seeds × 20,000 reads) and is "Untested on hardware".

### #6: multicore and `Rp2350Usb::room()` (for the one-Pico image, gateway PR #31; open)

Robin (the project owner) chose on 2026-10-11 to run the decoder on core 1 of the keyer box's own Pico 2. PR #6 adds the code that starts core 1: a portable, never-blocking launch handshake in `api::multicore` with nine host tests, and `pico2::multicore`, which starts core 1 on a static stack with a time limit, sets core 1's stack limit before its first push, and turns on its floating-point unit ([`firmware/pico2/src/multicore.rs:44-68`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L44-L68)). Its second commit aligns core 1's stack to 32 bytes so one MPU region can cover it exactly; its third adds `Rp2350Usb::room()`, which the keyer needs to forward a decoder line only when a whole keyer reply still fits behind it ([`firmware/pico2/src/usb/mod.rs:482-495`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/usb/mod.rs#L482-L495)). The commit message of `3d8af9c` says "Untested on hardware". Full description: [06 Multicore](06-multicore.md).

### What has run on a Pico 2

The only hardware run on record is the audited keyer image, build id `eef8c126` (gateway `eef8c12`, which pins rustos `c985211`, PR #3's head: its `api` and `pico2` sources are those of master's merges of #3 and #4). Before it, at 16:08Z on 2026-10-09, an unchecked UF2 was copied to the bare Pico 2, possibly the unaudited build `7f91ed67`; at 16:10Z it was replaced with the audited file (SHA-256 `f548596b…`) before anything was wired (project file `bench/quansheng.md`, log of 2026-10-09). On 2026-10-09 Robin flashed that UF2 onto a bare Pico 2 with no radio attached. `hfnode devices` listed it as `USB 2E8A:0009 hfnode, PICO2-KEYER, 1`, and `hfnode keyer check` reported "all ok; nothing was keyed": protocol 4, the limits as audited, build `eef8c126`, up 151 s, last started by POWER, key up, PTT up, idle, PTT line high (project file `bench/quansheng.md`, stage 2 and its log). The cw-message-gateway side of that run is on [14 Audit, bench and flashing][gw-14].

| rustos code | Status on hardware | Evidence or reason |
|---|---|---|
| #1 clocks, timer, USB enumeration and serial | **Ran** | The box enumerated with its configured strings and answered `keyer check`. `Rp2350Usb::new` waits on the timer ([`firmware/pico2/src/usb/mod.rs:402-403`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L402-L403)), which waits on the clocks. |
| #1 watchdog start and feed | **Ran** | Started at [`firmware/pico2-keyer/src/main.rs:216`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L216); the box stayed up 151 s and still reported POWER as its last start, so it was fed and did not reset. |
| #1 watchdog timeout reset, reset reason after a timeout | Not yet | Exercised by `hangtest` in bench stage 7, which has not run. |
| #1 USB link loss (`link_epoch` on unplug, suspend, bus reset) | Not yet | Bench `linktest` (stage 7) has not run. An unplug is reported as `Suspended`, and whether `CONNECTED` ever falls with VBUS forced is unverified ([`firmware/pico2/src/usb/mod.rs:177-190`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L177-L190)). |
| #2 PWM bring-up (`Rp2350Pwm::new`, `square_from_handle`) | **Ran** (inferred) | The keyer unwraps `square_from_handle` at [`firmware/pico2-keyer/src/main.rs:191`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L191), before USB starts at line 204; a `NoWrap` error would have panicked before the box could enumerate. |
| #2 a tone actually playing, `silence_all` | Not yet | No MCW keying has run; no fault has been triggered on a chip. |
| #3 `DBGPAUSE` = 0, SysTick | **Ran** (the writes; inferred) | Both constructors run before USB ([`firmware/pico2-keyer/src/main.rs:194-195`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L194-L195)). The record lists no trip. Nothing tested a debugger halt or a stopped tick. |
| #3 scratch registers | Write path ran | `set_scratch` runs every pass. Reading them back after a watchdog reset waits for `hangtest` (stage 7). |
| #3 fault safe state | Not on a chip | Checked in the built ELF by `check_faults.py` in CI ([`firmware/pico2-keyer/check_faults.py:1-29`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/check_faults.py#L1-L29)); no fault has been raised on a chip. |
| #4 template | Not code | |
| #5 ADC and DMA | **No** | The decoder box image (gateway PR #30 at `3efaf842`, SHA-256 `9970b5f5…`) is cleared to flash on a plain Pico 2, but no record of it being flashed was found in the project files read for this page. |
| #6 multicore, `room()` | **No** | "Untested on hardware" (commit `3d8af9c`). The one-Pico image (PR #31 at `bc5062ae`, SHA-256 `9ea00c58…51a7`) is cleared to flash only after PR #31 merges. |

## How it connects

rustos has no wire format; what crosses the boundary is the crate API, the linker script and the template ([01 Overview](01-overview.md) covers those). The gateway's three firmware images are its only users.

### Which image uses which driver

![A matrix of the ten rustos drivers against the three firmware images: the keyer box on main uses clocks, timer, SysTick, watchdog with scratch, USB, GPIO and PWM; the decoder box on PR #30 uses clocks, timer, watchdog, USB, GPIO, ADC and DMA; the one-Pico image on PR #31 uses all ten, plus Rp2350Usb::room](diagrams/08-driver-usage.svg)

*Figure: a filled dot means the image constructs that driver. The two right-hand columns, and the multicore row, are on open PRs.*

Where each image constructs each driver (an empty cell: not used):

| Driver (added by) | Keyer box, gateway main `44185fa`, rustos `8c91507` | Decoder box, gateway PR #30 at `3efaf84` (open), rustos `80bf599` | One-Pico image, gateway PR #31 at `bc5062a` (draft), rustos `410dffa` |
|---|---|---|---|
| `gpio` (before this project) | [`firmware/pico2-keyer/src/main.rs:176-186`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L176-L186) | [`firmware/pico2-decoder/src/main.rs:144-150`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/src/main.rs#L144-L150) | [`firmware/pico2-keyer/src/main.rs:240-251`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L240-L251) |
| `clocks` (#1; `CLK_ADC_HZ` #5) | [`firmware/pico2-keyer/src/main.rs:188`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L188) | [`firmware/pico2-decoder/src/main.rs:154`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/src/main.rs#L154) | [`firmware/pico2-keyer/src/main.rs:253`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L253) |
| `timer` (#1; `DBGPAUSE` #3) | [`firmware/pico2-keyer/src/main.rs:194`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L194) | [`firmware/pico2-decoder/src/main.rs:155`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/src/main.rs#L155) | [`firmware/pico2-keyer/src/main.rs:259-263`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L259-L263), shared with core 1 as `&'static` |
| `systick` (#3) | [`firmware/pico2-keyer/src/main.rs:195`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L195) | | [`firmware/pico2-keyer/src/main.rs:264`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L264) |
| `watchdog` (#1; scratch #3) | [`firmware/pico2-keyer/src/main.rs:196-216`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L196-L216), scratch read at 198-201 and written at [`firmware/pico2-keyer/src/main.rs:127-130`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L127-L130) | [`firmware/pico2-decoder/src/main.rs:156-169`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/src/main.rs#L156-L169), no scratch | [`firmware/pico2-keyer/src/main.rs:265-285`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L265-L285), scratch as on main |
| `usb` (#1; `room()` #6) | [`firmware/pico2-keyer/src/main.rs:204`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L204) | [`firmware/pico2-decoder/src/main.rs:182`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/src/main.rs#L182) | [`firmware/pico2-keyer/src/main.rs:273`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L273); `room()` at [`firmware/pico2-keyer/src/main.rs:197-199`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L197-L199) |
| `pwm` (#2) | [`firmware/pico2-keyer/src/main.rs:190-192`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L190-L192); `silence_all` at [`firmware/pico2-keyer/src/main.rs:250`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L250) | | [`firmware/pico2-keyer/src/main.rs:255-257`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L255-L257); `silence_all` at [`firmware/pico2-keyer/src/main.rs:352`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L352) |
| `dma` (#5) | | [`firmware/pico2-decoder/src/main.rs:170`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/src/main.rs#L170) | [`firmware/pico2-keyer/src/main.rs:291`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L291) |
| `adc` (#5) | | [`firmware/pico2-decoder/src/main.rs:171-179`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/src/main.rs#L171-L179) | [`firmware/pico2-keyer/src/main.rs:292-294`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L292-L294), through [`firmware/pico2-keyer/src/core1.rs:88-102`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/core1.rs#L88-L102) |
| `multicore` (#6) | | | [`firmware/pico2-keyer/src/main.rs:306-308`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L306-L308); core 1's stack at [`firmware/pico2-keyer/src/core1.rs:64-65`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/core1.rs#L64-L65) |

All three name a safe state in `entry!` ([`firmware/pico2-keyer/src/main.rs:60`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L60), [`firmware/pico2-decoder/src/main.rs:45`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/src/main.rs#L45), [`firmware/pico2-keyer/src/main.rs:75`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L75)). How each image uses its drivers is on the cw-message-gateway pages [08 Keyer box: firmware][gw-08] and [11 Decoder box and one-Pico image][gw-11].

After start-up, every rustos call the keyer makes goes through one trait, `keyer_core::Hardware`, which the firmware's `Board` implements: `now_ms` is `timer.now() / 1000`, `cpu_cycles` is `systick.cycles()`, `save` is two `set_scratch` calls, `usb_poll`, `link_epoch`, `usb_read` and `usb_write` are the `Rp2350Usb` methods `poll`, `link_epoch`, `read` and `write`, `set_tone` is `Rp2350PwmSquare::write`, and `feed_watchdog` is `feed` ([`firmware/pico2-keyer/src/main.rs:116-172`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L116-L172)). The method-by-method table is on [08 Keyer box: firmware][gw-08]; the one-Pico image adds `usb_room`, which is `room()` (PR #31, draft; see [11 Decoder box and one-Pico image][gw-11]).

### How the firmware pins rustos

Each firmware crate depends on `api` and `pico2` as git dependencies pinned by `rev`. The keyer's pin was set once and moved five times as rustos grew; the decoder crate (PR #30) started at `80bf599`:

| Gateway commit | Date | Gateway PR | Manifest | rustos pin | What that pin is |
|---|---|---|---|---|---|
| `7b5723c` | 2026-10-05 | #11 (branch) | [`firmware/pico2-keyer/Cargo.toml:23-24`](https://github.com/robinonsay/cw-message-gateway/blob/7b5723c7f63e0764406a8fb99f7fabafddf57471/firmware/pico2-keyer/Cargo.toml#L23-L24) | `5bb9199` | head of rustos #1, before it merged |
| `cf81687` | 2026-10-06 | #11 (head; merged 2026-10-07 as `2e39996`) | [`firmware/pico2-keyer/Cargo.toml:24-25`](https://github.com/robinonsay/cw-message-gateway/blob/cf81687a684aa831c9af53a12d8e9b97c7075cd2/firmware/pico2-keyer/Cargo.toml#L24-L25) | `cfc2b14` | master's merge of #1 |
| `5a77cee` | 2026-10-08 | #15 (first PWM use) | [`firmware/pico2-keyer/Cargo.toml:24-25`](https://github.com/robinonsay/cw-message-gateway/blob/5a77ceedbeba81feca36a7815a2955e0010fba74/firmware/pico2-keyer/Cargo.toml#L24-L25) | `c1c82c9` | head of rustos #2 |
| `7188469` | 2026-10-08 | #16 | [`firmware/pico2-keyer/Cargo.toml:27-28`](https://github.com/robinonsay/cw-message-gateway/blob/7188469468d1d94ea368ac9598a9c82199c3b458/firmware/pico2-keyer/Cargo.toml#L27-L28) | `c985211` | head of rustos #3 (holds #1-#3) |
| `9dcd7b1` | 2026-10-09 | #24 (merged 2026-10-09 as `76f35d1`) | [`firmware/pico2-keyer/Cargo.toml:30-31`](https://github.com/robinonsay/cw-message-gateway/blob/9dcd7b18ad6e6ef4ba28026e1ea3ae05a77974f7/firmware/pico2-keyer/Cargo.toml#L30-L31) | `8c91507` | master's merge of #4; on main today ([`firmware/pico2-keyer/Cargo.toml:21-31`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/Cargo.toml#L21-L31)) |
| `76c60ef` | 2026-10-11 | #30 (decoder crate, new) | [`firmware/pico2-decoder/Cargo.toml:24-25`](https://github.com/robinonsay/cw-message-gateway/blob/76c60eff93c0b434371b8337400fc4b493b4ab3a/firmware/pico2-decoder/Cargo.toml#L24-L25) | `80bf599` | master's merge of #5; unchanged at PR #30's head ([`firmware/pico2-decoder/Cargo.toml:21-25`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/Cargo.toml#L21-L25)) |
| `b570317` | 2026-10-11 | #31 (keyer crate) | [`firmware/pico2-keyer/Cargo.toml:30-31`](https://github.com/robinonsay/cw-message-gateway/blob/b570317eedba55f6632a07b930b9f35e2797e37d/firmware/pico2-keyer/Cargo.toml#L30-L31) | `410dffa` | head of rustos #6, open; "To be moved to master's merge of rustos #6 once it is merged" ([`firmware/pico2-keyer/Cargo.toml:24-31`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/Cargo.toml#L24-L31)) |

Gateway PRs #12 to #16 were merged within one minute on 2026-10-08 (12:38:56Z to 12:39:18Z, by GitHub's merge times). #13's commits reached `main` through `62cde5f`, PR #12's merge (#12's branch had merged #13's head `bf03b4b`); #15's and #16's reached it through `eef8c12`, PR #14's merge. `eef8c12` is the commit the audited keyer image `eef8c126` was built from, so the image on the bench pins `c985211`.

A rustos merge changes nothing in the gateway until someone moves a pin. An application generated from the template is different: it follows rustos's default branch unpinned ([`templates/pico2/Cargo.toml.liquid:10-12`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/Cargo.toml.liquid#L10-L12)).

### Moving the pin can change the UF2

The keyer's manifest says it in its own words: "Moving this pin can change the UF2 with no source change, since cargo mixes the commit into the crate hashes that order the code. This one does not: built with the same build id, the UF2 is byte for byte the audited one" ([`firmware/pico2-keyer/Cargo.toml:27-29`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/Cargo.toml#L27-L29)). What was found when the pin moved to rustos #4:

- **The commit alone can move code.** cargo hashes a git dependency's commit into the crate's metadata hash, which goes into every symbol name, and the linker lays code out in an order that follows those names. Pinning `6b02215`, PR #4's head, whose tree is identical to `8c91507`, moved 46,030 bytes of the UF2 (`rust_begin_unwind` moved before `main`); pinning `8c91507` moved none (project memory note `rustos-template-cargo-error.md`). The same mechanism is why `build.sh` builds at one fixed path: a dependency outside the firmware's workspace has its absolute path hashed the same way ([`firmware/pico2-keyer/build.sh:7-14`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/build.sh#L7-L14)).
- **The build id moves bytes too.** `build.sh` compiles the commit's first eight characters into the image ([`firmware/pico2-keyer/build.sh:22`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/build.sh#L22), [`firmware/pico2-keyer/build.sh:34`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/build.sh#L34)). The audit built main `76f35d1` (pin `8c91507`) both ways: with build id `eef8c126` it gave the audited UF2, SHA-256 `f548596b…`, byte for byte; with main's own id `76f35d1e` it gave `36488b57…`, which differs in 28 bytes of layout and is not cleared for flashing (project file `audit/report.md` §3, PR #24 entry). The box therefore still runs `eef8c126`.
- **The lesson.** Never assume either way. Rebuild at the audited build id and compare byte for byte. Any pin move is a new head of the firmware, and the UF2 from that head needs a fresh audit and a new hash before it is flashed. The audit says the same of PR #31: "The rustos #6 re-pin needs a new hash" (project file `audit/report.md` §3).

## Interfaces and APIs

Everything public that the six PRs added, by PR. Master is `80bf599`; PR #6 items are at `410dffa` and are **not on master**. Line numbers are where the item is defined. Constructors take the board's `DeviceHandle<T>` by value, so each runs at most once per boot (the ownership model is on [01 Overview](01-overview.md)).

### #1: `pico2::clocks`

| Item | Signature or value | Notes |
|---|---|---|
| `XOSC_HZ` | `u32` = 12,000,000 | [`firmware/pico2/src/clocks.rs:91`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L91) |
| `CLK_REF_HZ` | `u32` = `XOSC_HZ` | [`firmware/pico2/src/clocks.rs:95`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L95); the tick source |
| `CLK_SYS_HZ` | `u32` = 150,000,000 | [`firmware/pico2/src/clocks.rs:99`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L99); asserted at line 179 |
| `CLK_PERI_HZ` | `u32` = `CLK_SYS_HZ` | [`firmware/pico2/src/clocks.rs:103`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L103) |
| `CLK_USB_HZ` | `u32` = 48,000,000 | [`firmware/pico2/src/clocks.rs:107`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L107); asserted at line 180 |
| `PLL_SYS_VCO_HZ`, `PLL_USB_VCO_HZ` | 1,500 MHz, 1,200 MHz | [`firmware/pico2/src/clocks.rs:115-118`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L115-L118) |
| `PllConfig { refdiv, fbdiv, postdiv1, postdiv2 }` | `u32` fields; `const fn vco_hz`, `output_hz`, `is_valid` | [`firmware/pico2/src/clocks.rs:126-169`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L126-L169) |
| `PLL_SYS_CONFIG`, `PLL_USB_CONFIG` | 1 × 125 / 5 / 2; 1 × 100 / 5 / 5 | [`firmware/pico2/src/clocks.rs:172-175`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L172-L175) |
| `Rp2350Clocks` | zero-sized proof | [`firmware/pico2/src/clocks.rs:370-372`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L370-L372) |
| `Rp2350Clocks::new(_handle: DeviceHandle<Rp2350Clocks>) -> Self` | busy-waits for the crystal (about 6 ms), then for the two PLL locks; waits forever if the crystal never starts | [`firmware/pico2/src/clocks.rs:374-386`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L374-L386) |

The RP2350 erratum E12 rule, `clk_sys` at least 10 % faster than `clk_usb` while USB is in use, is a `const` assertion ([`firmware/pico2/src/clocks.rs:185`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L185)).

### #1 (and #3): `pico2::timer`

| Item | Signature or value | Notes |
|---|---|---|
| `TICK_CYCLES` | `u32` = `CLK_REF_HZ / 1_000_000` = 12 | [`firmware/pico2/src/timer.rs:68`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/timer.rs#L68); one tick per µs |
| `Rp2350Timer` | zero-sized | [`firmware/pico2/src/timer.rs:172-174`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/timer.rs#L172-L174) |
| `Rp2350Timer::new(_handle, _clocks: &Rp2350Clocks) -> Self` | starts the `TIMER0` tick, cycles `TIMER0` through reset (count starts at 0), writes `DBGPAUSE` = 0 (#3) | [`firmware/pico2/src/timer.rs:176-197`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/timer.rs#L176-L197) |
| `now(&self) -> u64` | µs since `new`; reads `TIMERAWH`, `TIMERAWL`, `TIMERAWH` and retries if the high word changed | [`firmware/pico2/src/timer.rs:199-224`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/timer.rs#L199-L224); safe from either core |
| `elapsed_since(&self, start: u64) -> u64` | `now().wrapping_sub(start)` | [`firmware/pico2/src/timer.rs:226-230`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/timer.rs#L226-L230) |
| `delay_us(&self, us: u64)`, `delay_ms(&self, ms: u32)` | busy-wait at least that long; does not feed the watchdog | [`firmware/pico2/src/timer.rs:232-252`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/timer.rs#L232-L252) |

### #1 (and #3): `pico2::watchdog`

| Item | Signature or value | Notes |
|---|---|---|
| `MAX_TIMEOUT_US` | `u32` = `0x00ff_ffff` (about 16.7 s) | [`firmware/pico2/src/watchdog.rs:213`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L213) |
| `SCRATCH_MARKER` | `u32` = `0x7275_7374`, kept in `SCRATCH4` while the driver's watchdog runs | [`firmware/pico2/src/watchdog.rs:217`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L217) |
| `Scratch { Scratch0 = 0, Scratch1 = 1 }` (#3) | the only two scratch registers the application may use | [`firmware/pico2/src/watchdog.rs:354-362`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L354-L362) |
| `ResetReason` | `ChipReset(ChipResetCause)`, `WatchdogTimeout`, `WatchdogForced(ForcedBy)` | [`firmware/pico2/src/watchdog.rs:364-376`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L364-L376) |
| `ForcedBy` | `Trigger` (`force_reset` or other code wrote `TRIGGER`), `Reboot` (bootrom, picotool, SDK reboot) | [`firmware/pico2/src/watchdog.rs:378-389`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L378-L389) |
| `ChipResetCause` | `PowerOn`, `Brownout`, `RunPin`, `DebuggerReset`, `DebuggerRescue`, `RiscvDebuggerReset`, `GlitchDetector`, `SwitchedCorePowerDown`, `WatchdogChipLevel`, `Unknown` | [`firmware/pico2/src/watchdog.rs:391-416`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L391-L416) |
| `ResetSnapshot { reason, chip_reset, scratch4 }` | the raw `u32` registers behind the reason, for logging | [`firmware/pico2/src/watchdog.rs:418-427`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L418-L427) |
| `Rp2350Watchdog::new(_handle, _clocks: &Rp2350Clocks) -> Self` | reads and classifies the reset reason, clears the marker; does not start the watchdog | [`firmware/pico2/src/watchdog.rs:482-506`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L482-L506) |
| `reset_reason(&self) -> ResetReason`, `reset_snapshot(&self) -> ResetSnapshot` | | [`firmware/pico2/src/watchdog.rs:508-516`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L508-L516) |
| `start(&mut self, timeout_us: u32) -> u32` | clamps to `1..=MAX_TIMEOUT_US`, returns the value used; µs | [`firmware/pico2/src/watchdog.rs:518-548`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L518-L548) |
| `feed(&self)` | reloads the counter; does nothing before `start` | [`firmware/pico2/src/watchdog.rs:550-562`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L550-L562) |
| `scratch(&self, Scratch) -> u32` (#3) | 0 after a chip-level reset | [`firmware/pico2/src/watchdog.rs:564-569`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L564-L569) |
| `set_scratch(&mut self, Scratch, u32)` (#3) | survives a watchdog reset; cleared by power-up, brown-out, RUN pin, debugger reset | [`firmware/pico2/src/watchdog.rs:571-576`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L571-L576) |
| `time_remaining_us(&self) -> u32`, `is_running(&self) -> bool` | | [`firmware/pico2/src/watchdog.rs:578-588`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L578-L588) |
| `force_reset(&mut self) -> !` | resets now through the watchdog; the next boot reads `WatchdogForced(Trigger)` | [`firmware/pico2/src/watchdog.rs:590-611`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L590-L611) |

### #1 (and #6): `pico2::usb` and `api::usb`

| Item | Signature or value | Notes |
|---|---|---|
| `UsbDeviceConfig { vendor_id: u16, product_id: u16, device_release: u16, manufacturer, product, serial_number: &'static str }` | from `api::usb::descriptor`, re-exported | [`api/src/usb/descriptor.rs:84-101`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L84-L101), [`firmware/pico2/src/usb/mod.rs:238-240`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L238-L240) |
| `RX_BUFFER_SIZE`, `TX_BUFFER_SIZE` | `usize` = 512 each | [`firmware/pico2/src/usb/mod.rs:248-254`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L248-L254); a full receive buffer makes the controller NAK the host, so nothing is dropped |
| `LinkEvent` | `BusReset`, `Suspended`, `Disconnected`, `Deconfigured` | [`firmware/pico2/src/usb/mod.rs:279-294`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L279-L294) |
| `UsbError` | `WouldBlock`, `NotConfigured` (byte-at-a-time traits only) | [`firmware/pico2/src/usb/mod.rs:296-304`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L296-L304) |
| `Rp2350Usb` | the device: owned controller state (control handler, endpoint-0 stage, the two 512-byte rings); no public fields | [`firmware/pico2/src/usb/mod.rs:325-326`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L325-L326) |
| `Rp2350Usb::new(_handle, _clocks: &Rp2350Clocks, timer: &Rp2350Timer, config: UsbDeviceConfig) -> Self` | resets the controller, holds the pull-up off 10 ms (`DISCONNECT_MS`), attaches | [`firmware/pico2/src/usb/mod.rs:352-424`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L352-L424) |
| `poll(&mut self)` | call at least once per ms; never waits for the host | [`firmware/pico2/src/usb/mod.rs:428-449`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L428-L449) |
| `read(&mut self, buf: &mut [u8]) -> usize` | bytes received; 0 if none | [`firmware/pico2/src/usb/mod.rs:451-459`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L451-L459) |
| `write(&mut self, data: &[u8]) -> usize` | bytes queued, possibly fewer than given; 0 while not configured | [`firmware/pico2/src/usb/mod.rs:461-480`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L461-L480) |
| `room(&self) -> usize` (**#6 only**) | bytes `write` would take now: free space in the 512-byte transmit buffer, 0 while not configured; touches no hardware | [`firmware/pico2/src/usb/mod.rs:482-495`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/usb/mod.rs#L482-L495) |
| `is_configured`, `is_suspended`, `link_up` | `(&self) -> bool` | [`firmware/pico2/src/usb/mod.rs:482-499`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L482-L499) |
| `link_epoch(&self) -> u32` | count of link-lost events since `new`, wrapping. The first enumeration advances it too, because every attach begins with a bus reset: read it after the host has configured the port | [`firmware/pico2/src/usb/mod.rs:501-509`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L501-L509) |
| `last_link_event(&self) -> Option<LinkEvent>`, `line_coding(&self) -> api::usb::cdc_acm::LineCoding`, `dtr`, `rts` | information only; the driver never acts on them. `LineCoding` is not re-exported by `pico2::usb` (only `UsbDeviceConfig` is): name it from `api` | [`firmware/pico2/src/usb/mod.rs:511-531`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L511-L531) |
| `impl Read<u8>`, `impl Write<u8>` | one byte; `UsbError` | [`firmware/pico2/src/usb/mod.rs:1066-1097`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L1066-L1097) |
| `api::usb::{setup, descriptor, cdc_acm, control, ring}` | the chip-independent device logic, 35 host tests. Public items: `setup::SetupPacket`, `Direction`, `RequestKind`, `Recipient` and the `request`, `descriptor_type` and `feature` constants; `descriptor::UsbDeviceConfig`, `device_descriptor`, `CONFIGURATION_DESCRIPTOR`, `LANGUAGE_ID_DESCRIPTOR`, `string_descriptor`, `string_descriptor_len` and the endpoint, interface and string-index constants; `cdc_acm::LineCoding`, `ControlLineState`, `StopBits`, `Parity` and the `class_request` constants; `control::ControlHandler`, `ControlAction`, `Effect`, `DeviceState`, `REPLY_CAPACITY`; `ring::ByteRing<N>`; and, in `api::usb` itself, `control_in_needs_zlp` and `bulk_in_needs_zlp` | [`api/src/usb/mod.rs:98-102`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/mod.rs#L98-L102); see [03 The api crate](03-api-crate.md) |

### #1 and #2: `pico2::common` additions

| Item | Signature | Notes |
|---|---|---|
| `reset::cycle_reset(mask: u32)` (#1) | `pub unsafe fn` | assert, release, wait for `RESET_DONE`; the mask is not complemented. Safety: the blocks in `mask` lose all state, so they must belong to the caller, and never include `IO_QSPI` (bit 7) or `PADS_QSPI` (bit 10) on an image running from flash ([`firmware/pico2/src/common/reset.rs:230-251`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/reset.rs#L230-L251)) |
| `reset::is_reset_done(mask: u32) -> bool` (#2) | `pub unsafe fn` | one read of `RESET_DONE`, no wait ([`firmware/pico2/src/common/reset.rs:211-228`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/reset.rs#L211-L228)) |
| `RegAddr` variants | `SYSCFG`, `CLOCKS`, `PSM`, `XOSC`, `PLL_SYS`, `PLL_USB`, `TIMER0`, `WATCHDOG`, `POWMAN`, `TICKS`, `USBCTRL_DPRAM`, `USBCTRL_REGS` (#1); `PWM` (#2); `ADC`, `DMA` (#5) | [`firmware/pico2/src/common/reg.rs:57-155`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/reg.rs#L57-L155) |
| Board handles | `clocks`, `timer`, `watchdog`, `usb` (#1); `pwm` (#2); `systick` (#3); `dma`, `adc` (#5); `core1` (#6) | [`firmware/pico2/src/common/board.rs:136-143`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/board.rs#L136-L143), [`firmware/pico2/src/common/board.rs:147`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/common/board.rs#L147) (PR #6) |

### #2: `pico2::pwm`

| Item | Signature or value | Notes |
|---|---|---|
| `MIN_FREQUENCY_HZ`, `MAX_FREQUENCY_HZ` | `u32` = 100, 20,000 | [`firmware/pico2/src/pwm.rs:210-214`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L210-L214) |
| `Channel { A, B }` | | [`firmware/pico2/src/pwm.rs:323-330`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L323-L330) |
| `gpio_to_slice(gpio: usize) -> usize`, `gpio_to_channel(gpio: usize) -> Channel` | `const fn`; GPIO*n* and GPIO*n*+16 share an output | [`firmware/pico2/src/pwm.rs:332-353`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L332-L353) |
| `SquareConfig { div_int: u8, div_frac: u8, top: u16 }` | | [`firmware/pico2/src/pwm.rs:394-406`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L394-L406) |
| `SquareConfig::for_frequency(clk_hz: u32, hz: u32) -> Option<Self>` | smallest divider that fits, then the nearest whole number of counts; `None` outside 100-20,000 Hz | [`firmware/pico2/src/pwm.rs:408-442`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L408-L442) |
| `divider_sixteenths`, `period_counts`, `compare_level`, `period_sixteenths`, `frequency_hz(clk_hz)` | `const fn`s on `SquareConfig` | [`firmware/pico2/src/pwm.rs:444-472`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L444-L472) |
| `PwmError` | `FrequencyOutOfRange { hz, min, max }`, `ChannelInUse { slice, channel }`, `SliceFrequencyConflict { slice, running_hz, requested_hz }`, `NoWrap { slice }` | [`firmware/pico2/src/pwm.rs:536-577`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L536-L577) |
| `Rp2350Pwm` | one per boot; records which outputs it has handed out and the period of each slice it has started; no public fields | [`firmware/pico2/src/pwm.rs:579-593`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L579-L593) |
| `impl ErrorType for Rp2350Pwm`, `Error = PwmError` | the error type of `square_from_handle` | [`firmware/pico2/src/pwm.rs:731-734`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L731-L734) |
| `Rp2350Pwm::new(_handle, _clocks: &Rp2350Clocks) -> Self` | releases `IO_BANK0`/`PADS_BANK0`, disconnects any pin a previous run left on the PWM, cycles the PWM block through reset | [`firmware/pico2/src/pwm.rs:597-625`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L597-L625) |
| `square_from_handle<const N: usize>(&mut self, PinHandle<N>, hz: u32) -> Result<Rp2350PwmSquare<N>, PwmError>` | pin `N` connected while its output is held low; first pin on a slice waits up to two periods for the first wrap | [`firmware/pico2/src/pwm.rs:627-693`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L627-L693) |
| `Rp2350Pwm::silence_all()` | `pub unsafe fn`; `CC` = 0 on every slice, nothing while PWM is in reset | [`firmware/pico2/src/pwm.rs:697-728`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L697-L728) |
| `Rp2350PwmSquare<const N: usize>` | | [`firmware/pico2/src/pwm.rs:736-751`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L736-L751) |
| `Rp2350PwmSquare::frequency_hz(&self) -> u32` | within 134 ppm of the request; within 20 ppm from 300 to 3000 Hz | [`firmware/pico2/src/pwm.rs:759-765`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L759-L765) |
| `impl Write<bool> for Rp2350PwmSquare<N>`, `Error = Infallible` | `true`: 50 % square wave; `false`: `CC` = 0; takes effect at the next wrap | [`firmware/pico2/src/pwm.rs:768-804`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L768-L804) |
| `impl Drop for Rp2350PwmSquare<N>` | `write(false)` | [`firmware/pico2/src/pwm.rs:806-811`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L806-L811) |

### #3: `pico2::systick` and the runtime

| Item | Signature or value | Notes |
|---|---|---|
| `systick::CYCLES_MASK` | `u32` = `0x00ff_ffff` | [`firmware/pico2/src/systick.rs:77-79`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/systick.rs#L77-L79); the keyer asserts its own mask equals it ([`firmware/pico2-keyer/src/main.rs:62-63`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L62-L63)) |
| `systick::CYCLES_PER_MS` | `u32` = `CLK_SYS_HZ / 1000` = 150,000 | [`firmware/pico2/src/systick.rs:81-83`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/systick.rs#L81-L83) |
| `Rp2350SysTick::new(_handle, _clocks: &Rp2350Clocks) -> Self` | free-running on the processor clock, no interrupt | [`firmware/pico2/src/systick.rs:95-109`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/systick.rs#L95-L109) |
| `Rp2350SysTick::cycles(&self) -> u32` | processor cycles modulo 2^24, counting up; wraps about every 112 ms | [`firmware/pico2/src/systick.rs:111-117`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/systick.rs#L111-L117) |
| `entry!(main)`, `entry!(main, safe_state = f)` | `f: fn()`; both signatures are type-checked at the `entry!` line | [`firmware/pico2/src/lib.rs:395-415`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L395-L415) |
| `no_safe_state()` | `pub fn`; does nothing | [`firmware/pico2/src/lib.rs:417-418`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L417-L418) |
| `__rustos_safe_state` | the symbol `entry!` defines and the handlers call | [`firmware/pico2/src/lib.rs:339-342`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L339-L342) |
| `DefaultHandler`, `OnHardFault` | `extern "C"`; safe state, then `loop {}` | [`firmware/pico2/src/lib.rs:447-469`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L447-L469) |

### #4: templates

No API. The template's manifest is [`templates/pico2/Cargo.toml.liquid`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/Cargo.toml.liquid).

### #5: `pico2::dma`, `pico2::adc`, and `clocks` additions

| Item | Signature or value | Notes |
|---|---|---|
| `clocks::CLK_ADC_HZ` | `u32` = 48,000,000, from `PLL_USB` | [`firmware/pico2/src/clocks.rs:109-112`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L109-L112) |
| `clocks::start_clk_adc()` | `pub(crate) unsafe fn`; called by `Rp2350Adc::new` only | [`firmware/pico2/src/clocks.rs:427-443`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L427-L443) |
| `Rp2350DmaChannel<const C: usize>` | zero-sized ownership of channel `C`; `number(&self) -> usize` | [`firmware/pico2/src/dma.rs:170-191`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/dma.rs#L170-L191); `C` > 15 is a build error |
| `Rp2350Dma { pub ch0 … pub ch15 }` | | [`firmware/pico2/src/dma.rs:199-234`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/dma.rs#L199-L234) |
| `Rp2350Dma::new(_handle: DeviceHandle<Rp2350Dma>) -> Self` | cycles the DMA block through reset; no clocks argument | [`firmware/pico2/src/dma.rs:236-263`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/dma.rs#L236-L263) |
| `adc::MIN_RATE_HZ`, `adc::MAX_RATE_HZ` | `u32` = 1,000, 480,000 | [`firmware/pico2/src/adc.rs:158-168`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L158-L168) |
| `adc::RING_SAMPLES` | `usize` = 8192 (16 kB) | [`firmware/pico2/src/adc.rs:170-171`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L170-L171) |
| `adc::GUARD_SAMPLES` | `usize` = 512 | [`firmware/pico2/src/adc.rs:176-182`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L176-L182) |
| `adc::SAMPLE_ERROR`, `adc::SAMPLE_VALUE` | `u16` = `1 << 15`, `0x0fff` | [`firmware/pico2/src/adc.rs:184-192`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L184-L192) |
| `RateConfig { div_int: u16, div_frac: u8 }`; `for_rate(clk_hz, hz) -> Option<Self>`, `period_256ths`, `rate_hz(clk_hz)` | the pacing divider; 32 kHz is exactly 1500 cycles | [`firmware/pico2/src/adc.rs:299-346`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L299-L346) |
| `AdcError` | `NotReady`, `RateOutOfRange { hz, min, max }`, `Overrun`, `FifoOverflow`, `DmaError` | [`firmware/pico2/src/adc.rs:391-421`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L391-L421) |
| `Rp2350Adc::new(_handle, _clocks: &Rp2350Clocks) -> Result<Self, AdcError>` | starts `clk_adc`, cycles the ADC through reset, powers it, waits for `READY` (at most 100,000 polls) | [`firmware/pico2/src/adc.rs:435-460`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L435-L460) |
| `stream_from_handle<const N: usize, const C: usize>(self, PinHandle<N>, Rp2350DmaChannel<C>, &'static mut AdcRing, hz: u32) -> Result<Rp2350AdcStream, AdcError>` | `N` must be 26-29 (a build error otherwise); consumes everything, even on error | [`firmware/pico2/src/adc.rs:462-518`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L462-L518) |
| `AdcRing` | `#[repr(C, align(16384))]`, 8192 `u16`; `const fn new()`, `Default` | [`firmware/pico2/src/adc.rs:521-549`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L521-L549) |
| `Rp2350AdcStream` | `rate_hz(&self) -> u32` | [`firmware/pico2/src/adc.rs:552-588`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L552-L588) |
| `Rp2350AdcStream::read(&mut self, out: &mut [u16]) -> Result<usize, AdcError>` | copies what arrived, oldest first; never waits | [`firmware/pico2/src/adc.rs:590-637`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L590-L637) |
| `impl Drop for Rp2350AdcStream` | stops the converter and the channel, powers the ADC down | [`firmware/pico2/src/adc.rs:670-689`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L670-L689) |

### #6 (open): `api::multicore` and `pico2::multicore`

Summarised here; [06 Multicore](06-multicore.md) has every item and the algorithm.

| Item | Signature or value | Where (at `410dffa`) |
|---|---|---|
| `api::multicore::Mailbox` (trait) | `try_send(&mut self, u32) -> bool`, `try_receive(&mut self) -> Option<u32>`, `wake(&mut self)`; every method returns at once. Implemented by `pico2::multicore::Fifo`; implement it to port the launch to another chip | [`api/src/multicore.rs:54-73`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L54-L73), [`firmware/pico2/src/multicore.rs:220-244`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L220-L244) |
| `QUIET_US`, `DRAIN_MAX` | `u64` = 100 µs, `usize` = 16 | [`api/src/multicore.rs:75-85`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L75-L85) |
| `Step { Pending, Launched }`, `Launch::new(vector_table, stack_pointer, entry: u32)`, `Launch::step(&mut self, &mut impl Mailbox, now: u64) -> Step` | the six-word handshake `[0, 0, 1, vector table, stack pointer, entry]`, one bounded step at a time | [`api/src/multicore.rs:87-189`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L87-L189) |
| `pico2::multicore::Core { Core0, Core1 }`, `core() -> Core` | which core is running; safe in a fault handler | [`firmware/pico2/src/multicore.rs:149-166`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L149-L166) |
| `Fifo` (not `Send`), `FifoErrors`, `Fifo::take_errors` | this core's ends of the inter-core FIFOs | [`firmware/pico2/src/multicore.rs:170-244`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L170-L244) |
| `Core1Stack<const WORDS: usize>` | `#[repr(C, align(32))]`, `WORDS` eight-byte words (`u64`); `MIN_WORDS` = 128 = 1 kB; `WORDS` a multiple of 4 = whole 32-byte blocks; `const fn new()`; `range(&self) -> Range<usize>`, lowest byte to one past the highest. The one-Pico image uses `Core1Stack<8192>`, 64 kB ([`firmware/pico2-keyer/src/core1.rs:64-65`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/core1.rs#L64-L65), PR #31, draft) | [`firmware/pico2/src/multicore.rs:248-303`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L248-L303) |
| `LaunchError::NoAnswer` | | [`firmware/pico2/src/multicore.rs:307-315`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L307-L315) |
| `Rp2350Core1::new(_handle)`, `launch<const WORDS>(self, &'static Core1Stack<WORDS>, entry: fn(Fifo) -> !, &Rp2350Timer, timeout_us: u64) -> Result<Fifo, LaunchError>` | `WORDS` ≥ 128 and a multiple of 4, checked at build time | [`firmware/pico2/src/multicore.rs:317-396`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L317-L396) |

## Algorithms and processing

These are the behaviours the gateway's keyer depends on for safety, step by step. The never-loosen limits themselves are tabled on [13 Safety rules][gw-13]; the ones that touch rustos are the 500 ms watchdog ([`crates/keyer-core/src/lib.rs:61-62`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/keyer-core/src/lib.rs#L61-L62)), the 1 s key-down limit ([`crates/keyer-core/src/lib.rs:45-48`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/keyer-core/src/lib.rs#L45-L48)) and the 50 ms clock check ([`crates/keyer-core/src/lib.rs:89-94`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/keyer-core/src/lib.rs#L89-L94)). **Never loosen any of them, and never weaken a rustos behaviour below that they rely on.**

### Turning a tone off: `CC` = 0, never stop the slice (#2)

The slice, its counter and the wrap are drawn in [`05-pwm-square.svg`](diagrams/05-pwm-square.svg) on [05 Drivers: I/O](05-drivers-io.md).

A PWM output is high while the slice's counter is below `CC` and low from `CC` to `TOP`. The counter only moves while the slice is enabled, so a slice stopped in the high half of a period holds its pin high. The driver therefore never stops a running slice to silence it: `write(false)`, `Drop` and `silence_all` all store `CC` = 0 with the slice left running ([`firmware/pico2/src/pwm.rs:154-163`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L154-L163)). The hardware double-buffers `CC` and `TOP` and loads them at each wrap, so a change takes effect at the next wrap, up to one period later: at 700 Hz a tone starts and stops on whole 1.43 ms periods, with no short pulse at either end ([`firmware/pico2/src/pwm.rs:93-107`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L93-L107)).

Each pin writes only its own 16-bit half of the shared `CC` register, with one store through the chip's atomic set or clear alias, so the other channel's half is never read or rewritten ([`firmware/pico2/src/pwm.rs:165-178`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L165-L178), [`firmware/pico2/src/pwm.rs:788-803`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L788-L803)).

**Safe start** (`square_from_handle`, [`firmware/pico2/src/pwm.rs:109-152`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L109-L152)):

1. For the first pin on a slice: stop the slice, program `DIV` and `TOP`, write `CC` = 0 and `CTR` = 0, start it, and wait for its first wrap (the `INTR` flag), so the copies the output uses hold `CC` = 0. The wait gives up after `2 × (period_sixteenths / 16 + 1)` reads of `INTR`, at least two periods, with `NoWrap`, stopping the slice again ([`firmware/pico2/src/pwm.rs:822-853`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L822-L853)).
2. Pad: clear `OD`, set `IE`.
3. `FUNCSEL` = 4, the PWM function.
4. Clear the pad's isolation latch `ISO` last ([`firmware/pico2/src/pwm.rs:891-911`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L891-L911)).

Before any of that, `Rp2350Pwm::new` sets back to the null (high-impedance) function every pin whose `FUNCSEL` is still the PWM from a run before a processor-only reset, and only then resets the PWM block, so a tone left playing by the last run stops ([`firmware/pico2/src/pwm.rs:863-873`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L863-L873)).

**Frequency.** `for_frequency` finds the smallest divider `d` (in sixteenths, 16 to 4095) for which the nearest half-period `h` fits in 32,768 counts, then `TOP = 2h − 1`. `const` checks over the whole range prove the produced frequency is within 134 ppm of the request, and within 20 ppm from 300 to 3000 Hz ([`firmware/pico2/src/pwm.rs:489-518`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L489-L518)). For the keyer's 700 Hz at 150 MHz that is `DIV` = 3 + 5/16 and `TOP` = 64,689 ([`firmware/pico2/src/pwm.rs:522-529`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L522-L529)).

### `silence_all` and its `RESET_DONE` guard (#2)

```rust
pub unsafe fn silence_all() {
    if !unsafe { is_reset_done(RESET_PWM) } {
        return;
    }
    for slice in 0..SLICES {
        unsafe { (&raw mut (*pwm_clr()).slice[slice].cc).write_volatile(u32::MAX) };
    }
}
```

([`firmware/pico2/src/pwm.rs:721-728`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L721-L728))

It is for a panic or fault handler, which holds none of the `Rp2350PwmSquare` values and in which `Drop` does not run. It can run at any point of a boot. If the PWM block is not out of reset, as before `Rp2350Pwm::new`, it writes nothing: the block then holds its reset values (`CC` = 0), and the datasheet does not say what a write to a block held in reset does. Otherwise it writes all ones to the clear alias of each slice's `CC`, which clears both halves, and leaves every slice running; each output goes low at its slice's next wrap, within one period (10 ms at the 100 Hz minimum). It is `unsafe` because it overrides the owners of the outputs without telling them, so it belongs only where no owner runs again ([`firmware/pico2/src/pwm.rs:697-720`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L697-L720)). **Never-loosen:** call it after the key and PTT are open, never before; keep the `RESET_DONE` guard.

### The fault safe state (#3)

![Sequence of a fault or panic in the keyer image: the handler calls safe_state, which opens key and PTT in one store, silences every PWM output, holds the tone pin low and, in the one-Pico image, sets the fault flag; core 0 then trips and stops feeding, and the 500 ms watchdog resets the chip](diagrams/08-fault-safe-state.svg)

*Figure: steps 1 to 3 run on every fault or panic, in that order; notice that only the watchdog, once started, resets the chip, and that a stack overflow may stop a core before step 1.*

A fault on either core runs rustos's `OnHardFault` or `DefaultHandler`, which calls `__rustos_safe_state()`, the image's `safe_state()`; a panic runs the image's panic handler, which calls the same function. Then:

1. `outputs_open()`: one store of bits 16 and 17 to SIO `GPIO_OUT_CLR` (`0xd000_0020`): key and PTT open.
2. `Rp2350Pwm::silence_all()`: `CC` = 0 on every running slice.
3. `tone_low()`: GP18's `OUTOVER` set to "drive low", only once GP18 is a PWM output.
4. One-Pico image only: the `LINK` fault flag set. If core 1 faulted, core 0 sees the flag on its next pass, trips `CORE1`, keeps every output open and stops feeding the watchdog.
5. `loop {}`. If the watchdog has started, it resets the chip within 500 ms of the last feed. Before `watchdog.start` (a fault in `Rp2350Clocks::new` or `Rp2350Usb::new`, say), nothing resets the chip: the core stays in the loop with the outputs open until the board is unplugged ([`firmware/pico2-keyer/src/main.rs:279-288`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L279-L288)).

rustos's part is small and fixed: `OnHardFault` and `DefaultHandler` call the application's safe state first and then spin ([`firmware/pico2/src/lib.rs:447-469`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L447-L469)). `entry!` type-checks the function as `fn()` ([`firmware/pico2/src/lib.rs:395-415`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L395-L415)). The safe state runs with the program in whatever state faulted it, so it must only write the registers it needs: no locks, no allocation, no driver state. A fault inside it, from the HardFault handler, locks the processor up until a reset ([`firmware/pico2/src/lib.rs:380-394`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/lib.rs#L380-L394)).

The keyer's safe state is [`firmware/pico2-keyer/src/main.rs:236-277`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L236-L277): `outputs_open` first, then `silence_all`, then `tone_low`, and its panic handler calls the same function ([`firmware/pico2-keyer/src/main.rs:279-288`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L279-L288)). `check_faults.py` reads the built ELF and passes only if the first store each of `OnHardFault`, `DefaultHandler` and the panic handler makes, other than to the stack, puts bits 16 and 17 into `GPIO_OUT_CLR` ([`firmware/pico2-keyer/check_faults.py:1-41`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/check_faults.py#L1-L41)). The decoder image's safe state drives GP16-GP18 low with one store ([`firmware/pico2-decoder/src/main.rs:197-204`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/src/main.rs#L197-L204)). The one-Pico image adds the fault flag last, because the handlers run on whichever core faulted ([`firmware/pico2-keyer/src/main.rs:332-355`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L332-L355); [`firmware/pico2/src/lib.rs:397-400`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/lib.rs#L397-L400)). **Never-loosen:** the first store of every handler opens key and PTT together.

The watchdog does the rest: its reset scope resets everything but the oscillators, and a reset that goes through the power-on state machine returns non-isolated pads to their reset state (output disabled, internal pull-down) ([`firmware/pico2/src/watchdog.rs:151-183`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L151-L183)). The datasheet does not state the watchdog case outright and it has not been checked on hardware, which is why the keyer drives its outputs low first thing after every boot instead of relying on either reading.

The internal pull-down is not enough on its own: erratum RP2350-E9 can leave a pad that nothing drives near 2 V, enough to turn on the box's switch and key the radio. The keyer box therefore **requires** an external 4.7 kΩ pull-down from GP16 (key) to ground and another from GP17 (PTT) to ground, which hold both lines low while the chip starts, resets or is unplugged ([`docs/keyer.md:54-65`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/docs/keyer.md#L54-L65), [`docs/keyer.md:591-593`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/docs/keyer.md#L591-L593); the keyer's own summary is at [`firmware/pico2-keyer/src/main.rs:13-18`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L13-L18)). They are part of the box's wiring, not of the Pico 2 board, and no rustos code can stand in for them. **Never loosen:** the 4.7 kΩ pull-downs are required, and so is the inverting switch between each pin and the radio ([13 Safety rules][gw-13]).

### The timer that no debugger pauses, and the clock cross-check (#3)

`TIMER0`'s `DBGPAUSE` resets to `0b110`: pause while a debugger halts either core. With that value, a debugger that halts core 1 alone freezes the time core 0 reads while core 0 runs on and keeps feeding the watchdog, so a key-down limit measured on the timer never expires. `Rp2350Timer::new` writes 0, the same never-pause policy as the watchdog's pause bits, which `start` clears ([`firmware/pico2/src/timer.rs:42-56`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/timer.rs#L42-L56), [`firmware/pico2/src/watchdog.rs:65-75`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L65-L75)). The cost: stepping through code with a debugger sees time run on and the watchdog fire. **Never-loosen:** do not restore the pause bits.

The timer and the watchdog both count the 1 µs tick that the `TICKS` block divides from the crystal. If that tick stops, every deadline stops with it. `Rp2350SysTick` counts the 150 MHz processor clock instead, a separate path ([`firmware/pico2/src/systick.rs:6-24`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/systick.rs#L6-L24)). The keyer's `ClockCheck` reads both each pass. Once either has run 50 ms, it compares the timer's elapsed milliseconds `clock` with the SysTick's `cpu` (cycles ÷ 150,000, with the 24-bit count accumulated modulo 2^24) and trips the box if `clock * 2 < cpu || cpu * 4 < clock` ([`crates/keyer-core/src/control.rs:121-137`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/keyer-core/src/control.rs#L121-L137)). The first test catches a stopped or paused tick; the second catches a SysTick that is not counting, so the check fails as a trip rather than in silence. A trip opens the outputs and stops feeding the watchdog. Nothing in software can notice the processor clock itself stopping; the systick module says that case needs hardware outside the chip ([`firmware/pico2/src/systick.rs:19-24`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/systick.rs#L19-L24)).

### The watchdog: reset reason and scratch registers (#1, #3)

The whole cycle (read the evidence, start, save and feed, reset, read it back) is drawn in [`04-watchdog-reset.svg`](diagrams/04-watchdog-reset.svg) on [04 Drivers: core](04-drivers-core.md).

**Reset reason.** `Rp2350Watchdog::new` reads `WATCHDOG.REASON`, `POWMAN.CHIP_RESET` and `SCRATCH4` before writing anything, classifies them, and clears its own marker ([`firmware/pico2/src/watchdog.rs:490-506`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L490-L506)). `start` writes `SCRATCH_MARKER` into `SCRATCH4` because the bootrom's reboot routine (used by `picotool reboot` and at the end of a UF2 copy) also leaves `REASON.TIMER` set; the marker separates "this application's watchdog ran out" from "something rebooted the chip on purpose" ([`firmware/pico2/src/watchdog.rs:96-121`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L96-L121)):

| `REASON` | marker in `SCRATCH4` | `ResetReason` |
|---|---|---|
| `TIMER` | ours | `WatchdogTimeout` |
| `FORCE` | any | `WatchdogForced(Trigger)` |
| `TIMER` | not ours | `WatchdogForced(Reboot)` |
| 0 | any | `ChipReset(cause from CHIP_RESET)` |

The keyer maps `WatchdogTimeout` to its `Boot::Watchdog`; `ChipReset` with cause `PowerOn`, `RunPin`, `DebuggerReset`, `DebuggerRescue` or `RiscvDebuggerReset` to `Boot::Power`; and everything else, including brown-out and any forced or bootrom reboot, to `Boot::Other` ([`firmware/pico2-keyer/src/main.rs:205-215`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L205-L215)). A watchdog boot comes up tripped ([`crates/keyer-core/src/keyer.rs:406-409`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/keyer-core/src/keyer.rs#L406-L409)). The bootrom's reboot after a UF2 copy rewrites `SCRATCH4` and, by the driver's reading of the bootrom source, leaves `REASON.TIMER` set (unverified on hardware, [`firmware/pico2/src/watchdog.rs:96-127`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L96-L127)), which reads as `WatchdogForced(Reboot)`. So the first boot after flashing is expected to report OTHER (inferred from this mapping, as the audit's KB-7 notes in project file `audit/req-keyer-box-fixes.md`; not yet seen on hardware). That is why the bench steps say to replug: the 2026-10-09 check after a replug reported POWER (project file `bench/quansheng.md`, 16:19Z).

**Scratch registers.** `SCRATCH0` to `SCRATCH7` keep their contents through every reset short of a chip-level one, and every chip-level reset (power-on, brown-out, RUN pin, debugger reset) clears them. The bootrom reads `SCRATCH4` to `SCRATCH7` as its boot vector and passes reboot parameters in `SCRATCH2` and `SCRATCH3`, so the `Scratch` enum names only `SCRATCH0` and `SCRATCH1` ([`firmware/pico2/src/watchdog.rs:129-149`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L129-L149)). The keyer stores its `Saved` state there on every pass ([`firmware/pico2-keyer/src/main.rs:127-130`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L127-L130)) and reads it at boot ([`firmware/pico2-keyer/src/main.rs:197-201`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L197-L201)). The encoding ([`crates/keyer-core/src/keyer.rs:241-272`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/keyer-core/src/keyer.rs#L241-L272)):

- word 0: `0x4b59` (`MAGIC`) in bits 31:16, flags in bits 15:8 (the trip's index in bits 2:0, key-or-PTT-down in bit 3), a check byte in bits 7:0;
- word 1: the duty budget in milliseconds, as an `i32`;
- `decode` returns `None` unless the magic, the check byte and the unused flag bits all match, so zeros after a power-up, or another firmware's leftovers, give a fresh start.

`Keyer::restore` keeps a saved trip, takes the lower of the fresh and the saved duty budget (less `RESTART_CHARGE_MS` if the key was down), and rests before the first run ([`crates/keyer-core/src/keyer.rs:415-420`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/keyer-core/src/keyer.rs#L415-L420)). So a host that makes the box restart cannot clear a trip or get a fresh duty budget out of it.

### The ADC and DMA ring (#5)

The pin, the FIFO, the endless channel, the ring and the guard slot are drawn in [`05-adc-dma-ring.svg`](diagrams/05-adc-dma-ring.svg) on [05 Drivers: I/O](05-drivers-io.md).

**Set-up** (`stream_from_handle`, [`firmware/pico2/src/adc.rs:462-518`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L462-L518)):

1. The pin (26-29) goes to the ADC: null function, input buffer off, `OD` on, both pulls off, isolation released last. On the RP2350 an enabled input buffer on a pad held near mid-supply, as a biased audio input is, sources current and settles near 2.2 V (erratum E9), so the digital side must be off ([`firmware/pico2/src/adc.rs:90-113`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L90-L113), [`firmware/pico2/src/adc.rs:754-765`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L754-L765)).
2. The ADC's input and pacing divider are set, the FIFO is drained, and the DMA request (`DREQ_ADC`, 48) is enabled.
3. Every ring slot is set to the sentinel `0xffff`. The converter never writes it: a sample has its value in bits 11:0, an error flag in bit 15, and bits 14:12 reserved ([`firmware/pico2/src/adc.rs:188-203`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L188-L203)).
4. The DMA channel starts: endless mode (`TRANS_COUNT.MODE` = `0xf`, count 1), halfwords from the ADC FIFO without incrementing, into the ring with the write address wrapping on its low 14 bits (`RING_SIZE` = 14), which needs the ring aligned to 16 kB; `AdcRing`'s type guarantees that ([`firmware/pico2/src/adc.rs:708-725`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L708-L725)).
5. Last, the converter starts free-running.

**Pacing.** A conversion starts every `1 + DIV.INT + DIV.FRAC/256` cycles of the 48 MHz `clk_adc`; `for_rate` picks `INT` and `FRAC` for the requested rate. 32 kHz is exactly 1500 cycles (`INT` = 1499); 44.1 kHz is `INT` = 1087, `FRAC` = 111 ([`firmware/pico2/src/adc.rs:309-376`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L309-L376)).

**Reading** (`read`, [`firmware/pico2/src/adc.rs:606-637`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L606-L637)), never waiting:

1. If the channel reports a bus error (`CTRL.AHB_ERROR`), return `DmaError` now and on every later call.
2. If the ADC FIFO overflowed (`FCS.OVER`), clear the flag and return `FifoOverflow`; the ring's samples are still good.
3. `written` = the channel's `WRITE_ADDR` as a ring index (an acquire fence keeps the slot reads after it, [`firmware/pico2/src/adc.rs:639-648`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L639-L648)).
4. Look at the slot `GUARD_SAMPLES` (512) ahead of `written`. If it does not hold the sentinel, the channel has gone a whole lap past the reader: resynchronise and return `Overrun`. The guard keeps a channel that moves on a few samples between the two reads from looking like an overrun.
5. Otherwise copy up to `out.len()` samples from the read index towards `written`, writing the sentinel back into each slot as it is read.

An endless channel cannot count its laps, which is why the sentinel exists ([`firmware/pico2/src/adc.rs:68-80`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L68-L80)). Overrun is reported once reading lags by `RING_SAMPLES − GUARD_SAMPLES` = 7,680 samples, 240 ms at 32 kHz. **Resync** ([`firmware/pico2/src/adc.rs:650-667`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L650-L667)) sets every slot except the 512 from the channel's position back to the sentinel while the channel keeps writing, then restarts reading at that position; if the channel moved 512 or more slots meanwhile, it does it again. **Stopping** (`Drop`, [`firmware/pico2/src/adc.rs:670-689`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L670-L689)) clears free-running, waits for `CS.READY`, aborts the channel with the erratum E5 workaround ([`firmware/pico2/src/dma.rs:266-295`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/dma.rs#L266-L295)), drains the FIFO and powers the ADC down.

The decoder maps the three runtime errors to its own `AudioLoss` ([`firmware/pico2-decoder/src/main.rs:117-124`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/src/main.rs#L117-L124)).

### Multicore: the launch, `MSPLIM` and an MPU-ready stack (#6, open)

The handshake is drawn in [`06-core1-launch.svg`](diagrams/06-core1-launch.svg) and the stack, its limit and its MPU region in [`06-core1-stack.svg`](diagrams/06-core1-stack.svg), both on [06 Multicore](06-multicore.md).

Core 1 waits in the bootrom after every reset until core 0 sends it six words over the inter-core FIFO, each echoed back: 0, 0, 1, the vector table, the stack pointer, the entry point. `Launch::step` does at most a bounded drain, one send and one receive per call, so `Rp2350Core1::launch` can bound the whole handshake by the timer and return `NoAnswer` ([`firmware/pico2/src/multicore.rs:350-395`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L350-L395)). The entry point is a naked trampoline, `core1_start`, that loads the stack's lowest address and writes it to `MSPLIM` before anything is pushed, so a stack overflow on core 1 faults instead of overwriting memory below the stack; then `core1_run` enables core 1's floating-point unit and calls the application's function ([`firmware/pico2/src/multicore.rs:398-430`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L398-L430)). `Core1Stack` is 32-byte aligned and `launch` accepts only whole 32-byte blocks, because a Cortex-M33 MPU region's base and limit are 32-byte granular; `range()` gives the addresses ([`firmware/pico2/src/multicore.rs:248-296`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L248-L296)). The one-Pico image uses that range for core 1's stack region ([`firmware/pico2-keyer/src/core1.rs:266-287`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/core1.rs#L266-L287)) and sets core 0's own `MSPLIM` itself ([`firmware/pico2-keyer/src/main.rs:357-371`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L357-L371)). The step-by-step handshake and its nine tests are on [06 Multicore](06-multicore.md).

What follows an overflow is analysis, not tested on a chip. Exception entry pushes a frame on the same stack and the HardFault handler needs stack too, and rustos leaves `CCR.STKOFHFNMIGN` at 0, so the core can lock up before its safe state runs ([06 Multicore](06-multicore.md) gives the Arm rules). The audit read core 0 in the one-Pico image the same way: "a core 0 stack overflow locks up at MSPLIM (OnHardFault pushes first) and the watchdog opens the key within 0.5 s" (project file `audit/report.md` §3, PR #31 entry at `8d5fcf6`). After an overflow, then, the key opens because of the 500 ms watchdog (an overflow on core 0) or core 0's 100 ms watch of core 1's heartbeat (an overflow on core 1; `CORE1_MS` at [`crates/keyer-core/src/lib.rs:110`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/crates/keyer-core/src/lib.rs#L110), PR #31, draft), not because of the safe state.

### `Rp2350Usb::room()`: all or nothing (#6, open)

`write` may accept only part of a slice. `room()` returns how many bytes `write` would take right now: the transmit ring's free space (`ByteRing::free`, `N − len`, [`api/src/usb/ring.rs:48-51`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L48-L51)), or 0 while the device is not configured. The contract: a write of at most `room()` bytes, with no other write before it, is taken whole; a `poll` in between only adds room as packets leave, unless it ends the session, and then the write takes nothing ([`firmware/pico2/src/usb/mod.rs:482-495`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/usb/mod.rs#L482-L495)). The one-Pico keyer uses it to pass a decoder line to the host only when the line, its newline and a whole keyer reply after it all fit: `hw.usb_room() >= l.len() + 1 + LINE_BYTES` ([`crates/keyer-core/src/control.rs:641`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/crates/keyer-core/src/control.rs#L641), with `LINE_BYTES` at [`crates/keyer-core/src/control.rs:131-133`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/crates/keyer-core/src/control.rs#L131-L133)). A keyer reply is therefore never cut short by decoder output.

## Programming against it

### Name a safe state for your image

```rust
#![no_std]
#![no_main]

use api::common::Write;
use api::gpio::Gpio;
use pico2::common::board::Rp2350;
use pico2::gpio::gpio::Rp2350Gpio;

pico2::entry!(main, safe_state = safe_state);

fn main() -> ! {
    let board = Rp2350::take().unwrap();
    // First of all, before the clocks: every output that can key a radio, driven low.
    let mut gpio = Rp2350Gpio::new(board.gpio);
    let mut key = gpio.output_from_handle(board.pins.gpio16).unwrap();
    let _ = key.write(false);
    // ... then the clocks, the timer, the watchdog, USB, and the loop.
    loop {
        core::hint::spin_loop();
    }
}

/// Runs first in OnHardFault and DefaultHandler (and call it from your panic
/// handler). Plain register stores only: no driver, no lock.
fn safe_state() {
    // 1. Whatever can key a radio, open, in one store (SIO GPIO_OUT_CLR).
    const SIO_GPIO_OUT_CLR: *mut u32 = 0xd000_0020 as *mut u32;
    unsafe { SIO_GPIO_OUT_CLR.write_volatile((1 << 16) | (1 << 17)) };
    // 2. Then every PWM output off. Safe at any point of a boot.
    unsafe { pico2::pwm::Rp2350Pwm::silence_all() };
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    safe_state();
    loop { core::hint::spin_loop(); }
}
```

This is a shortened form of the keyer's safe state ([`firmware/pico2-keyer/src/main.rs:246-261`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L246-L261)), which also holds the tone pin low as a third step. If your image keys anything, add a check like `check_faults.py` to its CI.

### Leave a note across a watchdog reset

```rust
use pico2::watchdog::{ResetReason, Rp2350Watchdog, Scratch};

let mut wd = Rp2350Watchdog::new(board.watchdog, &clocks); // reads the reason first
let note = [wd.scratch(Scratch::Scratch0), wd.scratch(Scratch::Scratch1)];
let after_timeout = wd.reset_reason() == ResetReason::WatchdogTimeout;
// ... decode `note` with a magic number and a check, as keyer_core::keyer::Saved does:
// after a power-up both words are 0.
wd.start(500_000); // µs
loop {
    // ... one pass of work ...
    wd.set_scratch(Scratch::Scratch0, word0);
    wd.set_scratch(Scratch::Scratch1, word1);
    wd.feed();
}
```

### Sample audio into a ring

```rust
use pico2::adc::{AdcError, AdcRing, Rp2350Adc};
use pico2::dma::Rp2350Dma;

static mut RING: AdcRing = AdcRing::new(); // 16 kB, 16 kB-aligned, in .bss

let dma = Rp2350Dma::new(board.dma);
let adc = Rp2350Adc::new(board.adc, &clocks).unwrap();
let ring = unsafe { &mut *(&raw mut RING) }; // the only reference ever made
let mut audio = adc.stream_from_handle(board.pins.gpio26, dma.ch0, ring, 32_000).unwrap();
let mut block = [0u16; 64];
loop {
    match audio.read(&mut block) {
        Ok(n) => { /* block[..n], oldest first; mask with SAMPLE_VALUE, check SAMPLE_ERROR */ }
        Err(AdcError::Overrun) => { /* fell 240 ms behind: samples lost, reading restarts */ }
        Err(AdcError::FifoOverflow) => { /* conversions lost; ring still good */ }
        Err(_) => { /* DmaError: no more audio this boot */ }
    }
    // read at least every 240 ms at 32 kHz
}
```

This is the decoder box's sequence ([`firmware/pico2-decoder/src/main.rs:170-179`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/src/main.rs#L170-L179)) and the module's own example ([`firmware/pico2/src/adc.rs:5-29`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L5-L29)). To read the ring from core 1 running unprivileged, the channel's security level must first be lowered, which rustos has no API for; see [Known bugs, limits and flakes](#known-bugs-limits-and-flakes).

### Start core 1 (PR #6, open)

```rust
use pico2::multicore::{Core1Stack, Fifo, Rp2350Core1};

// 2048 eight-byte words = 16 kB, a whole number of 32-byte blocks.
static CORE1_STACK: Core1Stack<2048> = Core1Stack::new();

fn core1_main(fifo: Fifo) -> ! {
    loop { /* core 1's work; MSPLIM is already at the stack's bottom */ }
}

// In main, after the clocks and the timer:
let fifo = Rp2350Core1::new(board.core1)
    .launch(&CORE1_STACK, core1_main, &timer, 10_000) // Err(NoAnswer) after 10 ms
    .unwrap();
// For an MPU region over core 1's stack: 32-byte aligned at both ends.
let stack = CORE1_STACK.range();
```

This follows the module's own example ([`firmware/pico2/src/multicore.rs:4-24`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L4-L24)). `launch` refuses at build time a stack smaller than 128 words or not a multiple of 4. Start core 1 last, after the watchdog, and watch it from core 0, as the one-Pico image does; [06 Multicore](06-multicore.md) has the rest.

### Send a message whole, or not yet (PR #6)

```rust
if usb.room() >= msg.len() {
    let n = usb.write(msg); // takes all of msg
    debug_assert_eq!(n, msg.len());
} // else: keep msg, poll, try next pass
```

### Build against a rustos branch or a local checkout

Pin a `rev` in the application's `Cargo.toml` as the gateway does ([`firmware/pico2-keyer/Cargo.toml:30-31`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/Cargo.toml#L30-L31)), or use the `[patch]` section described on [01 Overview](01-overview.md) to build against a local checkout. Remember that every pin move gives a new UF2.

The keyer firmware is its own Cargo workspace and takes the patch on the command line, from `firmware/pico2-keyer` with rustos checked out beside cw-message-gateway ([`firmware/pico2-keyer/README.md:67-73`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/README.md#L67-L73)):

```sh
cargo build --release \
  --config 'patch."https://github.com/robinonsay/rustos".api.path="../../../rustos/api"' \
  --config 'patch."https://github.com/robinonsay/rustos".pico2.path="../../../rustos/firmware/pico2"'
```

That is a build check only. Never flash what it gives, and do not edit the gateway's pin to try a change on hardware ([01 Overview](01-overview.md)): only a UF2 the audit has cleared by its hash is flashed.

## Changing it

| To change | Touch | Then |
|---|---|---|
| Any driver the keyer uses (clocks, timer, SysTick, watchdog, USB, GPIO, PWM) | its module under `firmware/pico2/src/` | Nothing reaches the box until the gateway moves its pin; when it does, the new UF2 needs a fresh audit and a new hash ([14 Audit, bench and flashing][gw-14]). Run `cargo test` and `cargo pico2` in rustos, and build the keyer image against your checkout with the command in [Build against a rustos branch or a local checkout](#build-against-a-rustos-branch-or-a-local-checkout) (a build check only). |
| The PWM driver | `pwm.rs` | Keep: off by `CC` = 0 with the slice running; `Drop` turning the tone off; `silence_all`'s `RESET_DONE` guard; the safe-start order with `ISO` cleared last; `new` disconnecting stale PWM pins before the reset. Keep the `const` frequency checks passing. |
| The runtime's fault handlers | `lib.rs` | Keep: the safe state called first, before anything else, in both handlers; `entry!` type-checking it. Rebuild the keyer and run its `check_faults.py`. |
| The timer | `timer.rs` | Keep `DBGPAUSE` = 0 and the race-free `now()`. Core 1's MPU in the one-Pico image allows only reads of `TIMER0` from `0x400b_0020` to `0x400b_003f`, so `now()` must keep using `TIMERAWH`/`TIMERAWL` ([`firmware/pico2-keyer/src/core1.rs:260-263`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/core1.rs#L260-L263)). |
| The watchdog | `watchdog.rs` | Keep: pause bits cleared, the reset scope, the marker in `SCRATCH4`, `Scratch` limited to `SCRATCH0`/`SCRATCH1`. `keyer_core::keyer::Saved` depends on the two words surviving a watchdog reset. |
| The ADC or DMA | `adc.rs`, `dma.rs` | Keep the sentinel's value outside every possible sample, the ring's alignment, and the pin's digital side off. Core 1's MPU in the one-Pico image allows `read` to touch only `ADC` registers `0x400a_0000`-`0x400a_001f` and DMA channel 0's first 32 bytes ([`firmware/pico2-keyer/src/core1.rs:253-259`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/core1.rs#L253-L259)); a new register access in `read` faults there. The image's own `SECCFG_CH0` write (see [Known bugs, limits and flakes](#known-bugs-limits-and-flakes)) relies on `Rp2350Dma::new` resetting the whole DMA block, and on no rustos code writing channel 0's control registers between that reset and the image's `SECCFG` write, which it makes just before `stream_from_handle`: a control write locks `SECCFG` at Secure Privileged, and the image would then start no core 1 ([`firmware/pico2-keyer/src/core1.rs:123-136`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/core1.rs#L123-L136)). |
| Multicore, `room()` | `multicore.rs`, `api/src/multicore.rs`, `usb/mod.rs` (PR #6) | See [06 Multicore](06-multicore.md). Merging #6 means the one-Pico image re-pins to master's merge: a new head and a new hash. |
| The template | `templates/pico2/` | Keep the manifest named `Cargo.toml.liquid` (#4), or every downstream build prints an error again. |
| Fault handler names, `enable_fpu`, the `pico2::multicore` statics, or how drivers form register addresses | `lib.rs`, `multicore.rs`, any driver | The gateway's CI reads the built images by symbol name and by instruction shape, so a rename or refactor that is fine inside rustos can fail those checks the moment a pin moves. `check_faults.py` finds `OnHardFault` and `DefaultHandler` by name ([`firmware/pico2-keyer/check_faults.py:41`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/check_faults.py#L41)). `check_pins.py` (decoder box, PR #30) needs every handler's first store to drive GP16-GP18 low through `GPIO_OUT_CLR` (rule 5), accepts a `CPACR` store only as `enable_fpu`'s straight-line load, `orr` and store (rule 7), and assumes rustos drivers keep register addresses as constants, never loaded from memory ([`firmware/pico2-decoder/check_pins.py:26-62`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/check_pins.py#L26-L62)). `check_core1.py` (PR #31, draft) finds `CORE1_ENTRY` and `CORE1_STACK_LIMIT` by mangled name, walks core 1's code from `core1_start` and `core1_run`, names `__rustos_main`, and watches stores into `TIMER0` and the DMA ([`firmware/pico2-keyer/check_core1.py:71-96`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/check_core1.py#L71-L96)). CI runs all three ([`.github/workflows/ci.yml:195-204`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/.github/workflows/ci.yml#L195-L204), [`.github/workflows/ci.yml:278-279`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/.github/workflows/ci.yml#L278-L279), PR #31's tree). Rename or reshape any of these and the gateway PR that moves the pin must update the check in the same PR; the audit then re-checks the new image. |

**Before you move a gateway pin.** Build the image with `build.sh` at the commit, then rebuild with the currently audited build id and compare byte for byte, so you know whether the pin move alone changed the file. Either way the new head goes to the safety audit; the box keeps running the cleared file until a new hash is cleared. Never describe or use a way around that.

## Known bugs, limits and flakes

- **Untested on hardware:** the ADC and DMA drivers (#5), multicore and `room()` (#6), the fault safe state and `silence_all` on a real fault, a tone actually playing, the watchdog timeout path with its scratch carry-over, and USB link-loss detection (unplug, suspend). See [What has run on a Pico 2](#what-has-run-on-a-pico-2), and K2 on [09 Work left and known issues](09-work-left-known-issues.md) for the other USB points the code marks as unverified.
- **A stack overflow may lock a core up before its safe state runs** (analysis, untested): see [Multicore: the launch, `MSPLIM` and an MPU-ready stack](#multicore-the-launch-msplim-and-an-mpu-ready-stack-6-open). The 500 ms watchdog, or core 0's watch of core 1 in the one-Pico image, is then what opens the key.
- **No API for a DMA channel's security level.** Every channel resets Secure Privileged, and only a reset of the whole block clears its lock ([`firmware/pico2/src/dma.rs:29-51`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/dma.rs#L29-L51)). Core 1 in the one-Pico image runs unprivileged and reads channel 0's registers. The audit found, by reading the code of the first audited version of PR #31 (`8d5fcf6`, never flashed), that this would bus-fault on core 1's first decoder pass and reset the box about every 0.5 s, with the key and PTT open throughout (project file `audit/report.md` §3, PR #31 entry). The image now writes `SECCFG_CH0` directly (`0x5000_0480`, Secure Unprivileged, then locked, read back) after `Rp2350Dma::new` and before the stream starts, and starts no core 1 unless it reads back so ([`firmware/pico2-keyer/src/core1.rs:104-136`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/core1.rs#L104-L136)). The audit cleared that fix by reading (`bc5062a`); it has not run on hardware. It is a raw register write outside rustos that depends on `Rp2350Dma::new` having just reset the block.
- **The bootrom and `SCRATCH4`.** The watchdog's marker assumes the bootrom clears `SCRATCH4` only for a valid boot-vector request; one sentence of the datasheet speaks of "the bootrom clearing SCRATCH4 each boot", which would make every timeout read as `WatchdogForced(Reboot)`. Unverified on hardware ([`firmware/pico2/src/watchdog.rs:104-110`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L104-L110)). Also unverified: any side effect of leaving `POWMAN` on the low-power oscillator after `start` ([`firmware/pico2/src/watchdog.rs:60-63`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L60-L63)), and whether the watchdog counts once per tick on this chip ([`firmware/pico2/src/timer.rs:30-40`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/timer.rs#L30-L40)).
- **Pads across a watchdog reset** depend on a reading of the datasheet that has not been checked on hardware ([`firmware/pico2/src/watchdog.rs:175-183`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/watchdog.rs#L175-L183)). The rule that makes it not matter: drive every output that can key a radio to its safe level first thing after boot, and have the external circuit treat a floating pin as off.
- **Unbounded waits.** `Rp2350AdcStream`'s `Drop` waits on `CS.READY` without a limit (at most 96 `clk_adc` cycles by the datasheet), and `dma::abort` polls `CHAN_ABORT` without a limit ([`firmware/pico2/src/adc.rs:670-689`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L670-L689), [`firmware/pico2/src/dma.rs:278-295`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/dma.rs#L278-L295)). `Rp2350Clocks::new` waits forever if the crystal never starts ([`firmware/pico2/src/clocks.rs:374-382`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/clocks.rs#L374-L382)). None runs in the keyer's loop.
- **`AdcError::NotReady` leaves the ADC powered** ([`firmware/pico2/src/adc.rs:395-397`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L395-L397)). The one-Pico image then runs without a decoder.
- **One ADC stream and one launch per boot**, by construction: `stream_from_handle` and `launch` consume their drivers.
- **A processor clock that stops** cannot be detected by any software on the chip; the clock check covers a stopped tick only.
- **No CI in rustos**, so none of this is checked automatically there; the gateway's CI builds the images and runs `check_faults.py`, `check_core1.py` (PR #31) and `check_pins.py` (PR #30) on them, which tie rustos's symbol names and code shape to the gateway (see [Changing it](#changing-it)).

No flaky tests are known in the code described here.

## Work left

1. **Merge rustos #6**, then move PR #31's pin from `410dffa` to master's merge. That is a new keyer head: the audit re-checks it and posts a new hash before anything is flashed ([`firmware/pico2-keyer/Cargo.toml:24-31`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/Cargo.toml#L24-L31)). PR #31's tree also keeps the decoder crate pinned at `80bf599` ([`firmware/pico2-decoder/Cargo.toml:24-25`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-decoder/Cargo.toml#L24-L25)); moving that pin too is a separate decision, and any move gives a new UF2 that needs its own audit hash.
2. **First hardware runs**, each through the gateway's bench and audit steps: the decoder box image (PR #30), and the one-Pico image after PR #31 merges. Both exercise #5 for the first time. The one-Pico image's first boot is the first launch of core 1 on a chip; its bring-up step 11 (`core1test`, then `core1test --fault`) tests core 1's hang and fault paths ([`docs/keyer.md:321-330`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/docs/keyer.md#L321-L330), PR #31).
3. **The keyer's stage 7** (`hangtest`, `stucktest`, `linktest`) exercises the watchdog timeout, the reset reason and the scratch carry-over on a chip for the first time.
4. **Optional:** a DMA channel security option in rustos, so the one-Pico image need not write `SECCFG` itself. Not needed now; any change to that path is a change to an audited image.
5. **CI for rustos** (at least `cargo test` and `cargo pico2`).
6. **Optional:** set core 0's `MSPLIM` in rustos's `OnReset`, so every image gets a fault rather than silent corruption when core 0's stack overflows ([02 Boot and runtime](02-boot-runtime.md); K7 on [09 Work left and known issues](09-work-left-known-issues.md)). The one-Pico image sets it itself today ([`firmware/pico2-keyer/src/main.rs:357-371`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L357-L371)). Any change to that path changes every audited image that pins it.

The whole rustos list is on [09 Work left and known issues](09-work-left-known-issues.md); the gateway's is [16 Work left and known bugs][gw-16].

## Sources

- rustos at `80bf599` (master): `firmware/pico2/src/{lib,clocks,timer,systick,watchdog,pwm,dma,adc}.rs`, `firmware/pico2/src/usb/mod.rs`, `firmware/pico2/src/common/{board,reg,reset}.rs`, `api/src/usb/{mod,descriptor,ring}.rs`, `templates/README.md`, `templates/pico2/Cargo.toml.liquid`; `git log`, `git diff --stat` and commit messages for each PR's merge and commits (`02b762b`, `5123db5`, `c985211`); tree comparisons of each PR head with its merge.
- rustos PR #6 at `410dffa`: `api/src/multicore.rs`, `firmware/pico2/src/multicore.rs`, `firmware/pico2/src/usb/mod.rs`, `firmware/pico2/src/common/board.rs`, `firmware/pico2/src/lib.rs`; commit messages of `3d8af9c`, `422320d`, `410dffa`.
- GitHub pull request metadata (state, branch, head, merge time, merge commit) for rustos #1-#6 and cw-message-gateway #11-#16, #24, #30, #31; `git merge-base --is-ancestor` for which merge carried each gateway PR's head to `main`.
- cw-message-gateway at `44185fa` (main): `firmware/pico2-keyer/{Cargo.toml,build.sh,check_faults.py,README.md,src/main.rs}`, `crates/keyer-core/src/{lib,control,keyer}.rs`, `docs/keyer.md` (wiring, the 4.7 kΩ pull-downs, what not to wire); at `3efaf84` (PR #30): `firmware/pico2-decoder/{Cargo.toml,check_pins.py,src/main.rs}`, `crates/decoder-core/src/lib.rs`; at `bc5062a` (PR #31): `firmware/pico2-keyer/{Cargo.toml,check_core1.py,src/main.rs,src/core1.rs}`, `firmware/pico2-decoder/Cargo.toml`, `crates/keyer-core/src/{lib,control}.rs`, `docs/keyer.md` (bring-up step 11), `.github/workflows/ci.yml`; the keyer manifest at each pin commit in the pin history table.
- Project files: `audit/report.md` (§3: PR #16, #24, #5, #30, #31 entries), `audit/req-keyer-box-fixes.md` (KB-5, KB-7, KB-9), `bench/quansheng.md` (stage 2 and its log, 16:00Z-16:19Z on 2026-10-09).
- Project memory notes: `MEMORY.md`, `rustos-template-cargo-error.md`, `keyer-unattended-pr.md`, `pico-decoder-prs.md`, `one-pico-keyer-audit.md`, `flash-files.md` (each checked against the code and the files above).
- The other pages of this handover: [01 Overview](01-overview.md), [02 Boot and runtime](02-boot-runtime.md), [04 Drivers: core](04-drivers-core.md), [05 Drivers: I/O](05-drivers-io.md), [06 Multicore](06-multicore.md), [09 Work left and known issues](09-work-left-known-issues.md).

[gw-08]: https://github.com/robinonsay/cw-message-gateway/blob/claude/handover-docs-us00cn/docs/handover/08-keyer-box-firmware.md
[gw-11]: https://github.com/robinonsay/cw-message-gateway/blob/claude/handover-docs-us00cn/docs/handover/11-decoder-box-one-pico.md
[gw-13]: https://github.com/robinonsay/cw-message-gateway/blob/claude/handover-docs-us00cn/docs/handover/13-safety-rules.md
[gw-14]: https://github.com/robinonsay/cw-message-gateway/blob/claude/handover-docs-us00cn/docs/handover/14-audit-bench-flashing.md
[gw-16]: https://github.com/robinonsay/cw-message-gateway/blob/claude/handover-docs-us00cn/docs/handover/16-work-left-known-bugs.md
[gw-17]: https://github.com/robinonsay/cw-message-gateway/blob/claude/handover-docs-us00cn/docs/handover/17-glossary.md
