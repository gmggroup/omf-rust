use std::{any::Any, cell::RefCell, ffi::c_char, ptr::null_mut};

use crate::ffi_tools::{FfiConvert, FfiStorage, IntoFfi, into_ffi_free};

use super::Error;

thread_local! {
    static ERROR_STATE: RefCell<Option<Error>> = Default::default();
}

/// Sets the error state for the current thread, if it is not already set.
pub fn set_error(error: Error) {
    ERROR_STATE.with(|cell| {
        cell.borrow_mut().get_or_insert(error);
    })
}

/// Sets the error state from a panic payload, if it is not already set.
pub fn set_panic(payload: Box<dyn Any + Send>) {
    let message = if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_owned()
    } else {
        "unknown payload".to_owned()
    };
    set_error(Error::Panic(message));
}

#[derive(Debug)]
#[repr(C)]
pub struct CError {
    /// Status code.
    pub code: i32,
    /// Error detail. The meaning depends on the status code.
    pub detail: i32,
    /// A nul-terminated string containing a human-readable error message in English.
    pub message: *const c_char,
}

impl FfiConvert<Error> for CError {
    fn convert(error: Error, storage: &mut FfiStorage) -> Self {
        Self {
            code: error.code(),
            detail: error.detail(),
            message: storage.keep_string(error.to_string()),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_error() -> *mut CError {
    match ERROR_STATE.with(|cell| cell.take()) {
        Some(error) => error.into_ffi(),
        None => null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_error_clear() {
    ERROR_STATE.with(|cell| cell.take());
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_error_peek() -> i32 {
    ERROR_STATE
        .with(|cell| cell.borrow().as_ref().map(Error::code))
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_error_free(error: *mut CError) {
    unsafe { into_ffi_free(error) };
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use crate::ffi_tools::catch;

    use super::*;

    #[test]
    fn test_io_error() {
        catch::error(|| -> Result<(), Error> {
            let err = std::fs::read_to_string("does_not_exist.txt").unwrap_err();
            Err(omf::error::Error::IoError(err).into())
        });
        let c_err = omf_error();
        let message = unsafe { CStr::from_ptr((*c_err).message) }
            .to_str()
            .unwrap();
        // Different OS will give different error messages, only check start and end.
        assert!(
            message.starts_with("File IO error: ") && message.ends_with(" (os error 2)"),
            "Got: {message}\nExpected: File IO error: .* (os error 2)"
        );
        assert!(omf_error().is_null());
        omf_error_free(c_err);
    }
}
