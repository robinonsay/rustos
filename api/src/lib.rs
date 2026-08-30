//! # `api` — portable hardware-abstraction traits
//!
//! This crate is the **portability boundary** of the system. It contains the
//! trait definitions the rest of the workspace is written against, plus one
//! concrete type: [`common::board::Board`], the singleton that starts every
//! peripheral registered with it, exactly once per boot. There are no
//! register addresses and no dependency on any other crate in the workspace.
//! The only `unsafe` here is the [`common::Block`] lifecycle: `start` and
//! `stop` are `unsafe fn`s (their doc comments say why), and `Board::take`
//! contains the one `unsafe` call that invokes `start`. Everything here
//! compiles for the host as readily as it does for
//! `thumbv8m.main-none-eabihf`, which is what makes application logic written
//! against these traits testable without hardware.
//!
//! ## Where this sits
//!
//! ```text
//!   api          traits + the Board singleton, zero register access   <- you are here
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
//! `demo` crate constructs `Rp2350Gpio` and hard-codes pin 25 — but past that
//! point everything it does goes through these traits, so those naming lines
//! are the only ones that change when the chip does.
//!
//! ## Design notes
//!
//! Two conventions run through every trait here:
//!
//! * **Errors are an associated type, declared once per implementing type.**
//!   See [`common::ErrorType`] for why this is a separate supertrait rather
//!   than an associated type on each individual trait.
//! * **Traits are generic over the value type**, not the peripheral. A type
//!   that can be written as both a `bool` and a `u8` implements
//!   [`common::Write<bool>`](common::Write) and
//!   [`common::Write<u8>`](common::Write) rather than exposing two named
//!   methods.
#![no_std]

pub mod common;
pub mod gpio;
pub mod device;
