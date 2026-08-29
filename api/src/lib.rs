//! # `api` — portable hardware-abstraction traits
//!
//! This crate is the **portability boundary** of the system. It contains
//! traits and nothing else: no register addresses, no `unsafe`, no assumptions
//! about a CPU architecture, and no dependency on any other crate in the
//! workspace. Everything here compiles for the host as readily as it does for
//! `thumbv8m.main-none-eabihf`, which is what makes application logic written
//! against these traits testable without hardware.
//!
//! ## Where this sits
//!
//! ```text
//!   api          traits only, zero hardware        <- you are here
//!     ^
//!     |  implemented by
//!     |
//!   pico2        RP2350 register access + drivers, board wiring
//!     ^
//!     |  linked against
//!     |
//!   application  user code, generic over the traits
//! ```
//!
//! The arrow direction is the whole point: `api` never depends on `pico2`, so
//! a driver for a completely different chip can implement the same traits and
//! application code moves across unchanged.
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
