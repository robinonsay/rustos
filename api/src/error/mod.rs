//! An extensible, chip-independent error vocabulary.
//!
//! Every driver and application has its own error type, shaped by what can
//! go wrong in *it*: a UART reports framing errors, an I2C bus a missing
//! acknowledge. Code that only needs to know *what sort* of thing failed —
//! a scheduler logging an application's failure, a health monitor counting
//! timeouts — should not have to know every one of those types.
//!
//! So this module splits an error in two:
//!
//! * the **type** stays whatever its owner defines, with all its detail, and
//!   is printed with [`Debug`](core::fmt::Debug);
//! * its **kind**, an [`ErrorKind`], is a small shared classification that
//!   any code can match on.
//!
//! A type joins in by implementing [`Error`], whose one method returns the
//! kind. That is the extension point: new error types need no change here.
//! [`ErrorKind`] is `#[non_exhaustive]`, so kinds can be added later
//! without breaking code that matches on it (such code must have a `_`
//! arm).
//!
//! The kinds follow LibJuno's status codes (`juno/status.h`) where one
//! exists, so a LibJuno port maps one to one:
//!
//! | `ErrorKind` | LibJuno |
//! |-------------|---------|
//! | [`InvalidData`](ErrorKind::InvalidData) | `JUNO_STATUS_INVALID_DATA_ERROR` |
//! | [`Read`](ErrorKind::Read) | `JUNO_STATUS_READ_ERROR` |
//! | [`Write`](ErrorKind::Write) | `JUNO_STATUS_WRITE_ERROR` |
//! | [`Timeout`](ErrorKind::Timeout) | `JUNO_STATUS_TIMEOUT_ERROR` |
//! | [`OutOfRange`](ErrorKind::OutOfRange) | `JUNO_STATUS_OOB_ERROR` |
//! | [`Full`](ErrorKind::Full) | `JUNO_STATUS_TABLE_FULL_ERROR` |
//! | [`NotFound`](ErrorKind::NotFound) | `JUNO_STATUS_DNE_ERROR` |
//!
//! LibJuno's `JUNO_STATUS_NULLPTR_ERROR` has no counterpart: Rust
//! references cannot be null.
//!
//! ```ignore
//! use api::error::{Error, ErrorKind};
//!
//! #[derive(Debug)]
//! enum GpsError { NoFix, BadChecksum }
//!
//! impl Error for GpsError {
//!     fn kind(&self) -> ErrorKind {
//!         match self {
//!             GpsError::NoFix => ErrorKind::NotReady,
//!             GpsError::BadChecksum => ErrorKind::InvalidData,
//!         }
//!     }
//! }
//! ```

/// What sort of thing went wrong, independent of which type reported it.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// An argument was not acceptable: out of the documented range, the
    /// wrong length, an unusable address. Retrying with the same input
    /// fails again.
    InvalidInput,
    /// Data received or read was malformed or failed a check (checksum,
    /// parity, framing). `JUNO_STATUS_INVALID_DATA_ERROR`.
    InvalidData,
    /// An index or value fell outside what a fixed resource holds.
    /// `JUNO_STATUS_OOB_ERROR`.
    OutOfRange,
    /// A fixed-capacity table, queue or buffer had no room.
    /// `JUNO_STATUS_TABLE_FULL_ERROR`.
    Full,
    /// The thing asked for does not exist, or has not been produced yet.
    /// `JUNO_STATUS_DNE_ERROR`.
    NotFound,
    /// Something did not happen within the time allowed.
    /// `JUNO_STATUS_TIMEOUT_ERROR`.
    Timeout,
    /// The operation would have had to wait, and this call never waits.
    /// Try again later.
    WouldBlock,
    /// The resource is in use by someone else.
    Busy,
    /// The resource exists but is not ready yet (not configured, not
    /// started, no fix yet).
    NotReady,
    /// The operation is not supported by this implementation.
    Unsupported,
    /// Data was lost because it arrived faster than it was taken, or work
    /// ran past its deadline.
    Overrun,
    /// A read from a device or bus failed. `JUNO_STATUS_READ_ERROR`.
    Read,
    /// A write to a device or bus failed. `JUNO_STATUS_WRITE_ERROR`.
    Write,
    /// The hardware reported a fault (a bus error, a halted controller).
    Hardware,
    /// None of the above. The error's own type and `Debug` output carry
    /// the detail.
    Other,
}

/// An error that can say what [`ErrorKind`] it is.
///
/// Implement this for any error type, in any crate. `Debug` is required so
/// that code holding only a `&dyn Error` can still log the full detail.
pub trait Error: core::fmt::Debug {
    /// The kind of this error.
    fn kind(&self) -> ErrorKind;
}

/// An [`ErrorKind`] is itself an error of that kind, for code with nothing
/// more specific to say.
impl Error for ErrorKind {
    fn kind(&self) -> ErrorKind {
        // It is its own classification.
        *self
    }
}

/// `Infallible` has no values, so this is never called; it lets operations
/// that cannot fail satisfy an `Error` bound.
impl Error for core::convert::Infallible {
    fn kind(&self) -> ErrorKind {
        // An empty match: there is no value to classify.
        match *self {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    enum Mine {
        Late,
        Garbled,
    }

    impl Error for Mine {
        fn kind(&self) -> ErrorKind {
            match self {
                Mine::Late => ErrorKind::Timeout,
                Mine::Garbled => ErrorKind::InvalidData,
            }
        }
    }

    #[test]
    fn a_new_type_classifies_itself() {
        assert_eq!(Mine::Late.kind(), ErrorKind::Timeout);
        assert_eq!(Mine::Garbled.kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn errors_work_through_a_trait_object() {
        let e: &dyn Error = &Mine::Late;
        assert_eq!(e.kind(), ErrorKind::Timeout);
    }

    #[test]
    fn a_kind_is_an_error_of_that_kind() {
        assert_eq!(ErrorKind::Full.kind(), ErrorKind::Full);
    }
}
