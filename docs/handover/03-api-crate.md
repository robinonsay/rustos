# 03 The api crate

> `api` is rustos's portable crate: the traits, ownership types and pure logic that every driver and application is written against, with no register address in it and no dependency on any other crate. It holds four things. The first is the value-transfer traits `ErrorType`, `Read<T>` and `Write<T>`. The second is pin and peripheral ownership: `PinHandle<N>`, `DeviceHandle<T>` and the `define_board!` macro that creates each handle once per boot. The third is the GPIO traits. The fourth is the chip-independent half of a USB serial port: SETUP decoding, descriptors, the endpoint-0 request state machine, CDC-ACM line coding and a byte ring. rustos PR #6 (open, not on master) adds a fifth: the core-1 launch handshake, as a state machine that never blocks. Because nothing in it touches hardware, the whole crate builds and runs its tests on a desktop with `cargo test -p api`. It lives in `api/` at the repository root, and the RP2350 crate `firmware/pico2` implements its traits. Robin (the project owner) wrote `common`, `device` and `gpio`; this project added `api::usb` (rustos PR #1) and `api::multicore` (rustos PR #6). [08 Changes for cw-message-gateway](08-changes-for-cw-gateway.md) tells that history.

All code links are at rustos master `80bf599` unless marked "PR #6", which means rustos PR #6 at its head `410dffa` (open, not on master, untested on hardware). Links into cw-message-gateway are at its main `44185fa` unless marked PR #30 (`3efaf84`), PR #31 (`bc5062a`) or keyer build `eef8c126`.

**Terms used on this page.** The radio terms are in the cw-message-gateway [17 Glossary](https://github.com/robinonsay/cw-message-gateway/blob/claude/handover-docs-us00cn/docs/handover/17-glossary.md); [01 Overview](01-overview.md) introduces the chip.

| Term | Meaning here |
|---|---|
| trait, supertrait | A named set of method signatures a type opts into (an interface). `trait A: B` means every `A` must also be a `B`. |
| associated type, GAT | A type member of a trait that each implementation chooses (`type Error`). A generic associated type (GAT) takes parameters, here a pin number: `type Output<const N: usize>`. |
| zero-sized type | A struct with no data. It costs nothing at run time; it exists only for the compiler to track. |
| endpoint, IN, OUT | A numbered USB channel on the device. IN carries data to the host, OUT from it; bit 7 of an endpoint address is the direction (`0x82` = endpoint 2 IN). |
| control transfer, SETUP | The request/answer exchange on endpoint 0. The host opens each one with an 8-byte SETUP packet saying what it wants. |
| stage | A control transfer has a SETUP stage, an optional DATA stage, and a STATUS stage (a zero-length packet the other way). |
| ACK, NAK, STALL | The device's answers to the host: accepted; "not ready, ask again"; "not supported" or "halted". |
| ZLP | A zero-length packet. It ends a transfer whose last packet was exactly full. |
| DATA0, DATA1 (data toggle) | Every data packet carries one of two PIDs, alternating per endpoint, so a re-sent packet can be recognised as a duplicate. |
| descriptor, enumeration | The byte structures a device sends to describe itself, and the host's process of reading them, giving the device an address and choosing a configuration. |
| CDC-ACM | The USB class that desktop systems open as a serial port without a driver install (`/dev/ttyACM0`, `/dev/cu.usbmodem*`, a COM port). |
| line coding, DTR, RTS | The serial-port settings a host sends (baud rate, framing) and two modem-control lines. Over USB they are only stored values. |
| DPRAM | The RP2350 USB controller's 4 kB of RAM shared between the processor and the controller, where packet buffers live. |
| bootrom, FIFO, SEV | The chip's built-in boot code; the two small word queues between the two processor cores; the Arm instruction that wakes the other core. |
| SIO | The RP2350's single-cycle I/O block. It holds the GPIO registers and the inter-core FIFOs. |
| watchdog | A hardware timer that resets the chip unless software "feeds" it in time. |
| full speed | USB's 12 Mbit/s mode, the one this device uses. Control and bulk packets are at most 64 bytes. |
| VID, PID | The 16-bit vendor and product IDs a USB device reports. Hosts choose drivers and remember devices by the pair. |
| BCD | Binary-coded decimal: one decimal digit per 4 bits, so `0x0100` means 1.00. |

## Where it lives

| Path | What it holds | Lines (code and docs / tests) | Origin |
|---|---|---|---|
| `api/Cargo.toml` | Package `api`, edition 2024, an empty `[dependencies]` ([`api/Cargo.toml:1-8`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/Cargo.toml#L1-L8)). | 8 | Robin's original |
| `api/src/lib.rs` | `#![no_std]`, the crate docs, the module list ([`api/src/lib.rs:88-93`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/lib.rs#L88-L93)). PR #6 adds `pub mod multicore;` ([`api/src/lib.rs:99`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/lib.rs#L99)). | 93 (99 on PR #6) | Robin's original; `pub mod usb;` from PR #1, `pub mod multicore;` from PR #6 |
| `api/src/common/mod.rs` | `ErrorType`, `Write<T>`, `Read<T>`: the error model and value transfer. | 114 / 0 | Robin's original |
| `api/src/device/mod.rs` | `PinHandle<N>`, `DeviceHandle<T>`, `define_board!`. | 322 / 0 | Robin's original |
| `api/src/gpio/mod.rs` | `Pull`, `GpioPinIn<N>`, `GpioPinOut<N>`, `Gpio`. | 180 / 0 | Robin's original |
| `api/src/usb/mod.rs` | The USB module map and tour, `control_in_needs_zlp`, `bulk_in_needs_zlp`. | 154 / 26 | rustos PR #1 |
| `api/src/usb/setup.rs` | `SetupPacket` and its decoders; standard request, descriptor-type and feature codes. | 191 / 51 | rustos PR #1 |
| `api/src/usb/descriptor.rs` | Endpoint and interface numbers, `UsbDeviceConfig`, the device, configuration, language and string descriptors. | 249 / 116 | rustos PR #1 |
| `api/src/usb/cdc_acm.rs` | CDC class request codes, `LineCoding`, `StopBits`, `Parity`, `ControlLineState`. | 168 / 40 | rustos PR #1 |
| `api/src/usb/control.rs` | `ControlHandler`, the endpoint-0 state machine; `ControlAction`, `Effect`, `DeviceState`. | 494 / 309 | rustos PR #1 |
| `api/src/usb/ring.rs` | `ByteRing<N>`, the fixed-capacity byte queue. | 113 / 33 | rustos PR #1 |
| `api/src/multicore.rs` (PR #6) | `Mailbox`, `Launch`, `Step`, `QUIET_US`, `DRAIN_MAX`, and a model of core 1 for the tests. | 191 / 341 | rustos PR #6 (open) |

Robin's originals are her commits of 25 to 30 August 2026 (`4866bf4` to `2734898`). `api::usb` came in `c6fd4bf`, merged with rustos PR #1 as `cfc2b14`; that commit also added `pub mod usb;` to `lib.rs`.

The implementations live in the `pico2` crate: `firmware/pico2/src/common/board.rs` (the `define_board!` invocation), `gpio/gpio.rs` (the `Gpio` traits), `usb/mod.rs` and `usb/hw.rs` (the USB hardware half), `pwm.rs` (`ErrorType` on `Rp2350Pwm`; `ErrorType` and `Write<bool>` on `Rp2350PwmSquare<N>`), `adc.rs` (consumes a `PinHandle`), and on PR #6 `multicore.rs` (the `Mailbox`). The figure under "How it connects" draws every one of these links. [04 Core drivers](04-drivers-core.md), [05 I/O drivers](05-drivers-io.md) and [06 Multicore](06-multicore.md) describe them.

## How it works

### The portability boundary

`api` is depended on by everything and depends on nothing. The crate's own diagram draws `api` at the top, `pico2` below it ("implemented by") and the application below that ("linked against"), with the arrows pointing up at `api`. It says the arrow direction "is the whole point": `api` never depends on `pico2`, so a driver for another chip can implement the same traits ([`api/src/lib.rs:35-50`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/lib.rs#L35-L50)). One limit on "another chip": `take()` needs an atomic compare-and-swap, which some small cores lack (Known bugs). The crate has no dependencies ([`api/Cargo.toml:8`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/Cargo.toml#L8)) and is `#![no_std]` ([`api/src/lib.rs:88`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/lib.rs#L88)). It therefore compiles for the host as readily as for the RP2350's `thumbv8m.main-none-eabihf` target, "which is what makes application logic written against these traits testable without hardware" ([`api/src/lib.rs:29-33`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/lib.rs#L29-L33)). The workspace makes `api` its only default member, so a plain `cargo build` or `cargo test` at the root covers it on the host ([`Cargo.toml:3-7`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/Cargo.toml#L3-L7)).

Two conventions run through every trait ([`api/src/lib.rs:75-87`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/lib.rs#L75-L87)):

- **One error type per implementing type.** `ErrorType` declares it once, and every fallible trait takes `ErrorType` as a supertrait.
- **Traits are generic over the value, not the peripheral.** A GPIO pin is `Write<bool>`; a UART would be `Write<u8>`; a type that takes two representations implements both ([`api/src/common/mod.rs:64-68`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/common/mod.rs#L64-L68)).

### Ownership: handles and the board

Embedded C leaves "who owns pin 25" to convention. `api::device` turns it into a value ([`api/src/device/mod.rs:1-19`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L1-L19)):

- `PinHandle<N>` is a zero-sized value whose possession proves pin `N` exists on this board and is owned nowhere else. Its field is private and its only constructor `new` is `unsafe` ([`api/src/device/mod.rs:63-99`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L63-L99)).
- `DeviceHandle<T>` is the same claim for the one peripheral driven by driver type `T` ([`api/src/device/mod.rs:135-162`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L135-L162)).
- `define_board!` is invoked once by a chip-support crate with the board's pin map and peripherals. It generates a pins struct with one `pub` `PinHandle` field per pin, a board struct holding those pins and one `DeviceHandle` per device, a private `const fn new()` that makes every handle, and `pub fn take() -> Option<Self>` that hands the board out once per boot ([`api/src/device/mod.rs:239-322`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L239-L322)).

A driver constructor takes its `DeviceHandle` by value, and a pin-consuming constructor (GPIO, PWM or ADC) takes its `PinHandle` by value. Neither handle is `Copy` or `Clone`, so a second use is a compile error ("use of moved value"), not a runtime check ([`api/src/device/mod.rs:51-58`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L51-L58), [`api/src/device/mod.rs:132-134`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L132-L134)). The only run-time check in the scheme is the once-per-boot flag inside `take()`. [01 Overview](01-overview.md) draws this ownership model; this page gives the exact macro below.

### Values in and out: `ErrorType`, `Read`, `Write`

`Write<T>::write(&mut self, value: T)` sends a value; `Read<T>::read(&mut self)` reads one ([`api/src/common/mod.rs:74-114`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/common/mod.rs#L74-L114)). Both take `&mut self`. For `write`, that exclusive borrow plus handle uniqueness means two code paths cannot drive one pin ([`api/src/common/mod.rs:77-89`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/common/mod.rs#L77-L89)). For `read`, it lets an implementation consume state (a UART data register pops a byte), at the cost that pure observations also need exclusive access ([`api/src/common/mod.rs:98-112`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/common/mod.rs#L98-L112)). `Write` implementations "should be idempotent and non-blocking where the hardware allows" ([`api/src/common/mod.rs:70-73`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/common/mod.rs#L70-L73)).

The error model: `ErrorType::Error` must be `core::fmt::Debug`, nothing more, so `unwrap` can print it and a defmt-style logger can stream it, without forcing human-readable strings into flash ([`api/src/common/mod.rs:37-59`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/common/mod.rs#L37-L59)). Use `core::convert::Infallible` only for an operation that genuinely cannot fail; the docs warn against using it because failure is merely unlikely ([`api/src/common/mod.rs:51-58`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/common/mod.rs#L51-L58)). The RP2350 GPIO pins use `Infallible` ([`firmware/pico2/src/gpio/gpio.rs:283-291`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/gpio/gpio.rs#L283-L291)); the USB driver uses `UsbError` with `WouldBlock` and `NotConfigured` ([`firmware/pico2/src/usb/mod.rs:296-304`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L296-L304)). `api` defines no error kinds of its own. Whether an error means "try again" depends on each implementor's type (for example `UsbError::WouldBlock`), so code that is generic over `Read` or `Write` can only pass an error on or print it with `Debug`.

### GPIO

`Gpio` is a factory: it consumes a `PinHandle<N>` and returns a configured pin whose type carries `N` ([`api/src/gpio/mod.rs:165-180`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L165-L180)). `GpioPinIn<N>` and `GpioPinOut<N>` are marker traits over `Read<bool>` and `Write<bool>` ([`api/src/gpio/mod.rs:88`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L88), [`api/src/gpio/mod.rs:108`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L108)). `Pull` is a required argument for inputs, because a wrong or missing pull "does not fail loudly" ([`api/src/gpio/mod.rs:61-72`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L61-L72)). The whole path on the Pico 2 is `Rp2350::take()`, `Rp2350Gpio::new(board.gpio)`, `gpio.output_from_handle(board.pins.led)`, then `led.write(true)` ([`api/src/gpio/mod.rs:17-25`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L17-L25)).

### The USB serial port

A USB serial port splits into a hardware half and a protocol half, and `api::usb` is the protocol half: "pure computation on bytes", every byte pinned down by unit tests ([`api/src/usb/mod.rs:1-8`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/mod.rs#L1-L8)). `pico2::usb` is the hardware half: it "moves packets between the controller's dual-port RAM and that logic, and applies the `Effect`s it asks for" ([`firmware/pico2/src/usb/mod.rs:4-8`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L4-L8)).

![The USB serial port in layers, from the USB host at the top down through the RP2350 USB controller and DPRAM, the chip-specific pico2 driver and the portable api modules (enumeration, class requests, the two byte rings) to the application at the bottom](diagrams/03-usb-stack.svg)

*Figure: the two dashed frames are the crate boundary. Everything inside "portable" is reused unchanged by a driver for another chip; only the "chip-specific" frame is rewritten. The left column is endpoint 0, the right column the byte stream. The purple tag marks `room()`, which exists only on rustos PR #6.*

What happens, in order:

1. **Attach.** `Rp2350Usb::new` brings up the controller and turns on the pull-up; the host sees a full-speed device and resets the bus ([`firmware/pico2/src/usb/mod.rs:363-424`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L363-L424)). The driver calls `ControlHandler::bus_reset()` on every bus reset ([`firmware/pico2/src/usb/mod.rs:591-618`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L591-L618)).
2. **Enumeration on endpoint 0.** For each SETUP packet the driver copies the 8 bytes out of DPRAM, decodes them with `SetupPacket::from_bytes`, and asks `ControlHandler::setup` what to do ([`firmware/pico2/src/usb/mod.rs:889-908`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L889-L908)). The handler answers with one of four `ControlAction`s and, where the hardware must change, an `Effect`. The host typically reads the device descriptor, sets an address, reads the configuration descriptor and strings, and selects configuration 1 (the test helper `configured()` replays the address and configuration steps, [`api/src/usb/control.rs:514-528`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L514-L528)).
3. **Class requests.** Once the port is open, the host sends CDC-ACM requests on endpoint 0: SET_LINE_CODING, GET_LINE_CODING, SET_CONTROL_LINE_STATE, SEND_BREAK. The handler stores the values; nothing changes how bytes move ([`api/src/usb/cdc_acm.rs:1-16`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L1-L16)).
4. **The byte stream on endpoint 2.** In the Configured state, bytes from the host land in the `rx` ring and bytes the application writes wait in the `tx` ring. Both rings are `ByteRing<512>` fields of `Rp2350Usb` ([`firmware/pico2/src/usb/mod.rs:248-254`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L248-L254), [`firmware/pico2/src/usb/mod.rs:326-350`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L326-L350)).

### Starting core 1 (PR #6)

Out of every reset, core 1 waits in the bootrom for core 0 to send six words over the inter-core FIFO, `0, 0, 1, vector table, stack pointer, entry point`, each echoed back; any wrong echo starts the sequence again ([`api/src/multicore.rs:4-30`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L4-L30)). `api::multicore` implements core 0's side as `Launch`, a state machine over a `Mailbox` trait, whose `step` never blocks. That matters because the core running the launch may already hold a hardware watchdog ([`api/src/multicore.rs:42-52`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L42-L52)). It adds one rule to the datasheet's listing: before each `0`, the FIFO from core 1 must stay empty for `QUIET_US` (100 µs), or a late echo can leave core 0 one answer behind forever ([`api/src/multicore.rs:32-40`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L32-L40)). The handshake works only on a core 1 that has just been reset and is waiting in the bootrom. The module docs exclude "a core 1 that is already running something else, or is part way through a sequence an earlier, interrupted launch began" ([`api/src/multicore.rs:21-30`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L21-L30)). rustos has no way to reset core 1 on its own, so in practice this means one launch per boot, after a chip or watchdog reset. [06 Multicore](06-multicore.md) has the sequence diagram, the algorithm case by case, the resets and the RP2350 driver; this page gives the API below.

## How it connects

On endpoint 0 the driver and `ControlHandler` take turns: for each SETUP packet the handler decides the answer, and the driver runs the data and status stages on the hardware. The figure shows one whole control transfer, then the device states the transfers move through.

![The endpoint-0 control transfer: on each SETUP, ControlHandler::setup returns DataIn, DataOut, NoData or Stall, the driver runs the data and status stages, and the device moves between the Default, Address and Configured states](diagrams/03-control-transfer.svg)

*Figure: square boxes are steps the code takes, in `control.rs` or in the driver; rounded boxes are stages and states, with the driver's `Ep0Stage` name in brackets. Notice that a new address is written only after the status stage, and that a STALL lasts only until the next SETUP.*

### Who implements what

The crate's neighbours are the `pico2` modules that implement or consume each `api` module, and the applications outside rustos that name `api` items directly. The figure shows all of them; the tables below give the exact types and lines.

![What implements and uses each api module: in pico2, the GPIO, PWM and USB types implement the common and gpio traits, the board invokes define_board!, three constructors consume pin handles, every driver takes a device handle, Rp2350Usb owns the USB logic and, on PR #6, Fifo and Rp2350Core1 use api::multicore; outside rustos, the keyer box, decoder box, one-Pico image and template applications name api items directly, each pinned to a rustos revision](diagrams/03-dependents.svg)

*Figure: the left frame is `api`, the right frame `pico2`, and the bottom frame the applications; arrows point from the code that implements or uses an item to the `api` module that defines it. Purple dashed boxes are not merged yet: rustos PR #6, or an open cw-message-gateway PR. Notice that every application reaches `api` through its own pinned rustos revision, except one made from the template, which has no pin.*

| `api` item | Implemented or used by (pico2) | Where |
|---|---|---|
| `define_board!` | `Rp2350` / `Rp2350Pins`: `gpio0`-`gpio22`, `smps_ps` 23, `vbus_sense` 24, `led` 25, `gpio26`-`gpio28`, `vsys_adc` 29; nine devices on master (`gpio`, `clocks`, `timer`, `systick`, `watchdog`, `usb`, `pwm`, `dma`, `adc`); PR #6 adds `core1` | [`firmware/pico2/src/common/board.rs:97-146`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/board.rs#L97-L146), [`firmware/pico2/src/common/board.rs:147`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/common/board.rs#L147) |
| `Gpio` | `Rp2350Gpio`, `Input<N> = Rp2350GpioIn<N>`, `Output<N> = Rp2350GpioOut<N>`, `Error = GpioError` | [`firmware/pico2/src/gpio/gpio.rs:151-183`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/gpio/gpio.rs#L151-L183) |
| `GpioPinIn<N>`, `GpioPinOut<N>` | `Rp2350GpioIn<N>` (`Read<bool>`), `Rp2350GpioOut<N>` (`Write<bool>`), both `Error = Infallible`. The output pin offers no read-back: only `Rp2350GpioIn` implements `Read<bool>` ([`firmware/pico2/src/gpio/gpio.rs:321`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/gpio/gpio.rs#L321)) | [`firmware/pico2/src/gpio/gpio.rs:283-345`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/gpio/gpio.rs#L283-L345) |
| `ErrorType`, `Read<u8>`, `Write<u8>` | `Rp2350Usb`, `Error = UsbError`; byte at a time, beside its own slice methods `read(&mut [u8])` and `write(&[u8])` | [`firmware/pico2/src/usb/mod.rs:1061-1097`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L1061-L1097) |
| `ErrorType`, `Write<bool>` | `Rp2350Pwm` (`Error = PwmError`); `Rp2350PwmSquare<N>` (`Error = Infallible`; `write(true)` = 50 % square wave, `write(false)` = held low, effective at the next counter wrap). So a tone output can be driven by the same generic `Write<bool>` code as a GPIO pin. | [`firmware/pico2/src/pwm.rs:732`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L732), [`firmware/pico2/src/pwm.rs:770`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L770), [`firmware/pico2/src/pwm.rs:774-800`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L774-L800) |
| `PinHandle<N>` (consumed) | `Gpio::input_from_handle` and `output_from_handle` (above); `Rp2350Pwm::square_from_handle`; `Rp2350Adc::stream_from_handle`, where `N` = 26 to 29 is checked at build time | [`firmware/pico2/src/pwm.rs:653-657`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/pwm.rs#L653-L657), [`firmware/pico2/src/adc.rs:477-483`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L477-L483), [`firmware/pico2/src/adc.rs:579`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/adc.rs#L579) |
| `DeviceHandle<T>` (consumed) | the `new` of each of the nine drivers named in `define_board!` above, for example `Rp2350Gpio::new(_handle: DeviceHandle<Rp2350Gpio>)`; on PR #6 also `Rp2350Core1::new` | [`firmware/pico2/src/gpio/gpio.rs:132`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/gpio/gpio.rs#L132), [`firmware/pico2/src/usb/mod.rs:363-364`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L363-L364) |
| `usb::control`, `setup`, `descriptor`, `cdc_acm`, `ring`, ZLP functions | `Rp2350Usb` owns a `ControlHandler` and two `ByteRing<512>`; re-exports `UsbDeviceConfig` | [`firmware/pico2/src/usb/mod.rs:226-240`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L226-L240), [`firmware/pico2/src/usb/mod.rs:326-350`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L326-L350) |
| `multicore::Mailbox`, `Launch`, `Step` (PR #6) | `Fifo` implements `Mailbox` over the SIO FIFO registers; `Rp2350Core1::launch` steps a `Launch` until `Launched` or its time limit | [`firmware/pico2/src/multicore.rs:220-244`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L220-L244), [`firmware/pico2/src/multicore.rs:350-395`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/multicore.rs#L350-L395) |

### The driver's contract with `ControlHandler`

The handler never touches hardware; the driver runs every stage. The handler's own summary is at [`api/src/usb/control.rs:9-22`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L9-L22). In detail:

| `setup()` returns | What the driver does (pico2) | Where |
|---|---|---|
| `DataIn { len, zlp }` | Sends the first `len` bytes of `reply()` in packets of at most 64 bytes, DATA1 first and alternating, then a ZLP if `zlp`. It arms the zero-length OUT status (DATA1) together with the last IN packet, and goes Idle when the status completes. | [`firmware/pico2/src/usb/mod.rs:913-916`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L913-L916), [`firmware/pico2/src/usb/mod.rs:939-968`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L939-L968) |
| `DataOut { len }` | Only if `len <= 64`: the driver supports single-packet OUT data stages only, and stalls a larger `DataOut`. Arms one OUT packet (DATA1). When it completes, passes the bytes received (at most `len`) to `data_out()` and runs the action that returns. | [`firmware/pico2/src/usb/mod.rs:917-928`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L917-L928), [`firmware/pico2/src/usb/mod.rs:933`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L933), [`firmware/pico2/src/usb/mod.rs:1023-1029`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L1023-L1029) |
| `NoData(effect)` | Applies the effect now, except `SetAddress`, then arms a zero-length IN status (DATA1). When the status completes it writes a `SetAddress` address to `ADDR_ENDP`. | [`firmware/pico2/src/usb/mod.rs:929-932`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L929-L932), [`firmware/pico2/src/usb/mod.rs:999-1007`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L999-L1007) |
| `Stall` | Sets `EP_STALL_ARM` and the STALL bit in both EP0 buffer control words. The next SETUP ends it: the controller clears `EP_STALL_ARM`, and `handle_setup` rewrites both EP0 buffer control words to 0. | [`firmware/pico2/src/usb/mod.rs:983-991`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L983-L991), [`firmware/pico2/src/usb/mod.rs:160-162`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L160-L162), [`firmware/pico2/src/usb/mod.rs:898-903`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L898-L903) |

| `Effect` | Driver action (pico2 `apply_effect`) |
|---|---|
| `None` | Nothing. |
| `SetAddress(a)` | Nothing until the IN status completes, then `ADDR_ENDP = a` ([`firmware/pico2/src/usb/mod.rs:999-1006`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L999-L1006)). |
| `ConfigurationChanged { from, to }` | `from != 0`: disable EP1 IN, EP2 OUT and EP2 IN, empty both rings, count a `LinkEvent::Deconfigured`. `to != 0`: enable the three endpoints with DATA0 toggles and empty both rings again ([`firmware/pico2/src/usb/mod.rs:1039-1047`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L1039-L1047), [`firmware/pico2/src/usb/mod.rs:807-821`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L807-L821), [`firmware/pico2/src/usb/mod.rs:621-629`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L621-L629)). |
| `EndpointHalt { endpoint, .. }` | Abort the endpoint, write its buffer control word with or without STALL as `is_halted` says, reset its toggle to DATA0 ([`firmware/pico2/src/usb/mod.rs:852-883`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L852-L883)). |
| `InterfaceReset(i)` | `i = 0`: reset EP1 IN. `i = 1`: reset EP2 OUT and EP2 IN ([`firmware/pico2/src/usb/mod.rs:1049-1056`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L1049-L1056)). |

The driver also asks the handler two questions on every poll: `is_configured()` (no data endpoint is armed until it is true) and `is_halted(ep)` (a halted endpoint is not armed) ([`firmware/pico2/src/usb/mod.rs:725-771`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L725-L771)). `write` also asks `is_configured()` and takes nothing until it is true ([`firmware/pico2/src/usb/mod.rs:473-479`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L473-L479)).

### Who uses the crate outside rustos

The cw-message-gateway firmware images depend on `api` and `pico2` by git revision, so a change in rustos reaches them only when their `Cargo.toml` moves the pin:

| Image | rustos revision | `api` items used directly | Where |
|---|---|---|---|
| Keyer box (`firmware/pico2-keyer`), main | `8c91507` (same `api` and `pico2::usb` sources as `80bf599`) | `common::{Read, Write}`, `gpio::{Gpio, Pull}`; USB through `pico2::usb` with VID `0x2e8a`, PID `0x0009`, product `keyer_core::NAME` | [`firmware/pico2-keyer/Cargo.toml:30-31`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/Cargo.toml#L30-L31), [`firmware/pico2-keyer/src/main.rs:46-47`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L46-L47), [`firmware/pico2-keyer/src/main.rs:86-93`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L86-L93) |
| Decoder box (`firmware/pico2-decoder`), PR #30 (open) | `80bf599` | `gpio::Gpio` | [`firmware/pico2-decoder/Cargo.toml:24-25`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/Cargo.toml#L24-L25), [`firmware/pico2-decoder/src/main.rs:33`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/src/main.rs#L33) |
| One-Pico keyer image, PR #31 (draft) | `410dffa` (rustos PR #6) | as the keyer, plus `device::PinHandle`; `pico2::multicore` and `Rp2350Usb::room()` | [`firmware/pico2-keyer/Cargo.toml:30-31`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/Cargo.toml#L30-L31), [`firmware/pico2-keyer/src/core1.rs:48`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/core1.rs#L48), [`firmware/pico2-keyer/src/main.rs:198`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L198) |
| A new application made from `templates/pico2` (in rustos) | none: `api` and `pico2` by git with no `rev`, so it takes the default branch's head when its lock file is first made | whatever it uses; the template itself uses `common::Write` and `gpio::Gpio` | [`templates/pico2/Cargo.toml.liquid:10-12`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/Cargo.toml.liquid#L10-L12), [`templates/pico2/src/main.rs:42-52`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/src/main.rs#L42-L52) |

hfnode, the host program, recognises the keyer box "by its product name" ([`firmware/pico2-keyer/src/main.rs:83-85`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L83-L85)), which reaches the host as string descriptor 2 built by `api::usb::descriptor::string_descriptor`. The box that enumerated on a Mac as `/dev/cu.usbmodem11  USB 2E8A:0009 hfnode, PICO2-KEYER, 1` at 16:17Z on 2026-10-09 ran keyer build `eef8c126`, which pins rustos `c9852112` ([`firmware/pico2-keyer/Cargo.toml:27-28`](https://github.com/robinonsay/cw-message-gateway/blob/eef8c126484a86ccc5b4fcf8832eba04f2754b07/firmware/pico2-keyer/Cargo.toml#L27-L28) at `eef8c126`). `git diff c9852112 80bf599 -- api/ firmware/pico2/src/usb/` is empty, as is the diff from main's current pin `8c91507`, so that is the code on master. That macOS session (enumeration at 16:17Z, and a passing `hfnode keyer --config ~/k1-bench.toml check` over the port at 16:19Z, which moved bytes both ways; project files `bench/quansheng.md`) is the only hardware evidence for this USB stack. Enumeration on Linux and Windows is not recorded.

## Interfaces and APIs

Every public item, by module. Signatures are as written in the source. Derives, for callers that compare or copy values: every enum and plain-data struct in `api::usb` derives `Clone, Copy, Debug, PartialEq, Eq` (`ControlLineState` also `Default`), which is what lets a test write `assert_eq!` on a `ControlAction`. `ControlHandler` and `ByteRing` derive only `Clone, Debug`; `Pull`, `PinHandle` and `DeviceHandle` derive nothing; on PR #6 `Step` derives the same five as the `api::usb` enums and `Launch` only `Debug`.

### `api` (`lib.rs`)

| Item | Signature | Where |
|---|---|---|
| crate attribute | `#![no_std]` | [`api/src/lib.rs:88`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/lib.rs#L88) |
| modules | `pub mod common; pub mod gpio; pub mod device; pub mod usb;` | [`api/src/lib.rs:90-93`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/lib.rs#L90-L93) |
| `multicore` (PR #6) | `pub mod multicore;` | [`api/src/lib.rs:99`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/lib.rs#L99) |
| `define_board!` | `#[macro_export]`, so it is reached as `api::define_board!` | [`api/src/device/mod.rs:239-240`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L239-L240) |

### `api::common`

| Item | Signature | Notes | Where |
|---|---|---|---|
| `ErrorType` | `pub trait ErrorType { type Error: core::fmt::Debug; }` | The one error type of an implementing type; supertrait of every fallible trait. | [`api/src/common/mod.rs:48-60`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/common/mod.rs#L48-L60) |
| `Write<T>` | `pub trait Write<T>: ErrorType { fn write(&mut self, value: T) -> Result<(), Self::Error>; }` | Should be idempotent and non-blocking where the hardware allows. | [`api/src/common/mod.rs:74-91`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/common/mod.rs#L74-L91) |
| `Read<T>` | `pub trait Read<T>: ErrorType { fn read(&mut self) -> Result<T, Self::Error>; }` | `&mut self` even for pure observations. | [`api/src/common/mod.rs:96-114`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/common/mod.rs#L96-L114) |

A trait's methods are callable only where the trait is imported: an application needs `use api::common::Write;` (and `Read`, `api::gpio::Gpio`) or the calls fail with "method not found" ([`api/src/gpio/mod.rs:27-31`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L27-L31)). On `Rp2350Usb`, `usb.read(&mut buf)` names the inherent slice method; the trait method is `Read::read(&mut usb)` ([`firmware/pico2/src/usb/mod.rs:1061-1065`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L1061-L1065)).

### `api::device`

| Item | Signature | Notes | Where |
|---|---|---|---|
| `PinHandle<N>` | `pub struct PinHandle<const N: usize> { _private: () }` | Zero-sized, not `Copy`, not `Clone`, no derives. | [`api/src/device/mod.rs:63-66`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L63-L66) |
| `PinHandle::new` | `pub const unsafe fn new() -> Self` | Safety: pin `N` exists and no other `PinHandle<N>` is live. Intended caller: the code `define_board!` generates; otherwise only host tests or undescribed hardware. | [`api/src/device/mod.rs:81-98`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L81-L98) |
| `DeviceHandle<T>` | `pub struct DeviceHandle<T> { _private: PhantomData<T> }` | Zero-sized claim on the peripheral driven by `T`. | [`api/src/device/mod.rs:135-137`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L135-L137) |
| `DeviceHandle::new` | `pub const unsafe fn new() -> Self` | Safety: no other `DeviceHandle<T>` is live. | [`api/src/device/mod.rs:150-161`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L150-L161) |
| `define_board!` | see the grammar below | Generates the board singleton. | [`api/src/device/mod.rs:239-322`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L239-L322) |

**`define_board!` grammar** ([`api/src/device/mod.rs:241-249`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L241-L249)):

```text
define_board! {
    $board:ident {
        $pins:ident {
            $pin_name:ident : $n:literal ,+ [,]
        }
        [ devices {
            $dev_name:ident : $dev_ty:ty ,+ [,]
        } ]
    }
}
```

At least one pin is required (`+`); the `devices` section is optional, but if present it needs at least one entry.

**What it generates** in the invoking module ([`api/src/device/mod.rs:250-320`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L250-L320)):

| Generated item | Exact shape | Where |
|---|---|---|
| `BOARD_TAKEN` | `static BOARD_TAKEN:core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);` (spacing as in the source). Private, never cleared. | [`api/src/device/mod.rs:253`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L253) |
| `$pins` | `pub struct $pins { pub $pin_name: api::device::PinHandle<$n>, ... }` | [`api/src/device/mod.rs:258-260`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L258-L260) |
| `$board` | `pub struct $board { pub pins: $pins, pub $dev_name: api::device::DeviceHandle<$dev_ty>, ... }` | [`api/src/device/mod.rs:264-267`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L264-L267) |
| `$board::new` | `const fn new() -> Self`, private; one `unsafe` block creating every handle | [`api/src/device/mod.rs:275-280`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L275-L280) |
| `$board::take` | `pub fn take() -> Option<Self>`: `BOARD_TAKEN.compare_exchange(false, true, Acquire, Acquire).ok().map(\|_\| Self::new())` | [`api/src/device/mod.rs:314-319`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L314-L319) |

Two requirements on the invoking module, from the macro's "Known rough edges" ([`api/src/device/mod.rs:233-238`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L233-L238)): `use core::sync::atomic::Ordering::Acquire;` must be in scope, because the expansion names `Acquire` bare (`pico2` does this at [`firmware/pico2/src/common/board.rs:76-82`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/board.rs#L76-L82)); and only one invocation per module, because `BOARD_TAKEN` is a fixed name.

### `api::gpio`

| Item | Signature | Notes | Where |
|---|---|---|---|
| `Pull` | `pub enum Pull { Up, Down, None }` | No derives. `Up` for a switch to ground (pressed reads `false`), `Down` for a switch to VCC, `None` for an actively driven signal or an external resistor. | [`api/src/gpio/mod.rs:35-72`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L35-L72) |
| `GpioPinIn<N>` | `pub trait GpioPinIn<const N: usize>: Read<bool> {}` | Marker: an input on pin `N`. | [`api/src/gpio/mod.rs:88`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L88) |
| `GpioPinOut<N>` | `pub trait GpioPinOut<const N: usize>: Write<bool> {}` | Marker: a push-pull output on pin `N`. Read-back of the pad is optional, offered by also implementing `Read<bool>`. | [`api/src/gpio/mod.rs:102-108`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L102-L108) |
| `Gpio` | `pub trait Gpio: ErrorType` | One GPIO port, already out of reset. Its `Error` covers configuration failures. | [`api/src/gpio/mod.rs:165`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L165) |
| `Gpio::Input<N>` | `type Input<const N: usize>: GpioPinIn<N>;` | GAT: one pin type per pin number. | [`api/src/gpio/mod.rs:168`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L168) |
| `Gpio::Output<N>` | `type Output<const N: usize>: GpioPinOut<N>;` | | [`api/src/gpio/mod.rs:170`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L170) |
| `Gpio::input_from_handle` | `fn input_from_handle<const N: usize>(&mut self, handle: PinHandle<N>, pull: Pull) -> Result<Self::Input<N>, Self::Error>;` | Consumes the handle. | [`api/src/gpio/mod.rs:174`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L174) |
| `Gpio::output_from_handle` | `fn output_from_handle<const N: usize>(&mut self, handle: PinHandle<N>) -> Result<Self::Output<N>, Self::Error>;` | Consumes the handle. Initial level is the implementation's choice; the RP2350 drives low before enabling the output ([`firmware/pico2/src/gpio/gpio.rs:438-442`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/gpio/gpio.rs#L438-L442)). | [`api/src/gpio/mod.rs:175-179`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L175-L179) |

### `api::usb` (`usb/mod.rs`)

| Item | Signature | Rule | Where |
|---|---|---|---|
| modules | `pub mod setup; pub mod descriptor; pub mod cdc_acm; pub mod control; pub mod ring;` | | [`api/src/usb/mod.rs:98-102`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/mod.rs#L98-L102) |
| `control_in_needs_zlp` | `pub const fn control_in_needs_zlp(reply_len: usize, w_length: usize, max_packet: usize) -> bool` | `reply_len < w_length && reply_len.is_multiple_of(max_packet)` | [`api/src/usb/mod.rs:126-128`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/mod.rs#L126-L128) |
| `bulk_in_needs_zlp` | `pub const fn bulk_in_needs_zlp(last_packet_len: usize, max_packet: usize, more_queued: bool) -> bool` | `!more_queued && last_packet_len != 0 && last_packet_len == max_packet` | [`api/src/usb/mod.rs:151-153`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/mod.rs#L151-L153) |

### `api::usb::setup`

The SETUP packet, USB 2.0 §9.3, all fields little-endian ([`api/src/usb/setup.rs:4-13`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L4-L13)):

| Offset | Field | Size | `SetupPacket` field |
|---|---|---|---|
| 0 | `bmRequestType` | 1 | `request_type: u8` (bit 7 direction, bits 6:5 type, bits 4:0 recipient) |
| 1 | `bRequest` | 1 | `request: u8` |
| 2 | `wValue` | 2 | `value: u16` |
| 4 | `wIndex` | 2 | `index: u16` |
| 6 | `wLength` | 2 | `length: u16` (0 = no data stage) |

| Item | Signature | Notes | Where |
|---|---|---|---|
| `SetupPacket` | `pub struct SetupPacket { pub request_type: u8, pub request: u8, pub value: u16, pub index: u16, pub length: u16 }` | `Clone, Copy, Debug, PartialEq, Eq` | [`api/src/usb/setup.rs:17-34`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L17-L34) |
| `Direction` | `pub enum Direction { HostToDevice, DeviceToHost }` | bit 7; `HostToDevice` also for no data stage | [`api/src/usb/setup.rs:37-43`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L37-L43) |
| `RequestKind` | `pub enum RequestKind { Standard, Class, Vendor, Reserved }` | bits 6:5 = 0, 1, 2, 3 | [`api/src/usb/setup.rs:46-56`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L46-L56) |
| `Recipient` | `pub enum Recipient { Device, Interface, Endpoint, Other, Reserved(u8) }` | bits 4:0 = 0, 1, 2, 3, 4-31 | [`api/src/usb/setup.rs:60-72`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L60-L72) |
| `SetupPacket::from_bytes` | `pub const fn from_bytes(b: [u8; 8]) -> Self` | Decodes exactly as the hardware stored them. | [`api/src/usb/setup.rs:76-84`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L76-L84) |
| `direction` | `pub const fn direction(&self) -> Direction` | `request_type & 0x80` | [`api/src/usb/setup.rs:87-93`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L87-L93) |
| `kind` | `pub const fn kind(&self) -> RequestKind` | `(request_type >> 5) & 0x3` | [`api/src/usb/setup.rs:96-103`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L96-L103) |
| `recipient` | `pub const fn recipient(&self) -> Recipient` | `request_type & 0x1f` | [`api/src/usb/setup.rs:106-114`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L106-L114) |
| `descriptor_type` | `pub const fn descriptor_type(&self) -> u8` | high byte of `wValue` | [`api/src/usb/setup.rs:118-120`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L118-L120) |
| `descriptor_index` | `pub const fn descriptor_index(&self) -> u8` | low byte of `wValue` | [`api/src/usb/setup.rs:123-125`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L123-L125) |

Constants (all `pub const`):

| Module | Constants | Where |
|---|---|---|
| `setup::request` (`u8`) | `GET_STATUS` 0, `CLEAR_FEATURE` 1, `SET_FEATURE` 3, `SET_ADDRESS` 5, `GET_DESCRIPTOR` 6, `SET_DESCRIPTOR` 7, `GET_CONFIGURATION` 8, `SET_CONFIGURATION` 9, `GET_INTERFACE` 10, `SET_INTERFACE` 11, `SYNCH_FRAME` 12 | [`api/src/usb/setup.rs:130-153`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L130-L153) |
| `setup::descriptor_type` (`u8`) | `DEVICE` 1, `CONFIGURATION` 2, `STRING` 3, `INTERFACE` 4, `ENDPOINT` 5, `DEVICE_QUALIFIER` 6, `OTHER_SPEED_CONFIGURATION` 7, `INTERFACE_ASSOCIATION` 11, `CS_INTERFACE` `0x24` | [`api/src/usb/setup.rs:158-179`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L158-L179) |
| `setup::feature` (`u16`) | `ENDPOINT_HALT` 0, `DEVICE_REMOTE_WAKEUP` 1, `TEST_MODE` 2 | [`api/src/usb/setup.rs:183-190`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L183-L190) |

### `api::usb::descriptor`

| Constant | Type | Value | Meaning | Where |
|---|---|---|---|---|
| `EP0_MAX_PACKET` | `usize` | 64 | Endpoint 0 max packet size | [`api/src/usb/descriptor.rs:33`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L33) |
| `BULK_MAX_PACKET` | `usize` | 64 | EP2 OUT and IN max packet size | [`api/src/usb/descriptor.rs:37`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L37) |
| `NOTIFY_MAX_PACKET` | `usize` | 16 | EP1 IN max packet size; never sends | [`api/src/usb/descriptor.rs:43`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L43) |
| `EP_NOTIFY_IN` | `u8` | `0x81` | Interrupt IN, CDC notifications (never armed) | [`api/src/usb/descriptor.rs:46`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L46) |
| `EP_DATA_OUT` | `u8` | `0x02` | Bulk OUT, bytes from the host | [`api/src/usb/descriptor.rs:48`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L48) |
| `EP_DATA_IN` | `u8` | `0x82` | Bulk IN, bytes to the host | [`api/src/usb/descriptor.rs:50`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L50) |
| `INTERFACE_COMM` | `u8` | 0 | CDC communication interface | [`api/src/usb/descriptor.rs:53`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L53) |
| `INTERFACE_DATA` | `u8` | 1 | CDC data interface | [`api/src/usb/descriptor.rs:55`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L55) |
| `NUM_INTERFACES` | `u8` | 2 | | [`api/src/usb/descriptor.rs:57`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L57) |
| `CONFIGURATION_VALUE` | `u8` | 1 | `bConfigurationValue` of the one configuration | [`api/src/usb/descriptor.rs:61`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L61) |
| `STRING_MANUFACTURER`, `STRING_PRODUCT`, `STRING_SERIAL` | `u8` | 1, 2, 3 | String indices | [`api/src/usb/descriptor.rs:64-68`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L64-L68) |
| `MAX_STRING_UNITS` | `usize` | 126 | UTF-16 units per string (2 + 2 × 126 = 254 bytes) | [`api/src/usb/descriptor.rs:72`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L72) |
| `DEVICE_DESCRIPTOR_LEN` | `usize` | 18 | | [`api/src/usb/descriptor.rs:104`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L104) |
| `CONFIGURATION_DESCRIPTOR_LEN` | `usize` | 75 | | [`api/src/usb/descriptor.rs:145`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L145) |
| `LANGUAGE_ID_DESCRIPTOR` | `[u8; 4]` | `[4, 3, 0x09, 0x04]` | String 0: US English only | [`api/src/usb/descriptor.rs:214`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L214) |
| `CONFIGURATION_DESCRIPTOR` | `[u8; 75]` | see the byte map below | | [`api/src/usb/descriptor.rs:178-209`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L178-L209) |

| Item | Signature | Notes | Where |
|---|---|---|---|
| `UsbDeviceConfig` | `pub struct UsbDeviceConfig { pub vendor_id: u16, pub product_id: u16, pub device_release: u16, pub manufacturer: &'static str, pub product: &'static str, pub serial_number: &'static str }` | What an application chooses. `device_release` is BCD (`0x0100` = 1.00). Strings are truncated to 126 UTF-16 units. `Clone, Copy, Debug, PartialEq, Eq`. | [`api/src/usb/descriptor.rs:84-101`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L84-L101) |
| `device_descriptor` | `pub const fn device_descriptor(cfg: &UsbDeviceConfig) -> [u8; DEVICE_DESCRIPTOR_LEN]` | The 18 bytes below. | [`api/src/usb/descriptor.rs:124-142`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L124-L142) |
| `string_descriptor_len` | `pub fn string_descriptor_len(s: &str) -> usize` | `2 + 2 * min(utf16 units, 126)` | [`api/src/usb/descriptor.rs:218-220`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L218-L220) |
| `string_descriptor` | `pub fn string_descriptor(s: &str, out: &mut [u8]) -> usize` | Writes `bLength`, 3, then UTF-16LE units; returns bytes written; 0 if `out` is shorter than 2. | [`api/src/usb/descriptor.rs:231-248`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L231-L248) |

**Device descriptor** (USB 2.0 Table 9-8; [`api/src/usb/descriptor.rs:106-123`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L106-L123)):

| Offset | Field | Value |
|---|---|---|
| 0 | `bLength` | 18 |
| 1 | `bDescriptorType` | 1 |
| 2-3 | `bcdUSB` | `0x0200` (2.00; 2.01 would make Windows ask for a BOS descriptor) |
| 4-6 | class, subclass, protocol | `0xEF`, `0x02`, `0x01`: "this device uses Interface Association Descriptors" |
| 7 | `bMaxPacketSize0` | 64 |
| 8-9, 10-11, 12-13 | `idVendor`, `idProduct`, `bcdDevice` | from `UsbDeviceConfig`, little-endian |
| 14, 15, 16 | `iManufacturer`, `iProduct`, `iSerialNumber` | 1, 2, 3 |
| 17 | `bNumConfigurations` | 1 |

**Configuration descriptor**, 75 bytes, 11 descriptors in one blob (layout comment [`api/src/usb/descriptor.rs:154-170`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L154-L170), bytes [`api/src/usb/descriptor.rs:178-209`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L178-L209)). Why the IAD: CDC-ACM is two interfaces, and the Interface Association Descriptor tells Windows they are one function, so Windows 10 and later bind the in-box `usbser.sys` with no INF ([`api/src/usb/descriptor.rs:16-27`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L16-L27)).

| Offset | Len | Descriptor | Fields |
|---|---|---|---|
| 0 | 9 | Configuration | `wTotalLength` 75, `bNumInterfaces` 2, `bConfigurationValue` 1, `iConfiguration` 0, `bmAttributes` `0x80` (bus powered, no remote wakeup), `bMaxPower` 50 (100 mA) |
| 9 | 8 | Interface Association | `bFirstInterface` 0, `bInterfaceCount` 2, class/subclass/protocol `02/02/00`, `iFunction` 0 |
| 17 | 9 | Interface 0 (communication) | alternate 0, 1 endpoint, class `02` (CDC), subclass `02` (ACM), protocol `00` (no AT commands) |
| 26 | 5 | CDC Header (functional) | `bDescriptorSubtype` 0, `bcdCDC` `0x0110` |
| 31 | 5 | CDC Call Management | subtype 1, `bmCapabilities` 0 (no call management), `bDataInterface` 1 |
| 36 | 4 | CDC Abstract Control Management | subtype 2, `bmCapabilities` `0x02` (line coding, control line state, SERIAL_STATE; not SEND_BREAK) |
| 40 | 5 | CDC Union | subtype 6, control interface 0, subordinate interface 1 |
| 45 | 7 | Endpoint `0x81` | interrupt (`0x03`), 16 bytes, `bInterval` 16 |
| 52 | 9 | Interface 1 (data) | alternate 0, 2 endpoints, class `0x0A` (CDC data), subclass 0, protocol 0 |
| 61 | 7 | Endpoint `0x02` | bulk (`0x02`), 64 bytes |
| 68 | 7 | Endpoint `0x82` | bulk (`0x02`), 64 bytes |

The ACM capability `0x02` and the note that SEND_BREAK is "still accepted" although its bit is clear are at [`api/src/usb/descriptor.rs:172-177`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L172-L177).

### `api::usb::cdc_acm`

| Item | Signature / value | Notes | Where |
|---|---|---|---|
| `class_request::SEND_ENCAPSULATED_COMMAND` | `u8` `0x00` | Not supported (stalls) | [`api/src/usb/cdc_acm.rs:23`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L23) |
| `class_request::GET_ENCAPSULATED_RESPONSE` | `u8` `0x01` | Not supported (stalls) | [`api/src/usb/cdc_acm.rs:25`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L25) |
| `class_request::SET_LINE_CODING` | `u8` `0x20` | Host to device, 7 bytes | [`api/src/usb/cdc_acm.rs:28`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L28) |
| `class_request::GET_LINE_CODING` | `u8` `0x21` | Device to host, 7 bytes | [`api/src/usb/cdc_acm.rs:31`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L31) |
| `class_request::SET_CONTROL_LINE_STATE` | `u8` `0x22` | No data; DTR bit 0, RTS bit 1 of `wValue` | [`api/src/usb/cdc_acm.rs:34`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L34) |
| `class_request::SEND_BREAK` | `u8` `0x23` | No data; duration in ms in `wValue` (accepted, ignored) | [`api/src/usb/cdc_acm.rs:36`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L36) |
| `StopBits` | `#[repr(u8)] pub enum StopBits { One = 0, OnePointFive = 1, Two = 2 }` | | [`api/src/usb/cdc_acm.rs:40-49`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L40-L49) |
| `Parity` | `#[repr(u8)] pub enum Parity { None = 0, Odd = 1, Even = 2, Mark = 3, Space = 4 }` | | [`api/src/usb/cdc_acm.rs:52-65`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L52-L65) |
| `LineCoding` | `pub struct LineCoding { pub baud: u32, pub stop_bits: StopBits, pub parity: Parity, pub data_bits: u8 }` | `data_bits` 5, 6, 7, 8 or 16; the baud rate has no effect on USB throughput | [`api/src/usb/cdc_acm.rs:76-86`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L76-L86) |
| `LINE_CODING_LEN` | `usize` 7 | | [`api/src/usb/cdc_acm.rs:89`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L89) |
| `LineCoding::DEFAULT` | 115200 baud, 1 stop bit, no parity, 8 data bits | What GET_LINE_CODING reports until a SET | [`api/src/usb/cdc_acm.rs:94-99`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L94-L99) |
| `LineCoding::from_bytes` | `pub fn from_bytes(b: &[u8]) -> Option<Self>` | `None` unless exactly 7 bytes with defined enum values | [`api/src/usb/cdc_acm.rs:104-132`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L104-L132) |
| `LineCoding::to_bytes` | `pub const fn to_bytes(&self) -> [u8; LINE_CODING_LEN]` | | [`api/src/usb/cdc_acm.rs:135-138`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L135-L138) |
| `impl Default for LineCoding` | `Self::DEFAULT` | | [`api/src/usb/cdc_acm.rs:141-145`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L141-L145) |
| `ControlLineState` | `pub struct ControlLineState { pub dtr: bool, pub rts: bool }` | `Default` = both low | [`api/src/usb/cdc_acm.rs:150-156`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L150-L156) |
| `ControlLineState::from_value` | `pub const fn from_value(value: u16) -> Self` | Bits above 1 ignored | [`api/src/usb/cdc_acm.rs:161-166`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L161-L166) |

Line coding on the wire (PSTN 1.2 §6.3.11; [`api/src/usb/cdc_acm.rs:67-75`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L67-L75)): bytes 0-3 `dwDTERate` little-endian, byte 4 `bCharFormat` (`StopBits`), byte 5 `bParityType` (`Parity`), byte 6 `bDataBits`. The default encodes as `00 C2 01 00 00 00 08` ([`api/src/usb/cdc_acm.rs:185-189`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L185-L189)). DTR is reported, never required: the driver never gates data on it ([`api/src/usb/cdc_acm.rs:11-16`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L11-L16)).

### `api::usb::control`

| Item | Signature | Notes | Where |
|---|---|---|---|
| `REPLY_CAPACITY` | `pub const REPLY_CAPACITY: usize = 256;` | Largest reply is a 254-byte string descriptor | [`api/src/usb/control.rs:56-59`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L56-L59) |
| `DeviceState` | `pub enum DeviceState { Default, Address, Configured }` | USB 2.0 §9.1.1; Attached, Powered and Suspended are the driver's | [`api/src/usb/control.rs:61-71`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L61-L71) |
| `ControlAction` | `pub enum ControlAction { DataIn { len: usize, zlp: bool }, DataOut { len: usize }, NoData(Effect), Stall }` | What the driver must do; see the contract table above | [`api/src/usb/control.rs:74-102`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L74-L102) |
| `Effect` | `pub enum Effect { None, SetAddress(u8), ConfigurationChanged { from: u8, to: u8 }, EndpointHalt { endpoint: u8, halted: bool }, InterfaceReset(u8) }` | A hardware change the request needs | [`api/src/usb/control.rs:105-142`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L105-L142) |
| `ControlHandler` | `pub struct ControlHandler { /* private */ }` | `Clone, Debug`. Holds the config, state, configuration value, halt bits, line coding, control lines, pending OUT and a 256-byte reply buffer. | [`api/src/usb/control.rs:153-165`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L153-L165) |
| `new` | `pub const fn new(config: UsbDeviceConfig) -> Self` | Default state, `LineCoding::DEFAULT`, DTR and RTS low | [`api/src/usb/control.rs:181-193`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L181-L193) |
| `config` | `pub const fn config(&self) -> &UsbDeviceConfig` | | [`api/src/usb/control.rs:196-198`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L196-L198) |
| `state` | `pub const fn state(&self) -> DeviceState` | | [`api/src/usb/control.rs:201-203`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L201-L203) |
| `configuration` | `pub const fn configuration(&self) -> u8` | 0 or 1 | [`api/src/usb/control.rs:206-208`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L206-L208) |
| `is_configured` | `pub const fn is_configured(&self) -> bool` | "may bytes flow?" | [`api/src/usb/control.rs:211-213`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L211-L213) |
| `is_halted` | `pub const fn is_halted(&self, endpoint: u8) -> bool` | false for EP0 and unknown addresses | [`api/src/usb/control.rs:218-223`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L218-L223) |
| `line_coding` | `pub const fn line_coding(&self) -> LineCoding` | | [`api/src/usb/control.rs:226-228`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L226-L228) |
| `control_line_state` | `pub const fn control_line_state(&self) -> ControlLineState` | | [`api/src/usb/control.rs:231-233`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L231-L233) |
| `reply` | `pub fn reply(&self) -> &[u8]` | Valid after a `setup` that returned `DataIn`; exactly `len` bytes | [`api/src/usb/control.rs:235-239`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L235-L239) |
| `bus_reset` | `pub fn bus_reset(&mut self)` | Back to Default: configuration 0, no halts, line coding back to `LineCoding::DEFAULT`, DTR and RTS low, pending request and reply forgotten; the config is kept. The handler holds no address; the driver resets `ADDR_ENDP`. | [`api/src/usb/control.rs:241-254`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L241-L254) |
| `setup` | `pub fn setup(&mut self, pkt: &SetupPacket) -> ControlAction` | Forgets any pending OUT first; dispatches on the request kind | [`api/src/usb/control.rs:256-269`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L256-L269) |
| `data_out` | `pub fn data_out(&mut self, data: &[u8]) -> ControlAction` | `NoData(Effect::None)` or `Stall` | [`api/src/usb/control.rs:271-289`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L271-L289) |

### `api::usb::ring`

`ByteRing<const N: usize>` ([`api/src/usb/ring.rs:10-25`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L10-L25)): a FIFO of up to `N` bytes stored inline, `Clone, Debug`, "not synchronised: it is meant to be owned by one driver and touched from one execution context" ([`api/src/usb/ring.rs:18-19`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L18-L19)).

| Method | Signature | Returns | Where |
|---|---|---|---|
| `new` | `pub const fn new() -> Self` | an empty ring | [`api/src/usb/ring.rs:29-31`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L29-L31) |
| `capacity` | `pub const fn capacity(&self) -> usize` | `N` | [`api/src/usb/ring.rs:34-36`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L34-L36) |
| `len` | `pub const fn len(&self) -> usize` | bytes stored | [`api/src/usb/ring.rs:39-41`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L39-L41) |
| `is_empty` | `pub const fn is_empty(&self) -> bool` | | [`api/src/usb/ring.rs:44-46`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L44-L46) |
| `free` | `pub const fn free(&self) -> usize` | `N - len` | [`api/src/usb/ring.rs:49-51`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L49-L51) |
| `clear` | `pub fn clear(&mut self)` | | [`api/src/usb/ring.rs:54-57`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L54-L57) |
| `push` | `pub fn push(&mut self, b: u8) -> bool` | `false`, storing nothing, if full | [`api/src/usb/ring.rs:60-68`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L60-L68) |
| `pop` | `pub fn pop(&mut self) -> Option<u8>` | oldest byte or `None` | [`api/src/usb/ring.rs:71-79`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L71-L79) |
| `push_slice` | `pub fn push_slice(&mut self, data: &[u8]) -> usize` | bytes taken (a prefix of `data`) | [`api/src/usb/ring.rs:82-91`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L82-L91) |
| `pop_slice` | `pub fn pop_slice(&mut self, out: &mut [u8]) -> usize` | bytes moved into the front of `out` | [`api/src/usb/ring.rs:95-105`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L95-L105) |
| `impl Default` | `Self::new()` | | [`api/src/usb/ring.rs:108-112`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L108-L112) |

### `api::multicore` (PR #6, open)

| Item | Signature | Notes | Where |
|---|---|---|---|
| `Mailbox` | `pub trait Mailbox { fn try_send(&mut self, word: u32) -> bool; fn try_receive(&mut self) -> Option<u32>; fn wake(&mut self); }` | One core's end of the two FIFOs. Every method returns at once. A word sent also wakes the other core. | [`api/src/multicore.rs:54-73`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L54-L73) |
| `QUIET_US` | `pub const QUIET_US: u64 = 100;` | µs the FIFO from core 1 must stay empty before each `0` | [`api/src/multicore.rs:75-79`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L75-L79) |
| `DRAIN_MAX` | `pub const DRAIN_MAX: usize = 16;` | Most words one step drains before a `0` | [`api/src/multicore.rs:81-85`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L81-L85) |
| `Step` | `pub enum Step { Pending, Launched }` | `Clone, Copy, Debug, PartialEq, Eq` | [`api/src/multicore.rs:87-94`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L87-L94) |
| `Launch` | `pub struct Launch { /* words: [u32; 6], next, sent, quiet_from */ }` | `Debug` only | [`api/src/multicore.rs:105-116`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L105-L116) |
| `Launch::new` | `pub const fn new(vector_table: u32, stack_pointer: u32, entry: u32) -> Self` | Words `[0, 0, 1, vector_table, stack_pointer, entry]`; `entry` must have the Thumb bit (bit 0) set, as a Rust function pointer does. Use only on a core 1 just reset and waiting in the bootrom ([`api/src/multicore.rs:21-30`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L21-L30)); in rustos that means one launch per boot. | [`api/src/multicore.rs:118-131`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L118-L131) |
| `Launch::step` | `pub fn step(&mut self, mailbox: &mut impl Mailbox, now: u64) -> Step` | `now` in µs from any clock that counts up; never blocks; `Launched` on the last echo and on every later call. On a core 1 that is running other code, or is part way through an earlier launch, it may never reach `Launched`, so the caller must bound it in time. | [`api/src/multicore.rs:133-189`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L133-L189) |

## Algorithms and processing

### `take()`: once per boot

`take()` is a single atomic compare-and-swap of `BOARD_TAKEN` from `false` to `true` ([`api/src/device/mod.rs:314-319`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L314-L319)). A load followed by a store would leave a window in which two callers (the two cores, or an interrupt between load and store) both read `false`; the compare-exchange guarantees exactly one caller sees `false` ([`api/src/device/mod.rs:289-298`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L289-L298)). `Acquire` on both outcomes stops later memory operations being reordered before the claim. The once-only property comes from the atomicity, not the ordering, and would hold under `Relaxed` ([`api/src/device/mod.rs:300-313`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L300-L313)). The flag is never cleared, so the board can be taken once per boot, not once per `main`.

### Decoding a SETUP packet

`from_bytes` copies byte 0 to `request_type` and byte 1 to `request`, and reads bytes 2-3, 4-5 and 6-7 as little-endian `u16`s ([`api/src/usb/setup.rs:76-84`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L76-L84)). Nothing is validated at this point; `ControlHandler` checks each field where the request needs it. A worked example: what every host sends first, `80 06 00 01 00 00 40 00`, decodes as device-to-host, standard, device, `GET_DESCRIPTOR`, descriptor type 1 (DEVICE) index 0, `wIndex` 0, `wLength` 64 ([`api/src/usb/setup.rs:196-208`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L196-L208)).

### The endpoint-0 request dispatch

`setup()` first clears `pending_out` and the reply length, because a new SETUP aborts any transfer in progress (USB 2.0 §8.5.3). It then dispatches on the request type: Standard to `standard()`, Class to `class()`, Vendor and Reserved to `Stall` ([`api/src/usb/control.rs:261-269`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L261-L269)). This table lists every request the handler accepts and its exact conditions; everything not listed answers `Stall`. `bmRequestType` values are the usual ones; the code matches on the decoded direction, recipient and `bRequest`.

| `bmRequestType` | Request | Accepted when | Answer | Where |
|---|---|---|---|---|
| `0x80` | GET_DESCRIPTOR, DEVICE, index 0 | any state | `DataIn`: the 18-byte device descriptor | [`api/src/usb/control.rs:353-356`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L353-L356) |
| `0x80` | GET_DESCRIPTOR, CONFIGURATION, index 0 | any state | `DataIn`: the 75-byte blob | [`api/src/usb/control.rs:357`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L357) |
| `0x80` | GET_DESCRIPTOR, STRING, index 0 | any state | `DataIn`: `04 03 09 04` | [`api/src/usb/control.rs:358`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L358) |
| `0x80` | GET_DESCRIPTOR, STRING, index 1, 2 or 3 | any state, any `wIndex` (one language) | `DataIn`: manufacturer, product or serial as UTF-16LE | [`api/src/usb/control.rs:359-370`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L359-L370) |
| `0x80` | GET_DESCRIPTOR, anything else | | `Stall`: string index 4 and above, including `0xEE` (the Microsoft OS string); DEVICE or CONFIGURATION with an index other than 0; device qualifier, other-speed, BOS and every other type | [`api/src/usb/control.rs:366`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L366) (strings; test [`api/src/usb/control.rs:575-577`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L575-L577)), [`api/src/usb/control.rs:371-374`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L371-L374) |
| `0x80` | GET_STATUS, device | any state | `DataIn`: `00 00` (bus powered, no remote wakeup) | [`api/src/usb/control.rs:381-383`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L381-L383) |
| `0x81` | GET_STATUS, interface | Configured, `wIndex` < 2 | `DataIn`: `00 00` | [`api/src/usb/control.rs:384-390`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L384-L390) |
| `0x82` | GET_STATUS, endpoint | `wIndex` ≤ `0xff`; EP0 (`0x00` or `0x80`) in any state; `0x81`, `0x02`, `0x82` only when Configured | `DataIn`: `[halted, 0]` | [`api/src/usb/control.rs:391-403`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L391-L403) |
| `0x80` | GET_CONFIGURATION | any state | `DataIn`: one byte, 0 or 1 | [`api/src/usb/control.rs:323-326`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L323-L326) |
| `0x81` | GET_INTERFACE | Configured, `wIndex` < 2 | `DataIn`: `00` | [`api/src/usb/control.rs:327-333`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L327-L333) |
| any OUT standard | (any) with `wLength` ≠ 0 | never | `Stall` | [`api/src/usb/control.rs:334`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L334) |
| `0x00` | SET_ADDRESS | `wValue` ≤ 127, `wIndex` 0, not Configured | `NoData(SetAddress(a))`; state Address, or Default if a = 0 | [`api/src/usb/control.rs:410-417`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L410-L417) |
| `0x00` | SET_CONFIGURATION | `wIndex` 0, not Default, `wValue` 0 or 1 | `NoData(ConfigurationChanged { from, to })`; clears all halts; state Configured or Address | [`api/src/usb/control.rs:421-435`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L421-L435) |
| `0x01` | SET_INTERFACE | Configured, `wIndex` < 2, `wValue` 0 | `NoData(InterfaceReset(wIndex))` | [`api/src/usb/control.rs:337-343`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L337-L343) |
| `0x02` | SET_FEATURE / CLEAR_FEATURE (ENDPOINT_HALT) | `wIndex` ≤ `0xff`; EP0 in any state; `0x81`, `0x02`, `0x82` when Configured | EP0: `NoData(None)` (accepted, ignored). Others: set or clear the halt bit, `NoData(EndpointHalt { endpoint, halted })`. CLEAR always reports the toggle reset, halted or not. | [`api/src/usb/control.rs:438-458`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L438-L458) |
| `0x00`, `0x01` | SET_FEATURE / CLEAR_FEATURE, device or interface | never | `Stall` (remote wakeup not advertised, test mode is high-speed only) | [`api/src/usb/control.rs:459-461`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L459-L461) |
| `0x21` | SET_LINE_CODING | interface 0, `wLength` exactly 7 | `DataOut { len: 7 }`; `pending_out = LineCoding` | [`api/src/usb/control.rs:472-478`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L472-L478) |
| `0xA1` | GET_LINE_CODING | interface 0 | `DataIn`: 7 bytes | [`api/src/usb/control.rs:479-482`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L479-L482) |
| `0x21` | SET_CONTROL_LINE_STATE | interface 0, `wLength` 0 | `NoData(None)`; DTR and RTS stored | [`api/src/usb/control.rs:483-486`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L483-L486) |
| `0x21` | SEND_BREAK | interface 0, `wLength` 0 | `NoData(None)`; nothing else | [`api/src/usb/control.rs:487-489`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L487-L489) |
| class, any other | | | `Stall`; also any class request not to interface 0 | [`api/src/usb/control.rs:467-470`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L467-L470), [`api/src/usb/control.rs:490`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L490) |
| `0x40`, `0xC0`, `0x60`, `0xE0` | vendor or reserved | never | `Stall` | [`api/src/usb/control.rs:267`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L267) |

The halt bits are one byte: bit 0 `0x81`, bit 1 `0x02`, bit 2 `0x82` ([`api/src/usb/control.rs:167-176`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L167-L176)). SET_CONFIGURATION clears all of them ([`api/src/usb/control.rs:432`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L432)).

**Building a reply.** `reply_with` copies the bytes into the 256-byte buffer, then `finish_reply` clips to `wLength` ([`api/src/usb/control.rs:293-315`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L293-L315)). A device-to-host request with `wLength` 0 has no data stage, so it becomes `NoData(Effect::None)` with an empty reply ([`api/src/usb/control.rs:304-309`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L304-L309)). Otherwise the answer is `DataIn { len, zlp }` with `zlp = control_in_needs_zlp(len, wLength, 64)`. String descriptors are built straight into the reply buffer ([`api/src/usb/control.rs:368-369`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L368-L369)).

**The OUT data stage.** Only SET_LINE_CODING has one. `data_out` takes `pending_out` and resets it, so a second call, or a call after a new SETUP, answers `Stall` ([`api/src/usb/control.rs:276-289`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L276-L289), test [`api/src/usb/control.rs:762-769`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L762-L769)). Invalid line coding answers `Stall` and keeps the old value ([`api/src/usb/control.rs:739-742`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L739-L742)).

### Device states

Part 2 of the control-transfer figure under "How it connects" draws these transitions.

`Default` after `new()` or `bus_reset()`. `SET_ADDRESS(a)` with a = 1..127 moves Default or Address to Address, and with a = 0 back to Default; it stalls while Configured ([`api/src/usb/control.rs:410-417`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L410-L417)). `SET_CONFIGURATION(1)` moves Address to Configured, `SET_CONFIGURATION(0)` moves Configured to Address, and `SET_CONFIGURATION(1)` while Configured is reported as `from: 1, to: 1`, which the driver treats as a new session ([`api/src/usb/control.rs:421-435`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L421-L435), [`firmware/pico2/src/usb/mod.rs:1039-1047`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L1039-L1047)). Note that the handler changes `state` when it answers SET_ADDRESS, while the hardware takes the address only after the status stage. `state()` therefore says Address slightly before the controller does.

### The zero-length-packet rules

USB has no end-of-message marker: a transfer ends on a packet shorter than the maximum, or when the receiver has the bytes it asked for ([`api/src/usb/mod.rs:55-63`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/mod.rs#L55-L63)).

- **Control IN** (`control_in_needs_zlp`): a ZLP is needed when the reply is shorter than `wLength` and a whole number of packets, including zero. An 18-byte reply to `wLength` 64 needs none; 64 bytes to `wLength` 255 needs one; 64 bytes to `wLength` 64 needs none (the host stops on the count); 0 bytes to `wLength` 7 needs one (the ZLP is the reply) ([`api/src/usb/mod.rs:104-128`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/mod.rs#L104-L128)). A 31-character product string makes a 64-byte descriptor and does need the ZLP ([`api/src/usb/control.rs:580-591`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L580-L591)).
- **Bulk IN** (`bulk_in_needs_zlp`): after a full 64-byte packet with nothing queued behind it, send one ZLP so the host's read completes now rather than at the next byte ([`api/src/usb/mod.rs:130-153`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/mod.rs#L130-L153)). The driver asks this after every completed EP2 IN packet ([`firmware/pico2/src/usb/mod.rs:712-719`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L712-L719)).

### Descriptor encoding

`device_descriptor` is a `const fn` that splices the three `u16`s from the config into a fixed 18-byte template ([`api/src/usb/descriptor.rs:124-142`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L124-L142)). `string_descriptor` starts at offset 2 and writes one UTF-16 unit at a time, little-endian. It stops at 126 units or when the next unit would not fit in `out`, then writes `bLength` and type 3 ([`api/src/usb/descriptor.rs:231-248`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L231-L248)). Characters outside the Basic Multilingual Plane become surrogate pairs; a truncation never splits a unit but can split a pair ([`api/src/usb/descriptor.rs:225-230`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L225-L230)). "Pico" encodes as `0A 03 50 00 69 00 63 00 6F 00` ([`api/src/usb/descriptor.rs:317-324`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L317-L324)).

### Line coding

`from_bytes` rejects anything but exactly 7 bytes, a `bCharFormat` above 2, a `bParityType` above 4, or `bDataBits` other than 5, 6, 7, 8 or 16 ([`api/src/usb/cdc_acm.rs:104-132`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L104-L132)). `to_bytes` is the inverse ([`api/src/usb/cdc_acm.rs:135-138`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L135-L138)). The values never change how bytes move: "a USB serial port runs at USB speed whatever baud rate the host 'sets'" ([`api/src/usb/cdc_acm.rs:3-9`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L3-L9)).

### The byte ring

`ByteRing` keeps `head` (index of the oldest byte) and `len` rather than a tail index, so full (`len == N`) and empty (`len == 0`) differ without wasting a slot ([`api/src/usb/ring.rs:12-16`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L12-L16)). `push` writes at `(head + len) % N` and increments `len`; `pop` reads at `head`, then advances `head` modulo `N` ([`api/src/usb/ring.rs:60-79`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L60-L79)). The slice methods loop over `push` and `pop` one byte at a time and stop at the first refusal, so they never block and report a short count instead ([`api/src/usb/ring.rs:82-105`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L82-L105)). In the driver, EP2 OUT is armed only when at least 64 bytes are free, so a received packet always fits and the host is NAKed rather than losing data ([`firmware/pico2/src/usb/mod.rs:248-251`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L248-L251), [`firmware/pico2/src/usb/mod.rs:725-741`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L725-L741)).

### `Launch::step` (PR #6)

`pub fn step(&mut self, mailbox: &mut impl Mailbox, now: u64) -> Step` ([`api/src/multicore.rs:133-189`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L133-L189)) does one bounded piece of work per call, at most `DRAIN_MAX` (16) receives and one send, so it never blocks and the caller bounds the whole launch with its own clock. Before each `0` it drains the FIFO from core 1 and waits until that FIFO has stayed empty for `QUIET_US` (100 µs), so a launch on a fresh core 1 takes about `2 × QUIET_US` in the host model; a matching echo moves on to the next word and a wrong one starts over at the first `0`. [06 Multicore (rustos PR #6)](06-multicore.md) walks through each case of `step`, why the quiet wait is there, and the host model of core 1 with its nine tests.

## Programming against it

The snippets below were written for this page and not compiled; the patterns they follow are cited.

### Write code against the traits, not the chip

```rust
use api::common::Write;
use api::gpio::GpioPinOut;

/// One pulse on any output. Works on any chip whose driver implements the traits.
fn pulse<P: Write<bool>>(pin: &mut P) -> Result<(), P::Error> {
    pin.write(true)?;
    pin.write(false)
}

/// Demands an output on pin 25 specifically: the pin number is in the type.
fn led_on(led: &mut impl GpioPinOut<25>) {
    let _ = led.write(true);
}
```

`P::Error` is unambiguous because `Write` takes its error from the `ErrorType` supertrait ([`api/src/common/mod.rs:29-34`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/common/mod.rs#L29-L34)). The concrete types appear in one place only, as in the template application: `Rp2350::take()`, `Rp2350Gpio::new(board.gpio)`, `gpio.output_from_handle(board.pins.led)` ([`templates/pico2/src/main.rs:42-52`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/templates/pico2/src/main.rs#L42-L52)).

A host-side fake for a test of such code:

```rust
use api::common::{ErrorType, Write};

struct FakePin { level: bool, writes: u32 }
impl ErrorType for FakePin { type Error = core::convert::Infallible; }
impl Write<bool> for FakePin {
    fn write(&mut self, v: bool) -> Result<(), Self::Error> {
        self.level = v;
        self.writes += 1;
        Ok(())
    }
}
```

A host test that needs a real handle makes one with `unsafe { PinHandle::<25>::new() }`. The `Safety` docs allow a direct call "only in host-side tests or on hardware not (yet) described by a `define_board!` invocation" ([`api/src/device/mod.rs:90-94`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L90-L94)).

The input side reads through `Read<bool>`:

```rust
use api::common::Read;
use api::gpio::GpioPinIn;

/// A push button on pin 2, wired from the pin to ground.
fn pressed<P: GpioPinIn<2>>(button: &mut P) -> Result<bool, P::Error> {
    Ok(!button.read()?) // the pull-up holds it high; pressed pulls it low
}
```

The pin comes from `gpio.input_from_handle(board.pins.gpio2, Pull::Up)`. `Pull::Up` is the choice for a switch to ground, so a pressed button reads `false` ([`api/src/gpio/mod.rs:35-72`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L35-L72)).

### Declare a board

```rust
mod board {
    // A module of its own: nothing else here can call the private MyBoard::new().
    use core::sync::atomic::Ordering::Acquire; // the expansion names Acquire bare
    use api::define_board;
    use crate::my_gpio::MyGpio;

    define_board! {
        MyBoard {
            MyPins {
                button: 2,
                led: 13,
            }
            devices {
                gpio: MyGpio,
            }
        }
    }
}

// In main, once:
let board = board::MyBoard::take().unwrap(); // None on any second call
let gpio = MyGpio::new(board.gpio);           // MyGpio::new(_: DeviceHandle<MyGpio>)
```

`pico2`'s own invocation is the model: `firmware/pico2/src/common/board.rs` holds the imports and the invocation and nothing else ([`firmware/pico2/src/common/board.rs:76-146`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/common/board.rs#L76-L146)). Keep it that way. The generated `new()` is private but safe, so any other code in the invoking module could call it and duplicate every handle (Known bugs). List only pins that exist; the macro does not check for duplicates (also Known bugs).

### Drive the USB logic from a new chip's driver

A driver for another USB controller reuses `api::usb` whole and writes only the hardware half. The skeleton, following `pico2::usb`:

```rust
use api::usb::control::{ControlAction, ControlHandler, Effect};
use api::usb::descriptor::{UsbDeviceConfig, EP0_MAX_PACKET};
use api::usb::setup::SetupPacket;

let mut ctl = ControlHandler::new(UsbDeviceConfig {
    vendor_id: 0x2E8A, product_id: 0x000A, device_release: 0x0100,
    manufacturer: "Maker", product: "Thing", serial_number: "0001",
});

// On a bus reset: ctl.bus_reset(); forget both data endpoints.
// On a SETUP packet (8 bytes from the controller):
match ctl.setup(&SetupPacket::from_bytes(setup_bytes)) {
    ControlAction::DataIn { len, zlp } => {
        // send ctl.reply()[..len] in EP0_MAX_PACKET pieces, DATA1 first,
        // then a zero-length packet if zlp, then accept the 0-byte OUT status
    }
    ControlAction::DataOut { len } => {
        // receive one packet (DATA1); then act on ctl.data_out(&received[..n])
        // exactly as on a setup() result
    }
    ControlAction::NoData(effect) => {
        // apply effect now unless it is Effect::SetAddress; send a 0-byte IN (DATA1);
        // after it completes, program the SetAddress address
    }
    ControlAction::Stall => { /* stall both EP0 directions until the next SETUP */ }
}
// Data endpoints: arm them only while ctl.is_configured() && !ctl.is_halted(ep).
```

`pico2::usb`'s `handle_setup`, `run_action`, `ep0_in_complete`, `ep0_out_complete` and `apply_effect` are the reference implementation ([`firmware/pico2/src/usb/mod.rs:887-1058`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L887-L1058)). [07 Writing a driver](07-writing-a-driver.md) covers the rest of a new driver.

### Use the USB port from an application

The application uses `Rp2350Usb`, not `api::usb`, directly: call `poll()` at least once per millisecond, then `read` and `write` slices, and compare `link_epoch()` with the last value seen to learn that the host went away ([`firmware/pico2/src/usb/mod.rs:12-34`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L12-L34)). `write` may take fewer bytes than offered, and takes none until the host has configured the device ([`firmware/pico2/src/usb/mod.rs:461-480`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L461-L480)). On PR #6, `room()` tells a caller beforehand how many bytes `write` would take, so that it can send a message whole or not at all ([`firmware/pico2/src/usb/mod.rs:482-495`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/firmware/pico2/src/usb/mod.rs#L482-L495)). [05 I/O drivers](05-drivers-io.md) documents the driver.

### Start core 1 (PR #6)

Application code calls `Rp2350Core1::launch`, not `Launch`, directly ([06 Multicore](06-multicore.md)). Either way, start core 1 once per boot, on a core 1 that has just been reset; on any other core 1 the handshake may never finish ("Starting core 1" above). To drive `Launch` from another chip, implement `Mailbox` over that chip's FIFO and loop as the doc example does ([`api/src/multicore.rs:96-104`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L96-L104)):

```rust
use api::multicore::{Launch, Step};

let mut launch = Launch::new(vector_table, stack_top, entry_with_thumb_bit);
let start = now_us();
while launch.step(&mut mailbox, now_us()) == Step::Pending {
    if now_us() - start > timeout_us { return Err(NoAnswer); }
    // keep feeding the watchdog here if one is running
}
```

### Run the host tests

From the repository root, on the host:

```sh
cargo test -p api --locked                      # all unit tests and doctests in api
cargo test -p api --locked usb::control         # only tests whose path contains usb::control
cargo test -p api --locked set_address_rules    # one test by name
cargo clippy -p api --all-targets --locked -- -D warnings   # lint, as PR #6 did
```

`--locked` makes cargo use `Cargo.lock` as committed and fail rather than change it. The clippy line is the one page 09 lists; PR #6's description reports it clean at `410dffa` ([09 Work left and known issues](09-work-left-known-issues.md)). Toolchain: rustos has no `rust-toolchain.toml`; the README says it was "Verified with `rustc 1.98.0`" ([`README.md:272-273`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/README.md#L272-L273)). The firmware images that build `api` pin rustc 1.97.0 (see "Changing it"), so check a change on that version too.

`.cargo/config.toml` deliberately sets no default target, so these build for the machine you are on ([`.cargo/config.toml:1-7`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/.cargo/config.toml#L1-L7)). Do not use the `cargo pico2` alias for tests: it builds `pico2` for the chip. Cargo ignores the workspace's `panic = "abort"` profile setting ([`Cargo.toml:9-12`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/Cargo.toml#L9-L12)) for test builds, so failing assertions report normally.

What runs: at master, 35 `#[test]` functions in six files (`usb/mod.rs` 2, `setup.rs` 4, `descriptor.rs` 8, `cdc_acm.rs` 4, `control.rs` 15, `ring.rs` 2) and two doctests, the runnable examples on the two ZLP functions ([`api/src/usb/mod.rs:119-125`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/mod.rs#L119-L125), [`api/src/usb/mod.rs:144-150`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/mod.rs#L144-L150)). PR #6 adds nine tests in `multicore.rs`. Code blocks marked `ignore` or `text` are not compiled. rustos has no CI, so nothing runs these automatically; run them before and after every change. They were not run for this page (the handover rules forbid builds on the shared machine).

### Add a test

`common`, `device` and `gpio` have no tests at all, so nothing checks `take()` or the ownership rules (the "/ 0" in the Lines column under "Where it lives"). The `api::usb` modules and, on PR #6, `multicore.rs` do. For those, put a new test in the `#[cfg(test)] mod tests` block at the bottom of the file it tests; each block starts with `use super::*;`. The crate is `#![no_std]` and no existing test uses `std` or `alloc`: fixtures are arrays and constants, and even a 200-character string is built with `concat!` ([`api/src/usb/descriptor.rs:358-364`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L358-L364)). Keep to that.

For `control.rs`, build SETUP packets as eight bytes with the `pkt()` helper and start from the Configured state with `configured()` ([`api/src/usb/control.rs:510-528`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L510-L528)):

```rust
#[test]
fn line_coding_round_trip_through_the_handler() {
    let mut h = configured();
    // SET_LINE_CODING to interface 0, wLength 7
    assert_eq!(h.setup(&pkt([0x21, 0x20, 0, 0, 0, 0, 7, 0])), ControlAction::DataOut { len: 7 });
    // 57600 baud (0x0000E100), 1 stop bit, no parity, 8 data bits
    assert_eq!(h.data_out(&[0x00, 0xE1, 0x00, 0x00, 0, 0, 8]), ControlAction::NoData(Effect::None));
    // GET_LINE_CODING reads it back
    assert_eq!(h.setup(&pkt([0xA1, 0x21, 0, 0, 0, 0, 7, 0])), ControlAction::DataIn { len: 7, zlp: false });
    assert_eq!(h.reply(), &[0x00, 0xE1, 0x00, 0x00, 0, 0, 8]);
}
```

For `multicore.rs` (PR #6), reuse `Chip` and `run()`: put stale words into `chip.to_core0`, set `chip.echo`, or pass a schedule closure that decides whose turn each µs is ([`api/src/multicore.rs:359-383`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L359-L383)). A new `Mailbox` wrapper, like `Busy` or `Flood`, injects a fault ([`api/src/multicore.rs:431-497`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L431-L497)).

To test `define_board!`, invoke it in a test module of its own, with `Acquire` imported and a dummy device type:

```rust
#[cfg(test)]
mod board_tests {
    use core::sync::atomic::Ordering::Acquire; // the expansion names it bare
    pub struct Dummy;

    crate::define_board! {
        TestBoard {
            TestPins { a: 1, b: 2 }
            devices { dummy: Dummy }
        }
    }

    #[test]
    fn take_succeeds_once_per_process() {
        assert!(TestBoard::take().is_some());
        assert!(TestBoard::take().is_none());
    }
}
```

Each invocation has one `BOARD_TAKEN` flag, a `static` that every test in the process shares, so only one test may call `take()` on a given test board. The "second use of a handle is a compile error" rule can be pinned by a `compile_fail` doctest, which needs no board: make a handle with `unsafe { PinHandle::<1>::new() }`, move it twice, and the doctest passes only while that fails to compile. Neither test exists yet; this sketch was not compiled.

For a runnable documentation example, an untagged code block in a `///` comment is compiled and run as a doctest, as the ZLP functions show; mark examples that need the chip `ignore`, as `gpio/mod.rs` does ([`api/src/gpio/mod.rs:17-25`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L17-L25)).

## Changing it

**The trait signatures are the portability contract.** A change to `ErrorType`, `Read`, `Write` or the `gpio` traits changes every implementor in `pico2` (`gpio/gpio.rs`, `usb/mod.rs`, `pwm.rs`) and every application written against them. Prefer adding a trait or a method with a default over changing a signature, and say in the PR who is affected ([07 Writing a driver](07-writing-a-driver.md) has the same rule). Keep `api` free of register addresses and dependencies, so it stays host-testable ([`api/src/lib.rs:19-33`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/lib.rs#L19-L33)).

**Ownership rules that must not change.** `PinHandle::new` and `DeviceHandle::new` stay `unsafe`; the handles stay neither `Copy` nor `Clone`; `take()` stays a single compare-exchange that succeeds once per boot. These are what make "configured twice" a build error, and every driver relies on them ([`api/src/device/mod.rs:36-62`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L36-L62)).

**Where to change the rest:**

| Change | Files to touch | Tests to update |
|---|---|---|
| Change `define_board!` or a trait | `api/src/device/mod.rs` or the trait's file; `firmware/pico2/src/common/board.rs`; every implementor under "Who implements what"; every application under "Who uses the crate outside rustos" (each sees it only when its pin moves) | none exist; add the `define_board!` test above, and a `compile_fail` doctest for any ownership rule you touch |

**Where to change USB behaviour:**

| Change | Files to touch | Tests to update |
|---|---|---|
| Accept a new standard or class request | `control.rs` `standard()` or `class()`; a constant in `setup::request` or `cdc_acm::class_request`; for a class capability, the ACM `bmCapabilities` byte at [`api/src/usb/descriptor.rs:194`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L194). An OUT data stage longer than 64 bytes also needs multi-packet support in `pico2::usb` (`run_action` and `ep0_out_complete`), which stalls it today. **Vendor and reserved requests must keep stalling** ([`api/src/usb/control.rs:267`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L267)): see "Safety and the audit" below. | A test in `control.rs` per accepted and per stalled form |
| Add an endpoint or interface | `descriptor.rs` (constants, `CONFIGURATION_DESCRIPTOR`, `CONFIGURATION_DESCRIPTOR_LEN`, `NUM_INTERFACES`), `halt_bit` in `control.rs`, and in `pico2::usb` the DPRAM buffers, `enable_data_endpoints`, `disable_data_endpoints`, `reset_endpoint` and `apply_effect`. Keep `CONFIGURATION_DESCRIPTOR_LEN` at or below 255: `wTotalLength`'s high byte is written as 0 ([`api/src/usb/descriptor.rs:181`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L181)), and `reply_with` clips every reply to `REPLY_CAPACITY` (256) without an error ([`api/src/usb/control.rs:293-298`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L293-L298)). Past that, change both and raise `REPLY_CAPACITY`. | `configuration_descriptor_bytes` and `configuration_descriptor_is_self_consistent` (which counts 11 descriptors) ([`api/src/usb/descriptor.rs:275-310`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L275-L310)) |
| Change a packet size | `descriptor.rs`; `pico2::usb` asserts at compile time that every packet fits one 64-byte buffer ([`firmware/pico2/src/usb/mod.rs:273-275`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L273-L275)) | the descriptor byte tests |
| Add a string | a `STRING_*` index, a `UsbDeviceConfig` field, a match arm in `get_descriptor` ([`api/src/usb/control.rs:362-367`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L362-L367)), and the index field in the descriptor that names it. A new `UsbDeviceConfig` field is a breaking change: it derives no `Default`, so every struct literal must add the field, including the keyer's ([`firmware/pico2-keyer/src/main.rs:86-93`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L86-L93)) and the decoder's ([`firmware/pico2-decoder/src/main.rs:71-78`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/src/main.rs#L71-L78), PR #30). Those images see it only when their rustos pin moves, and that makes a new image, which needs a fresh audit and hash. | `get_string_descriptors` ([`api/src/usb/control.rs:556-578`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L556-L578)) |
| Change the rings' sizes | `RX_BUFFER_SIZE`, `TX_BUFFER_SIZE` in `pico2::usb` ([`firmware/pico2/src/usb/mod.rs:248-254`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/firmware/pico2/src/usb/mod.rs#L248-L254)); `ByteRing` itself is generic | none in `api` |
| Change the VID, PID or strings | the application's `UsbDeviceConfig`, not `api` | none in `api`; on the host side hfnode finds the keyer box by its product string, so changing it breaks discovery (cw-message-gateway's [`crates/hfnode/src/keyer.rs:370-374`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/hfnode/src/keyer.rs#L370-L374)) |

Changing a descriptor makes the device look new to a host that cached the old one; hosts cache driver choices per VID/PID ([`api/src/usb/descriptor.rs:76-83`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L76-L83)).

**`api::multicore` (PR #6).** Keep `step` non-blocking and bounded: the one-Pico keyer image (cw-message-gateway PR #31) starts its hardware watchdog ([`firmware/pico2-keyer/src/main.rs:285`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L285)) before it launches core 1 ([`firmware/pico2-keyer/src/main.rs:307`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L307)), so a `step` that blocked without bound would let the watchdog reset the box ([06 Multicore](06-multicore.md)). Do not remove the quiet wait, the drain, the start-over or the wake; PR #6's commit message ([`3d8af9c`](https://github.com/robinonsay/rustos/commit/3d8af9c2d3758ccf183f06670ab1f704ac2a417f)) says each has a test that fails without it (not re-run for this page). `QUIET_US` and `DRAIN_MAX` are the two tuning constants; the first sets the launch time, about `2 × QUIET_US`. The one-Pico keyer image gives the whole launch 10 ms (`CORE1_LAUNCH_US = 10_000`, beside its 500 ms watchdog; [`firmware/pico2-keyer/src/main.rs:98-100`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L98-L100), PR #31, draft), so `2 × QUIET_US` plus any restarts must stay well under 10 ms. If the launch runs out of time, that image runs with core 1 stopped and answers every receive request with `ERR RX OFF` ([`firmware/pico2-keyer/src/main.rs:306-308`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/firmware/pico2-keyer/src/main.rs#L306-L308), [`crates/keyer-core/src/control.rs:596`](https://github.com/robinonsay/cw-message-gateway/blob/bc5062ae4f56ae1f9949b3b067218c7e675d88f8/crates/keyer-core/src/control.rs#L596), both at PR #31).

**Safety and the audit.** `api` contains no keying limit, and nothing on this page loosens one. The keyer box's never-loosen rules (key line open at power-up, during USB enumeration, on host silence and in fault handlers, and the box limits) live in the cw-message-gateway keyer firmware and its `keyer-core` crate; the fault-handler rule is at [`firmware/pico2-keyer/src/main.rs:39-41`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L39-L41). That firmware does depend on this crate's USB behaviour. It reads `link_epoch()` ([`firmware/pico2-keyer/src/main.rs:136-138`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L136-L138)), and its control loop treats a changed epoch as a lost host: it calls `link_lost` and clears the reader and the output ([`crates/keyer-core/src/control.rs:379-386`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/keyer-core/src/control.rs#L379-L386)). Bytes flow only once the device is configured. Treat any change to when the device counts as configured, when the rings are cleared, or when `link_epoch` advances as a keyer-safety change. Read [13 Safety rules](https://github.com/robinonsay/cw-message-gateway/blob/claude/handover-docs-us00cn/docs/handover/13-safety-rules.md) and [08 Keyer box: firmware](https://github.com/robinonsay/cw-message-gateway/blob/claude/handover-docs-us00cn/docs/handover/08-keyer-box-firmware.md) before making one.

Three more rules for `api::usb`, because the keyer box runs it:

- **No panic reachable from host bytes.** A change must add no slice index, `unwrap` or arithmetic that a host's request can make fail. A panic opens the key and the PTT, then stops, so the watchdog resets the box ([`firmware/pico2-keyer/src/main.rs:39-41`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L39-L41)); it comes back tripped (`WATCHDOG`, or the trip it already had) and does not refill its duty budget ([`firmware/pico2-keyer/src/main.rs:33-38`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/src/main.rs#L33-L38), [`crates/keyer-core/src/keyer.rs:398-435`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/crates/keyer-core/src/keyer.rs#L398-L435)). The audit's report, written at cw-message-gateway `2e39996` before its fix KB-7 (PR #16), says such a panic clears a trip; the code no longer does (the watchdog-reset path is not yet checked on a chip: [09 Work left and known issues](09-work-left-known-issues.md), W15). The safety audit has not yet checked this code for such panics (Known bugs), so send any such change to the audit with the re-pin.
- **Vendor and reserved requests keep stalling** ([`api/src/usb/control.rs:267`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L267)). The safety audit's criterion A3 cites this line in its PASS (`audit/report.md`, criteria table, project files). Handling any vendor request, such as a reset into the bootloader, changes an audited property and needs the audit before any image re-pins to it.
- **Keep `api` compiling on rustc 1.97.0.** The keyer and decoder images build it with that pinned compiler ([`firmware/pico2-keyer/rust-toolchain.toml:5-7`](https://github.com/robinonsay/cw-message-gateway/blob/44185fa9a00369c379d9d8af40ca307e3867e860/firmware/pico2-keyer/rust-toolchain.toml#L5-L7); [`firmware/pico2-decoder/rust-toolchain.toml:5-7`](https://github.com/robinonsay/cw-message-gateway/blob/3efaf842e1aab75354e5e19d0384a703d4d786be/firmware/pico2-decoder/rust-toolchain.toml#L5-L7), PR #30). Moving to a newer compiler means a different firmware build.

Any rustos change that a firmware image picks up, by moving its pinned revision, changes the image. Each flashed file must match a hash the safety audit has cleared, and a new revision needs a fresh audit and a new hash before it is flashed ([14 Audit, bench and flashing](https://github.com/robinonsay/cw-message-gateway/blob/claude/handover-docs-us00cn/docs/handover/14-audit-bench-flashing.md)).

## Known bugs, limits and flakes

No test in `api` is known to fail or flake; with no CI, nothing records runs. Item 1 is a safety-audit open item; the rest are defects, limits and gaps in the code and docs.

1. **USB request handling has not been audited for panics.** The safety audit's report says, under its open items: "Not audited: implicit panics in the USB SETUP parsers (rustos/api/src/usb/control.rs and others). A host-triggered panic opens the key, but it also reboots the box and clears a trip" (`audit/report.md` §1, "Open items that need hardware (or missing documents)", project files). No later entry closes it. Its last clause is out of date: that section was written at cw-message-gateway `2e39996`, and since KB-7 a restart neither clears a trip nor refills the duty budget, and after a watchdog reset the box comes back tripped ("Safety and the audit" above). The parsers do have size guards: `SetupPacket::from_bytes` takes `[u8; 8]` ([`api/src/usb/setup.rs:76`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/setup.rs#L76)), `LineCoding::from_bytes` checks the length ([`api/src/usb/cdc_acm.rs:104-107`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/cdc_acm.rs#L104-L107)), and `reply_with` clips to `REPLY_CAPACITY` ([`api/src/usb/control.rs:293-298`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L293-L298)). That is not an audit. [09 Work left and known issues](09-work-left-known-issues.md) tracks it as K3.
2. **README "Host-side tests" is stale.** It says `cargo xtest`, "defined in `.cargo/config.toml`", runs the tests, and that "the suite currently contains no unit tests" ([`README.md:330-338`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/README.md#L330-L338)). At master the only alias is `cargo pico2` ([`.cargo/config.toml:6-7`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/.cargo/config.toml#L6-L7)), and there are 35 tests and two doctests. Use `cargo test -p api`.
3. **Docs name `api` items and a `demo` crate that no longer exist.** `lib.rs` says "the `demo` crate constructs `Rp2350Gpio`" ([`api/src/lib.rs:51-54`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/lib.rs#L51-L54)) and `gpio/mod.rs` says "The complete program is `demo/src/main.rs`" ([`api/src/gpio/mod.rs:31`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L31)); the workspace members are `api` and `firmware/pico2` ([`Cargo.toml:3`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/Cargo.toml#L3)), and the example program is `templates/pico2/src/main.rs`. The README's layout section has the same stale `demo/` entry ([`README.md:69-77`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/README.md#L69-L77)). The bare-metal tutorial's chapter 07 lists `api::common::Block`, `api::common::board::Board` and a `GpioPin` trait, none of which exist, and shows `use api::common::{Block, ErrorType, Read, Write};` ([`docs/tutorials/rp2350_baremetal/07_registers_and_bits.md:585-608`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/docs/tutorials/rp2350_baremetal/07_registers_and_bits.md#L585-L608)). `.vscode/settings.json` says tests run "via `cargo xtest`" ([`.vscode/settings.json:2-6`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/.vscode/settings.json#L2-L6)). [01 Overview](01-overview.md) and [09 Work left and known issues](09-work-left-known-issues.md) record both.
4. **`define_board!` does not reject duplicate pin numbers or device types.** `a: 5, b: 5` would compile and create two `PinHandle<5>` (the expansion writes one field per entry with no check; [`api/src/device/mod.rs:258-260`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L258-L260), [`api/src/device/mod.rs:275-280`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L275-L280)). The "only real pins" guarantee depends on the invoker listing each pin once. No such check was found in the code at `80bf599`.
5. **`define_board!`'s generated `new()` is safe and visible to the whole invoking module.** Privacy in Rust is per module, so code next to the invocation can call `MyBoard::new()` without `unsafe` and get a second set of every handle. The macro's own comment says it is "Sound only because this is private and its sole caller is `take()`" ([`api/src/device/mod.rs:269-280`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L269-L280)). `pico2` is safe because `common/board.rs` holds nothing but the invocation; invoke the macro in a module of its own.
6. **`define_board!` rough edges** (documented): `Acquire` must be imported by the invoking module, and there is one board per module ([`api/src/device/mod.rs:233-238`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L233-L238)).
7. **`take()` needs an atomic compare-and-swap** ([`api/src/device/mod.rs:314-319`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/device/mod.rs#L314-L319)). Rust's `core` offers `AtomicBool::compare_exchange` only on targets that have one. `rustc --print cfg --target thumbv6m-none-eabi` (rustc 1.97.0) reports no `target_has_atomic`, so a board for an Armv6-M chip such as the RP2040 would need a different claim, for example a critical section. Unverified: nobody has tried to build `api` for such a target.
8. **`Pull` has no derives** (no `Clone`, `Copy`, `Debug`, `PartialEq`), so a configuration value cannot be stored and reused or compared ([`api/src/gpio/mod.rs:64-72`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/gpio/mod.rs#L64-L72)).
9. **Class requests are not gated on the Configured state.** `class()` checks only the recipient and interface number ([`api/src/usb/control.rs:467-470`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L467-L470)), so SET_LINE_CODING and SET_CONTROL_LINE_STATE are accepted in the Default and Address states, while the standard interface requests stall there. Harmless in practice; noted as an inconsistency.
10. **The notification endpoint never sends.** EP1 IN exists because ACM requires it and NAKs every poll; no SERIAL_STATE notification is ever sent ([`api/src/usb/descriptor.rs:39-46`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L39-L46)). SEND_BREAK is accepted and ignored, and its capability bit is clear ([`api/src/usb/descriptor.rs:172-177`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L172-L177)).
11. **No remote wakeup, bus powered only.** GET_STATUS reports `00 00` and SET_FEATURE(DEVICE_REMOTE_WAKEUP) stalls ([`api/src/usb/control.rs:381-383`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L381-L383), [`api/src/usb/control.rs:459-461`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/control.rs#L459-L461)).
12. **`ByteRing` is single-context.** It is not synchronised ([`api/src/usb/ring.rs:18-19`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/ring.rs#L18-L19)). With two cores running (PR #6), never share one between them; use the FIFO or atomics ([06 Multicore](06-multicore.md)).
13. **String truncation can split a surrogate pair** at the 126-unit limit (documented, [`api/src/usb/descriptor.rs:228-230`](https://github.com/robinonsay/rustos/blob/80bf5998a60a41b3741041062ff7e9c59caf95b2/api/src/usb/descriptor.rs#L228-L230)).
14. **`cargo fmt --check` fails.** rustfmt 1.9.0 (`--check`, edition 2024, run read-only for this page) reports changes in eight `api` files: `lib.rs`, `device/mod.rs`, `gpio/mod.rs`, and `usb/mod.rs`, `cdc_acm.rs`, `control.rs`, `descriptor.rs` and `ring.rs`. `common/mod.rs`, `usb/setup.rs` and PR #6's `multicore.rs` are clean. Page 09 counts 19 files across the repository (K4). Do not reformat inside a functional change; the one-off `cargo fmt --all` belongs in its own PR.
15. **`common`, `device` and `gpio` have no tests.** Nothing checks `take()`, the handle rules or the trait shapes; "Add a test" gives a recipe.
16. **Unverified on hardware:** USB enumeration on Linux and Windows (only macOS is recorded); all of `api::multicore` and its 200 µs launch figure (PR #6, "Untested on hardware" in its commit message). The handshake's FIFO depth is quoted from the datasheet as both four and eight words; `DRAIN_MAX` = 16 covers both ([`api/src/multicore.rs:81-85`](https://github.com/robinonsay/rustos/blob/410dffa091e247e6e7546c7f2ff465f0af499d5a/api/src/multicore.rs#L81-L85)).

## Work left

- **Merge rustos PR #6** (open, head `410dffa`). After it merges, the one-Pico keyer image (cw-message-gateway PR #31, draft) re-pins rustos at master's merge commit, which changes that image and so needs a fresh audit and a new hash before it can be flashed. [09 Work left and known issues](09-work-left-known-issues.md) tracks this.
- **An audit of `api::usb` for panics a host can trigger** (Known bug 1, page 09 K3). It waits on the safety audit; Robin asks for it, and the audit decides.
- **Fix the stale docs** listed in Known bugs 2 and 3 (README test section and layout, the `demo` references in `lib.rs` and `gpio/mod.rs`, tutorial chapter 07's `Block`, `Board` and `GpioPin`, and `cargo xtest` in `.vscode/settings.json`).
- **Decide whether `define_board!` should reject duplicate pins** (Known bug 4); no plan for it was found.
- **Add tests for `common`, `device` and `gpio`** (Known bug 15), starting with the `define_board!` test under "Add a test".
- **Run `cargo fmt --all` once, in a PR of its own** (Known bug 14, page 09 K4).
- **Add CI** that runs `cargo test -p api` on every push; rustos has none.
- **Hardware checks still open:** USB on Linux and Windows hosts, and the whole multicore path on a chip (the keyer image's bring-up covers the latter; see [06 Multicore](06-multicore.md)).

## Sources

- rustos at master `80bf599`: `api/Cargo.toml`, `api/src/lib.rs`, `api/src/common/mod.rs`, `api/src/device/mod.rs`, `api/src/gpio/mod.rs`, `api/src/usb/{mod,setup,descriptor,cdc_acm,control,ring}.rs` (read in full); `firmware/pico2/src/usb/mod.rs` and `usb/hw.rs`, `firmware/pico2/src/gpio/gpio.rs`, `firmware/pico2/src/common/board.rs`, `firmware/pico2/src/pwm.rs` and `adc.rs` (the `api` impls and `*_from_handle`), the nine drivers' `new`; `templates/pico2/src/main.rs` and `templates/pico2/Cargo.toml.liquid`, `Cargo.toml`, `Cargo.lock` (present), `.cargo/config.toml`, `.vscode/settings.json`, `README.md`, `docs/tutorials/rp2350_baremetal/07_registers_and_bits.md` (§ on `api`); `git log` of `api/` (Robin's commits `4866bf4` to `2734898`; `c6fd4bf`, merged as `cfc2b14`); `git diff c9852112 80bf599` and `git diff 8c91507 80bf599` over `api/` and `firmware/pico2/src/usb/` (both empty); `rustfmt --check` 1.9.0 over `api/src`; `rustc --print cfg` for `thumbv6m-none-eabi` and `thumbv8m.main-none-eabihf`.
- rustos PR #6 at `410dffa`: `api/src/multicore.rs` (read in full), `api/src/lib.rs`, `firmware/pico2/src/multicore.rs` (the `Mailbox` implementation, `Rp2350Core1::new` and `launch`), `firmware/pico2/src/usb/mod.rs` (`room()`); commit messages of `3d8af9c`, `422320d`, `410dffa`.
- cw-message-gateway at main `44185fa`: `firmware/pico2-keyer` (`Cargo.toml`, `rust-toolchain.toml`, `src/main.rs`), `crates/keyer-core/src/control.rs`, `crates/hfnode/src/keyer.rs`; keyer build `eef8c126` (`firmware/pico2-keyer/Cargo.toml`); PR #30 at `3efaf84` (`firmware/pico2-decoder`: `Cargo.toml`, `rust-toolchain.toml`, `src/main.rs`); PR #31 at `bc5062a` (`firmware/pico2-keyer/src/main.rs` and `core1.rs`, `crates/keyer-core/src/control.rs`).
- Project files: `bench/quansheng.md` (the 2026-10-09 log, 16:00Z to 16:19Z: the flash of `eef8c126`, enumeration on macOS and `hfnode keyer check`); `audit/report.md` (criterion A3 and §1 open items; the same open item in `audit/req-keyer-box-fixes.md`).
- Sibling pages: [01 Overview](01-overview.md), [02 Boot and runtime](02-boot-runtime.md), [04 Core drivers](04-drivers-core.md), [05 I/O drivers](05-drivers-io.md), [06 Multicore](06-multicore.md), [07 Writing a driver](07-writing-a-driver.md), [08 Changes for cw-message-gateway](08-changes-for-cw-gateway.md), [09 Work left and known issues](09-work-left-known-issues.md).
