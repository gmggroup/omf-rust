//! Panic and result error catching code.

use std::panic::UnwindSafe;

use crate::error::{Error, set_error, set_panic};

/// Catches both panics and `Result<T, omf::Error>` results in a function.
///
/// Returns an `Option<T>` that is `None` when an error occurs, calling code should use
/// `unwrap_or` or similar for the error return value.
pub fn error<T, F>(func: F) -> Option<T>
where
    F: FnOnce() -> Result<T, Error> + UnwindSafe,
{
    match std::panic::catch_unwind(func) {
        Ok(Ok(value)) => Some(value),
        Ok(Err(error)) => {
            set_error(error);
            None
        }
        Err(payload) => {
            set_panic(payload);
            None
        }
    }
}

/// Catches just panics in a function.
///
/// Returns an `Option<T>` that is `None` when an error occurs, calling code should use
/// `unwrap_or` or similar for the error return value.
pub fn panic<T>(func: impl FnOnce() -> T + UnwindSafe) -> Option<T> {
    match std::panic::catch_unwind(func) {
        Ok(value) => Some(value),
        Err(payload) => {
            set_panic(payload);
            None
        }
    }
}

/// Catches just panics in a function, returning a boolean.
pub fn panic_bool(func: impl FnOnce() + UnwindSafe) -> bool {
    panic(|| {
        func();
        true
    })
    .unwrap_or(false)
}

/// Catches `Result<T, omf::Error>` results in a function.
///
/// Returns an `Option<T>` that is `None` when an error occurs, calling code should use
/// `unwrap_or` or similar for the error return value.
pub fn error_only<T, F>(func: F) -> Option<T>
where
    F: FnOnce() -> Result<T, Error>,
{
    match func() {
        Ok(value) => Some(value),
        Err(error) => {
            set_error(error);
            None
        }
    }
}
