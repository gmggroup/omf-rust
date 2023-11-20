use std::{ffi::c_char, io::Write, ptr::null};

use crate::ffi_tools::{into_ffi_free, FfiConvert, FfiStorage, IntoFfi};

pub fn handle_validation(problems: &omf::validate::Problems, validation: *mut *mut Validation) {
    if !problems.is_empty() {
        if validation.is_null() {
            let mut stdout = std::io::stdout().lock();
            _ = writeln!(stdout, "{problems}");
            _ = stdout.flush();
        } else {
            let strings: Vec<_> = problems.iter().map(|p| p.to_string()).collect();
            unsafe { validation.write(strings.into_ffi()) }
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Validation {
    pub n_messages: usize,
    pub messages: *const *const c_char,
}

impl FfiConvert<Vec<String>> for Validation {
    fn convert(strings: Vec<String>, storage: &mut FfiStorage) -> Self {
        let n_messages = strings.len();
        let ptrs = strings
            .into_iter()
            .map(|s| storage.keep_string(s))
            .chain(Some(null()))
            .collect();
        Self {
            n_messages,
            messages: storage.keep_vec(ptrs),
        }
    }
}

#[no_mangle]
pub extern "C" fn omf_validation_free(ptr: *mut Validation) -> bool {
    unsafe { into_ffi_free(ptr) }
}
