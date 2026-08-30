//! Pico 2 board definition: every pin that exists, every peripheral device,
//! and what the board has already wired some of the pins to.
//!
//! The `define_board!` invocation below is the only place in the system where
//! pin- and device-ownership handles are created. It expands to four things:
//!
//! * **`Rp2350Pins`** — a struct with one `pub` field per physical pin, each
//!   of type `api::device::PinHandle<N>` where `N` is that pin's number.
//!   A `PinHandle` is zero-sized: it stores nothing and costs nothing at
//!   runtime. Its value is what possession of it proves. The handle's one
//!   field is private and its `new` is `unsafe`, so the only way safe code
//!   obtains a `PinHandle<25>` is by moving it out of this struct — making
//!   the handle compile-time evidence that pin 25 exists on this board and
//!   that no other code owns it. Driver constructors such as
//!   `Gpio::output_from_handle` take the handle *by value*, consuming it, so
//!   each pin can be turned into a configured pin at most once.
//!   (`PinHandle::new` is the one constructor available outside a board
//!   definition; it is `const unsafe` because the caller, not the board,
//!   must then guarantee the pin exists and is unowned.)
//! * **`gpio: DeviceHandle<Rp2350Gpio>`** — one field per entry in the
//!   `devices` section, the peripheral-level counterpart of a `PinHandle`:
//!   a zero-sized claim on the whole GPIO port. `Rp2350Gpio::new` takes it
//!   by value, so the driver can be constructed at most once, and hardware
//!   bring-up (releasing the GPIO blocks from reset) happens there — when
//!   the peripheral is claimed, not at `take()`. A peripheral no code claims
//!   is never brought up.
//! * **`Rp2350`** — the board struct, holding `pins: Rp2350Pins` and the
//!   device handle fields.
//! * **`Rp2350::take() -> Option<Rp2350>`** — the singleton constructor. It
//!   flips a `static AtomicBool` with `compare_exchange`: one indivisible
//!   read-modify-write, so exactly one caller can observe `false` and store
//!   `true`. That matters because the RP2350 has two Cortex-M33 cores
//!   sharing memory — see the "Why `compare_exchange`, not a load followed
//!   by a store" note on `define_board!` in `api::device`. The first caller
//!   gets `Some` with the full set of handles and every later call — from
//!   anywhere, including the other core — gets `None`. That single runtime
//!   check is the root of the ownership scheme: handles are constructed
//!   exactly once per boot, inside `take()`, and every guarantee after that
//!   point is enforced by moves, at compile time, for free.
//!
//! ## Why some pins have semantic names
//!
//! The field names answer the question "what is safe to wire external
//! hardware to?". The RP2350A package bonds out GPIO0–29, but the Pico 2
//! board commits four of those to its own circuitry and does not route them
//! to the 40-pin header. A field named `gpioN` is a free pin on the header; a
//! field with a semantic name is already committed to on-board circuitry,
//! and the name says to what:
//!
//! Two rails appear below: VBUS is the 5 V arriving over the USB connector,
//! and VSYS is the board's main input rail (VBUS or an external supply),
//! which the on-board SMPS — switched-mode power supply, the regulator —
//! converts to the 3.3 V the chip runs on. Two peripheral terms appear too:
//! PWM is pulse-width modulation — here, the regulator switching at a fixed
//! rate regardless of load — and an ADC is an analog-to-digital converter,
//! so an "ADC input" pin can be sampled as a voltage, not just as high/low.
//!
//! | Field | GPIO | Board function |
//! |-------|------|----------------|
//! | `smps_ps` | 23 | Output to the SMPS power-save pin: low lets the regulator drop into pulse-skipping mode at light load (efficient, but more output ripple); high forces continuous PWM mode (steadier 3.3 V, useful during ADC reads). |
//! | `vbus_sense` | 24 | Input, reads high when VBUS power is present. |
//! | `led` | 25 | Output driving the on-board user LED (high = lit). |
//! | `vsys_adc` | 29 | ADC3 input wired to VSYS through a divider that reads VSYS/3, for measuring the supply voltage. Not on the header. |
//!
//! `gpio26`–`gpio28` are on the header and stay `gpioN`-named, but they are
//! the three pins that double as ADC inputs 0–2, so prefer other pins for
//! purely digital jobs.

// `define_board!` expands to a `compare_exchange(false, true, Acquire,
// Acquire)` spelled with the bare name, so the invoking module must have it
// in scope. This import looks unused; the macro expansion consumes it.
// `Acquire` is a memory ordering — see the "memory orderings" note on
// `define_board!` in `api::device` for what an ordering is and what
// `Acquire` forbids.
use core::sync::atomic::Ordering::Acquire;

use api::define_board;

use crate::gpio::gpio::Rp2350Gpio;


define_board!{
    Rp2350{
        Rp2350Pins {
            gpio0: 0,
            gpio1: 1,
            gpio2: 2,
            gpio3: 3,
            gpio4: 4,
            gpio5: 5,
            gpio6: 6,
            gpio7: 7,
            gpio8: 8,
            gpio9: 9,
            gpio10: 10,
            gpio11: 11,
            gpio12: 12,
            gpio13: 13,
            gpio14: 14,
            gpio15: 15,
            gpio16: 16,
            gpio17: 17,
            gpio18: 18,
            gpio19: 19,
            gpio20: 20,
            gpio21: 21,
            gpio22: 22,
            // GP23–GP25 and GP29 are committed to on-board functions on the
            // Pico 2 and are not routed to the 40-pin header; named for what
            // the board wired them to rather than as free GPIO.
            smps_ps: 23,     // SMPS power-save select
            vbus_sense: 24,  // high when USB power is present
            led: 25,         // on-board user LED
            gpio26: 26,      // header, also ADC0
            gpio27: 27,      // header, also ADC1
            gpio28: 28,      // header, also ADC2
            vsys_adc: 29,    // ADC3 reads VSYS/3; not on header
        }
        devices {
            gpio: Rp2350Gpio,
        }
    }
}
