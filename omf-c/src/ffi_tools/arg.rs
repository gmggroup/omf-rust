use std::ffi::{CStr, c_char};

use crate::error::{Error, InvalidArg};

/// Get a reference from a pointer or an error on null.
///
/// # Safety
///
/// Pointer must be null or valid. See `core::ptr::const_ptr::as_ref` for details.
pub unsafe fn ref_from_ptr<'a, T>(arg_name: &'static str, ptr: *const T) -> Result<&'a T, Error> {
    unsafe { ptr.as_ref() }.ok_or(InvalidArg::Null(arg_name).into())
}

/// Get a mutable reference from a pointer or an error on null.
///
/// # Safety
///
/// Pointer must be null or valid. See `core::ptr::const_ptr::as_mut` for details.
pub unsafe fn mut_from_ptr<'a, T>(arg_name: &'static str, ptr: *mut T) -> Result<&'a mut T, Error> {
    unsafe { ptr.as_mut() }.ok_or(InvalidArg::Null(arg_name).into())
}

/// Consumes a pointer, returning the contained value.
///
/// # Safety
///
/// Pointer must be null or valid.
pub unsafe fn consume_ptr<T>(arg_name: &'static str, ptr: *mut T) -> Result<T, Error> {
    if ptr.is_null() {
        Err(InvalidArg::Null(arg_name).into())
    } else {
        Ok(*unsafe { Box::from_raw(ptr) })
    }
}

/// Copies from a C `const char*` returning an error if the pointer is null, or an error
/// if the string is null or not UTF-8 encoded.
///
/// # Safety
///
/// `ptr` must point to a valid nul-terminated string. See `CStr::from_ptr` for details.
pub unsafe fn string_from_ptr(src: &'static str, ptr: *const c_char) -> Result<String, Error> {
    if ptr.is_null() {
        Err(InvalidArg::Null(src).into())
    } else {
        Ok(unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .map_err(|_| InvalidArg::NotUtf8(src))?
            .to_owned())
    }
}

/// Copies from a C `const char*` returning an error if the pointer is null, or if the
/// string is not UTF-8 encoded.
///
/// If `ptr` is null then an empty string is returned.
///
/// # Safety
///
/// `ptr` must be a valid string. See `CStr::from_ptr` for details.
pub unsafe fn string_from_ptr_or_null(
    src: &'static str,
    ptr: *const c_char,
) -> Result<String, Error> {
    if ptr.is_null() {
        Ok(String::new())
    } else {
        Ok(unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .map_err(|_| InvalidArg::NotUtf8(src))?
            .to_owned())
    }
}

/// Creates a slice from a pointer without copying, returning an error if the pointer is
/// null and the size isn't zero.
///
/// # Safety
///
/// `ptr` must point to at least `n` elements. See `std::slice::from_raw_parts` for details.
pub unsafe fn slice_from_ptr<T>(
    src: &'static str,
    src_n: &'static str,
    ptr: *const T,
    n: usize,
) -> Result<&'static [T], Error> {
    if !ptr.is_null() {
        Ok(unsafe { std::slice::from_raw_parts(ptr, n) })
    } else if n == 0 {
        Ok(&[])
    } else {
        Err(InvalidArg::NullArray(src, src_n).into())
    }
}

/// Creates a slice from a pointer without copying, returning an error if the pointer is
/// null and the size isn't zero.
///
/// # Safety
///
/// `ptr` must point to at least `n` elements. See `std::slice::from_raw_parts` for details.
pub unsafe fn slice_mut_from_ptr<T>(
    src: &'static str,
    src_n: &'static str,
    ptr: *mut T,
    n: usize,
) -> Result<&'static mut [T], Error> {
    if !ptr.is_null() {
        Ok(unsafe { std::slice::from_raw_parts_mut(ptr, n) })
    } else if n == 0 {
        Ok(&mut [])
    } else {
        Err(InvalidArg::NullArray(src, src_n).into())
    }
}

/// Like `slice_mut_from_ptr` but also checks the length.
pub unsafe fn slice_mut_from_ptr_len<T>(
    src: &'static str,
    src_n: &'static str,
    ptr: *mut T,
    n: usize,
    len: u64,
) -> Result<&'static mut [T], Error> {
    if n as u64 != len {
        Err(Error::BufferLengthWrong {
            found: n as u64,
            expected: len,
        })
    } else if !ptr.is_null() {
        Ok(unsafe { std::slice::from_raw_parts_mut(ptr, n) })
    } else if n == 0 {
        Ok(&mut [])
    } else {
        Err(InvalidArg::NullArray(src, src_n).into())
    }
}

/// Call `ref_from_ptr` using the name of the argument.
macro_rules! not_null (
    ($name:expr) => { unsafe { crate::ffi_tools::arg::ref_from_ptr(stringify!($name), $name) } };
);

/// Call `ref_from_ptr` using the name of the argument.
macro_rules! not_null_mut (
    ($name:expr) => { unsafe { crate::ffi_tools::arg::mut_from_ptr(stringify!($name), $name) } };
);

/// Call `consume_ptr` using the name of the argument.
macro_rules! not_null_consume (
    ($name:expr) => { unsafe { crate::ffi_tools::arg::consume_ptr(stringify!($name), $name) } };
);

/// Call `string_from_ptr` using the name of the argument.
macro_rules! string_not_null {
    ($name:expr) => {
        unsafe { crate::ffi_tools::arg::string_from_ptr(stringify!($name), $name) }
    };
}

/// Call `slice_from_ptr` using the names of the arguments.
macro_rules! slice {
    ($name:expr, $count:expr) => {
        unsafe {
            crate::ffi_tools::arg::slice_from_ptr(
                stringify!($name),
                stringify!($count),
                $name,
                $count,
            )
        }
    };
}

/// Call `slice_mut_from_ptr` using the names of the arguments.
macro_rules! slice_mut {
    ($name:expr, $count:expr) => {
        unsafe {
            crate::ffi_tools::arg::slice_mut_from_ptr(
                stringify!($name),
                stringify!($count),
                $name,
                $count,
            )
        }
    };
}

/// Call `slice_mut_from_ptr` using the names of the arguments.
macro_rules! slice_mut_len {
    ($name:expr, $count:expr, $len:expr) => {
        unsafe {
            crate::ffi_tools::arg::slice_mut_from_ptr_len(
                stringify!($name),
                stringify!($count),
                $name,
                $count,
                $len,
            )
        }
    };
}

pub(crate) use {
    not_null, not_null_consume, not_null_mut, slice, slice_mut, slice_mut_len, string_not_null,
};
