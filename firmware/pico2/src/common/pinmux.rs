//! Routing a GPIO pin to a serial peripheral: which pins can carry which
//! UART, I2C and SPI signal, and the register sequence that connects one.
//!
//! # Which pin carries which signal
//!
//! Every Bank 0 GPIO can be given to one of up to twelve functions,
//! F0–F11, by writing the function's number to that pin's
//! `GPIOn_CTRL.FUNCSEL` field. The serial buses use four columns of the
//! function table,
//! [Table 644, §9.4, p588–589](../../../../docs/rp2350-datasheet.pdf#page=588):
//!
//! | Column | `FUNCSEL` | Signals |
//! |--------|-----------|---------|
//! | F1  | 1  | SPI RX, CSn, SCK, TX |
//! | F2  | 2  | UART TX, RX, CTS, RTS |
//! | F3  | 3  | I2C SDA, SCL |
//! | F11 | 11 | a second UART TX/RX on the pins whose F2 is CTS/RTS |
//!
//! The `FUNCSEL` encoding is the column number: `GPIO0_CTRL` lists
//! `0x01 → SPI0_RX`, `0x02 → UART0_TX`, `0x03 → I2C0_SDA`
//! ([Table 650, p609](../../../../docs/rp2350-datasheet.pdf#page=609)),
//! and `GPIO2_CTRL` lists `0x0b → UART0_TX`
//! ([p612](../../../../docs/rp2350-datasheet.pdf#page=612)).
//!
//! [`BANK0`] transcribes those four columns for GPIO0–29, the pins the
//! Pico 2's RP2350A has ([`MAX_GPIO_PIN`]). The drivers look a pin up
//! there in a `const` context, so a pin that cannot carry the signal it is
//! given is a **build error**, not a runtime one.
//!
//! # Connecting a pin
//!
//! [`connect`] runs the sequence the GPIO and PWM drivers already use:
//! set up the pad, select the function, then release the pad's isolation
//! latch last. §9.7 says to clear `ISO` "once software has finished setting
//! up the IO muxing for a given pad, and the peripheral which is to be
//! muxed in"
//! ([§9.7, p594](../../../../docs/rp2350-datasheet.pdf#page=594)), so each
//! driver configures its peripheral first and connects its pins last.

use crate::common::MAX_GPIO_PIN;
use crate::common::reg::RegAddr;
use crate::common::reset::{clr_reset_reg, wait_for_reset_done};
use crate::gpio::{IoBank, PadsBank};

/// One signal a peripheral function puts on a pin. The UART modem lines
/// (CTS, RTS) and SPI CSn are listed so that the table is a faithful copy
/// of the datasheet, even though no driver here routes them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    /// SPI receive data (MISO when the RP2350 is the controller).
    SpiRx,
    /// SPI hardware chip-select. Not used: chip-select is a GPIO output.
    SpiCsn,
    /// SPI clock.
    SpiSck,
    /// SPI transmit data (MOSI when the RP2350 is the controller).
    SpiTx,
    /// UART transmit data.
    UartTx,
    /// UART receive data.
    UartRx,
    /// UART clear-to-send input. Not used: no flow control.
    UartCts,
    /// UART request-to-send output. Not used: no flow control.
    UartRts,
    /// I2C data.
    I2cSda,
    /// I2C clock.
    I2cScl,
}

/// A signal of one instance of a peripheral, such as "UART1 TX".
#[derive(Clone, Copy)]
pub(crate) struct Signal {
    /// Which instance: 0 or 1 (`UART0`/`UART1`, ...).
    instance: u8,
    /// Which of its signals.
    role: Role,
}

/// The four serial-bus columns of one row of Table 644.
#[derive(Clone, Copy)]
struct Row {
    /// Column F1, `FUNCSEL` = 1.
    f1: Signal,
    /// Column F2, `FUNCSEL` = 2.
    f2: Signal,
    /// Column F3, `FUNCSEL` = 3.
    f3: Signal,
    /// Column F11, `FUNCSEL` = 11; empty on most pins.
    f11: Option<Signal>,
}

/// Shorthand for one table cell, to keep [`BANK0`] in the datasheet's
/// shape.
const fn s(instance: u8, role: Role) -> Signal {
    // Plain constructor; no checks needed for a literal table.
    Signal { instance, role }
}

/// One row with no F11 entry.
const fn row(f1: Signal, f2: Signal, f3: Signal) -> Row {
    // F11 is blank for this pin in Table 644.
    Row { f1, f2, f3, f11: None }
}

/// One row with an F11 entry.
const fn row11(f1: Signal, f2: Signal, f3: Signal, f11: Signal) -> Row {
    // F11 carries a second UART TX or RX on this pin in Table 644.
    Row { f1, f2, f3, f11: Some(f11) }
}

// Short role names, so each table row fits on one line like the datasheet.
use Role::{I2cScl as SCL, I2cSda as SDA, SpiCsn as CSN, SpiRx as SRX, SpiSck as SCK, SpiTx as STX};
use Role::{UartCts as CTS, UartRts as RTS, UartRx as URX, UartTx as UTX};

/// Columns F1, F2, F3 and F11 of
/// [Table 644, p588–589](../../../../docs/rp2350-datasheet.pdf#page=588),
/// GPIO0–29, transcribed row by row. Index by GPIO number.
const BANK0: [Row; MAX_GPIO_PIN] = [
    //      F1 (SPI)     F2 (UART)    F3 (I2C)     F11 (UART aux)
    row(s(0, SRX), s(0, UTX), s(0, SDA)),              // GPIO0
    row(s(0, CSN), s(0, URX), s(0, SCL)),              // GPIO1
    row11(s(0, SCK), s(0, CTS), s(1, SDA), s(0, UTX)), // GPIO2
    row11(s(0, STX), s(0, RTS), s(1, SCL), s(0, URX)), // GPIO3
    row(s(0, SRX), s(1, UTX), s(0, SDA)),              // GPIO4
    row(s(0, CSN), s(1, URX), s(0, SCL)),              // GPIO5
    row11(s(0, SCK), s(1, CTS), s(1, SDA), s(1, UTX)), // GPIO6
    row11(s(0, STX), s(1, RTS), s(1, SCL), s(1, URX)), // GPIO7
    row(s(1, SRX), s(1, UTX), s(0, SDA)),              // GPIO8
    row(s(1, CSN), s(1, URX), s(0, SCL)),              // GPIO9
    row11(s(1, SCK), s(1, CTS), s(1, SDA), s(1, UTX)), // GPIO10
    row11(s(1, STX), s(1, RTS), s(1, SCL), s(1, URX)), // GPIO11
    row(s(1, SRX), s(0, UTX), s(0, SDA)),              // GPIO12
    row(s(1, CSN), s(0, URX), s(0, SCL)),              // GPIO13
    row11(s(1, SCK), s(0, CTS), s(1, SDA), s(0, UTX)), // GPIO14
    row11(s(1, STX), s(0, RTS), s(1, SCL), s(0, URX)), // GPIO15
    row(s(0, SRX), s(0, UTX), s(0, SDA)),              // GPIO16
    row(s(0, CSN), s(0, URX), s(0, SCL)),              // GPIO17
    row11(s(0, SCK), s(0, CTS), s(1, SDA), s(0, UTX)), // GPIO18
    row11(s(0, STX), s(0, RTS), s(1, SCL), s(0, URX)), // GPIO19
    row(s(0, SRX), s(1, UTX), s(0, SDA)),              // GPIO20
    row(s(0, CSN), s(1, URX), s(0, SCL)),              // GPIO21
    row11(s(0, SCK), s(1, CTS), s(1, SDA), s(1, UTX)), // GPIO22
    row11(s(0, STX), s(1, RTS), s(1, SCL), s(1, URX)), // GPIO23
    row(s(1, SRX), s(1, UTX), s(0, SDA)),              // GPIO24
    row(s(1, CSN), s(1, URX), s(0, SCL)),              // GPIO25
    row11(s(1, SCK), s(1, CTS), s(1, SDA), s(1, UTX)), // GPIO26
    row11(s(1, STX), s(1, RTS), s(1, SCL), s(1, URX)), // GPIO27
    row(s(1, SRX), s(0, UTX), s(0, SDA)),              // GPIO28
    row(s(1, CSN), s(0, URX), s(0, SCL)),              // GPIO29
];

/// `true` if `cell` is signal `role` of peripheral instance `instance`.
const fn is(cell: Signal, instance: usize, role: Role) -> bool {
    // Fieldless enums compare in a `const fn` through their discriminants.
    cell.instance as usize == instance && cell.role as u8 == role as u8
}

/// The `FUNCSEL` value that puts signal `role` of instance `instance` on
/// GPIO `pin`, or `None` if Table 644 has no such cell for that pin (or
/// the pin does not exist on this package).
///
/// When a pin has the signal in two columns, the lower column wins; no
/// pin in Table 644 does.
pub(crate) const fn funcsel(pin: usize, instance: usize, role: Role) -> Option<u32> {
    // Pins 30–47 exist only on the QFN-80 package (Table 644, p589).
    if pin >= MAX_GPIO_PIN {
        return None;
    }
    // The datasheet row for this pin.
    let r = BANK0[pin];
    if is(r.f1, instance, role) {
        // Column F1: FUNCSEL 0x01 (Table 650, p609).
        Some(1)
    } else if is(r.f2, instance, role) {
        // Column F2: FUNCSEL 0x02 (Table 650, p609).
        Some(2)
    } else if is(r.f3, instance, role) {
        // Column F3: FUNCSEL 0x03 (Table 650, p609).
        Some(3)
    } else {
        match r.f11 {
            // Column F11: FUNCSEL 0x0b (e.g. GPIO2_CTRL, p612).
            Some(cell) if is(cell, instance, role) => Some(11),
            // No column of this row carries the signal.
            _ => None,
        }
    }
}

// Spot checks of the lookup against cells read straight off Table 644
// (p588–589), so a typo in `BANK0` or a mistake in `funcsel` fails the
// build. Each line is one cell: (GPIO, instance, signal) → column.
const _: () = assert!(matches!(funcsel(0, 0, Role::UartTx), Some(2))); // GPIO0  F2 UART0 TX
const _: () = assert!(matches!(funcsel(1, 0, Role::UartRx), Some(2))); // GPIO1  F2 UART0 RX
const _: () = assert!(matches!(funcsel(2, 0, Role::UartTx), Some(11))); // GPIO2  F11 UART0 TX
const _: () = assert!(matches!(funcsel(4, 1, Role::UartTx), Some(2))); // GPIO4  F2 UART1 TX
const _: () = assert!(matches!(funcsel(5, 1, Role::UartRx), Some(2))); // GPIO5  F2 UART1 RX
const _: () = assert!(matches!(funcsel(8, 0, Role::I2cSda), Some(3))); // GPIO8  F3 I2C0 SDA
const _: () = assert!(matches!(funcsel(9, 0, Role::I2cScl), Some(3))); // GPIO9  F3 I2C0 SCL
const _: () = assert!(matches!(funcsel(10, 1, Role::I2cSda), Some(3))); // GPIO10 F3 I2C1 SDA
const _: () = assert!(matches!(funcsel(16, 0, Role::SpiRx), Some(1))); // GPIO16 F1 SPI0 RX
const _: () = assert!(matches!(funcsel(18, 0, Role::SpiSck), Some(1))); // GPIO18 F1 SPI0 SCK
const _: () = assert!(matches!(funcsel(19, 0, Role::SpiTx), Some(1))); // GPIO19 F1 SPI0 TX
const _: () = assert!(matches!(funcsel(27, 1, Role::UartRx), Some(11))); // GPIO27 F11 UART1 RX
// And cells that must not match: wrong instance, wrong signal, no pin.
const _: () = assert!(funcsel(0, 1, Role::UartTx).is_none()); // GPIO0 is UART0, not UART1
const _: () = assert!(funcsel(0, 0, Role::UartRx).is_none()); // GPIO0 has no UART RX
const _: () = assert!(funcsel(30, 0, Role::UartCts).is_none()); // GPIO30 is QFN-80 only

/// Electrical settings for a pad, applied by [`connect`]. Fields map to
/// bits of `PADS_BANK0.GPIOn`,
/// [Table 852, p785](../../../../docs/rp2350-datasheet.pdf#page=785).
#[derive(Clone, Copy)]
pub(crate) struct PadConfig {
    /// `PUE` (bit 3): weak pull-up. `PDE` (bit 2) is always cleared, so a
    /// pad is either pulled up or not pulled at all.
    pub pull_up: bool,
    /// `SLEWFAST` (bit 0): fast output edges.
    pub slew_fast: bool,
}

/// `PADS_BANK0.GPIOn.ISO`, bit 8, reset 1 (Table 852, p785).
const PAD_ISO: u32 = 1 << 8;
/// `PADS_BANK0.GPIOn.OD`, bit 7: output disable (Table 852, p785).
const PAD_OD: u32 = 1 << 7;
/// `PADS_BANK0.GPIOn.IE`, bit 6, reset 0: input enable (Table 852, p785).
const PAD_IE: u32 = 1 << 6;
/// `PADS_BANK0.GPIOn.PUE`, bit 3: pull-up enable (Table 852, p785).
const PAD_PUE: u32 = 1 << 3;
/// `PADS_BANK0.GPIOn.PDE`, bit 2, reset 1: pull-down enable (Table 852, p785).
const PAD_PDE: u32 = 1 << 2;
/// `PADS_BANK0.GPIOn.SCHMITT`, bit 1, reset 1 (Table 852, p785).
const PAD_SCHMITT: u32 = 1 << 1;
/// `PADS_BANK0.GPIOn.SLEWFAST`, bit 0, reset 0 (Table 852, p785).
const PAD_SLEWFAST: u32 = 1 << 0;

/// `IO_BANK0` (bit 6) and `PADS_BANK0` (bit 9) of `RESETS.RESET`
/// ([Table 534, p504](../../../../docs/rp2350-datasheet.pdf#page=504)):
/// the two blocks every pin is routed through.
const RESET_IO_PADS: u32 = (1 << 6) | (1 << 9);

/// Release `IO_BANK0` and `PADS_BANK0` from reset, if nothing has yet, and
/// wait until both are ready.
///
/// Never asserts their reset, which would disturb pins other drivers own;
/// the PWM driver does the same ([`crate::pwm`]).
///
/// # Safety
///
/// Writes `RESETS.RESET`; see [`clr_reset_reg`].
pub(crate) unsafe fn release_io_banks() {
    unsafe {
        // AND with the complement: clears only bits 6 and 9 (release).
        clr_reset_reg(!RESET_IO_PADS);
        // RESET_DONE reads 1 once each block is out of reset (Table 536, p506).
        wait_for_reset_done(RESET_IO_PADS);
    }
}

/// Give GPIO `pin` to the peripheral function `funcsel`, with the pad set
/// up as `pad`.
///
/// The same order as the GPIO driver's output path: pad first, then
/// `FUNCSEL`, then `ISO` cleared last
/// ([§9.7, p594](../../../../docs/rp2350-datasheet.pdf#page=594)). Until
/// `ISO` is cleared the pad holds its latched state; on a pin untouched
/// since power-on, that is high-impedance and pulled low
/// ([§9.3, p586](../../../../docs/rp2350-datasheet.pdf#page=586)).
///
/// Always set: `IE` = 1, which §9.3 asks for "before using the pads for
/// digital I/O" (p587) and which every input signal (UART RX, I2C SDA and
/// SCL, SPI RX) needs; `OD` = 0, so the peripheral's own output enable
/// decides whether the pad drives (Table 852: `OD` "has priority over
/// output enable from peripherals"); `SCHMITT` = 1, its reset value; `PDE`
/// = 0. `DRIVE` keeps its value: 4 mA on a pad nothing else has touched.
///
/// # Safety
///
/// `pin` < [`MAX_GPIO_PIN`] and the caller owns it (holds or consumed its
/// `PinHandle`); `funcsel` is a value [`funcsel`] returned for it;
/// `IO_BANK0` and `PADS_BANK0` are out of reset ([`release_io_banks`]).
pub(crate) unsafe fn connect(pin: usize, funcsel: u32, pad: PadConfig) {
    // PADS_BANK0 viewed as its register layout (base from RegAddr).
    let pads_addr = RegAddr::PADS_BANK0 as usize as *mut PadsBank;
    // IO_BANK0 viewed as its register layout.
    let io_addr = RegAddr::IO_BANK0 as usize as *mut IoBank;
    unsafe {
        // Raw pointer to this pin's pad register: PADS_BANK0 + 4 + 4 * pin.
        let pad_reg = &raw mut (*pads_addr).pads[pin];
        // Start from the current value so DRIVE keeps its setting.
        let mut value = pad_reg.read_volatile();
        // Output disable off: the peripheral's output enable decides.
        value &= !PAD_OD;
        // Input buffer on: needed to receive anything on this pin.
        value |= PAD_IE;
        // Schmitt trigger on (its reset value), for clean input edges.
        value |= PAD_SCHMITT;
        // No pull-down on any serial-bus pin.
        value &= !PAD_PDE;
        // Pull-up only where asked for.
        if pad.pull_up {
            value |= PAD_PUE
        } else {
            value &= !PAD_PUE
        }
        // Fast edges only where asked for.
        if pad.slew_fast {
            value |= PAD_SLEWFAST
        } else {
            value &= !PAD_SLEWFAST
        }
        // One store: ISO is still 1 here, so nothing reaches the pin yet.
        pad_reg.write_volatile(value);
        // GPIOn_CTRL: FUNCSEL in bits 4:0, every override field "normal"
        // (0), so output, output enable and input come from the
        // peripheral (Table 650, p608–609).
        let ctrl = &raw mut (*io_addr).gpio[pin].ctrl;
        // Whole-register write: selects the function, clears overrides.
        ctrl.write_volatile(funcsel);
        // Re-read the pad register and release the isolation latch last.
        let value = pad_reg.read_volatile();
        // From this store on, the peripheral owns the pin.
        pad_reg.write_volatile(value & !PAD_ISO);
    }
}
