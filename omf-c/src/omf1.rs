use std::{ffi::c_char, path::PathBuf, ptr::null_mut, sync::Mutex};

use crate::{
    elements::Limits,
    ffi_tools::{
        arg::{not_null, not_null_mut, string_not_null},
        catch,
    },
    validation::{handle_validation, Validation},
};

#[derive(Debug, Default)]
pub struct Omf1Converter(Mutex<omf::omf1::Converter>);

macro_rules! inner {
    ($converter:ident) => {
        not_null_mut!($converter)?.0.lock().expect("intact lock")
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_omf1_detect(path: *const c_char) -> bool {
    catch::error(|| {
        let path = PathBuf::from(string_not_null!(path)?);
        omf::omf1::detect_open(&path)?;
        Ok(())
    })
    .is_some()
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_omf1_converter_new() -> *mut Omf1Converter {
    catch::error(|| Ok(Box::into_raw(Default::default()))).unwrap_or_else(null_mut)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_omf1_converter_free(converter: *mut Omf1Converter) -> bool {
    catch::error(|| {
        if !converter.is_null() {
            unsafe {
                _ = Box::from_raw(converter);
            }
        }
        Ok(())
    })
    .is_some()
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_omf1_converter_compression(converter: *mut Omf1Converter) -> i32 {
    catch::error(|| Ok(inner!(converter).compression().level() as i32)).unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_omf1_converter_set_compression(
    converter: *mut Omf1Converter,
    compression: i32,
) -> bool {
    catch::error(|| {
        inner!(converter).set_compression(if compression == -1 {
            omf::file::Compression::default()
        } else {
            omf::file::Compression::new(compression.clamp(0, 9) as u32)
        });
        Ok(())
    })
    .is_some()
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_omf1_converter_limits(converter: *mut Omf1Converter) -> Limits {
    catch::error(|| {
        let limits = if converter.is_null() {
            Default::default()
        } else {
            inner!(converter).limits()
        };
        Ok(limits.into())
    })
    .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_omf1_converter_set_limits(
    converter: *mut Omf1Converter,
    limits: *const Limits,
) -> bool {
    catch::error(|| {
        let limits = not_null!(limits)?;
        inner!(converter).set_limits((*limits).into());
        Ok(())
    })
    .is_some()
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_omf1_converter_convert(
    converter: *mut Omf1Converter,
    input_path: *const c_char,
    output_path: *const c_char,
    validation: *mut *mut Validation,
) -> bool {
    catch::error(|| {
        let converter = inner!(converter);
        let input_path = PathBuf::from(string_not_null!(input_path)?);
        let output_path = PathBuf::from(string_not_null!(output_path)?);
        match converter.convert_open(input_path, output_path) {
            Ok(warnings) => {
                handle_validation(&warnings, validation);
                Ok(())
            }
            Err(omf::error::Error::ValidationFailed(problems)) => {
                handle_validation(&problems, validation);
                Err(omf::error::Error::ValidationFailed(problems).into())
            }
            Err(e) => Err(e.into()),
        }
    })
    .is_some()
}
