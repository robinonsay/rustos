//! # `api` — portable hardware-abstraction traits
//!
//! This crate is the **portability boundary** of the system. It contains the
//! trait definitions the rest of the workspace is written against, plus the
//! pin-ownership machinery in [`device`]: the zero-sized
//! [`device::PinHandle`] and the [`define_board!`] macro that generates the
//! board type creating one handle per physical pin, once per boot.
//!
//! Two Rust terms carry this whole crate, so first: a **trait** is a named
//! set of function signatures — the compile-time counterpart of an
//! interface. A type opts in by providing an `impl` block containing those
//! functions, and generic code written against the trait (a bound such as
//! `T: Write<bool>`) accepts any implementing type. A **register** is a
//! hardware-defined word at a fixed memory address through which a
//! peripheral is controlled; any code that can dereference such an address
//! can reconfigure any peripheral, which is why pin ownership needs
//! enforcement above the hardware — the job of [`device`].
//!
//! There are no register addresses and no dependency on any other crate in
//! the workspace. An `unsafe fn` is one whose preconditions the compiler
//! cannot verify: the caller must wrap the call in an `unsafe` block and is
//! responsible for the obligations listed in the function's `# Safety`
//! section. The only `unsafe fn` this crate exposes is
//! [`device::PinHandle::new`], the constructor that creates a handle without
//! a board; the crate's one `unsafe` block is the call to it in the code
//! [`define_board!`] generates, which expands into the invoking crate and is
//! the intended caller the Safety section describes. Everything here
//! compiles for the host as readily as it does for
//! `thumbv8m.main-none-eabihf` — the compiler's name for the target: the
//! Armv8-M Mainline instruction set of the RP2350's Cortex-M33 cores, no
//! operating system, hardware floating point — which is what makes
//! application logic written against these traits testable without hardware.
//!
//! ## Where this sits
//!
//! ```text
//!   api          traits + define_board!, zero register access   <- you are here
//!     ^
//!     |  implemented by
//!     |
//!   pico2        Cortex-M runtime + RP2350 register access and drivers
//!     ^
//!     |  linked against
//!     |
//!   application  names a concrete driver, then drives it via the traits
//! ```
//!
//! The arrow direction is the whole point: `api` never depends on `pico2`, so
//! a driver for a completely different chip can implement the same traits.
//! An application still has to name a concrete driver type somewhere — the
//! `demo` crate constructs `Rp2350Gpio` and asks the board for its `led`
//! pin — but past that point everything it does goes through these traits, so
//! those naming lines are the only ones that change when the chip does.
//!
//! ## The three modules, in reading order
//!
//! * [`common`] — the value-transfer vocabulary. [`common::ErrorType`] names
//!   the single error type a peripheral reports; [`common::Read`] and
//!   [`common::Write`] move values to and from a peripheral.
//! * [`device`] — pin ownership. A [`device::PinHandle<N>`](device::PinHandle)
//!   is a zero-sized value whose possession proves, at compile time, that
//!   physical pin `N` exists on this board and is not owned anywhere else;
//!   [`define_board!`] generates the board singleton that creates every
//!   handle exactly once.
//! * [`gpio`] — the first peripheral. [`gpio::Gpio`] is the factory trait
//!   that consumes a `PinHandle` and returns a configured input or output
//!   pin, typed by its pin number.
//!
//! ## Design notes
//!
//! Two conventions run through every trait here:
//!
//! * **Errors are an associated type, declared once per implementing type.**
//!   See [`common::ErrorType`] for why this is a separate supertrait rather
//!   than an associated type on each individual trait — both terms are
//!   defined there.
//! * **Traits are generic over the value type**, not the peripheral. A type
//!   that can be written as both a `bool` and a `u8` implements
//!   [`common::Write<bool>`](common::Write) and
//!   [`common::Write<u8>`](common::Write) rather than exposing two named
//!   methods.
#![no_std]

pub mod common;
pub mod gpio;
pub mod device;
