//! FFI types for OMF metadata.
//!
//! These wrap `serde_json::Value` into something usable from C. I've avoided using wrapping
//! a Rust enum here because tagged enums in C are a bit messy and easy to use incorrectly
//! by treating an f64 as a pointer and crashing.

use std::{ffi::c_char, ptr::null};

use crate::{
    error::Error,
    ffi_tools::arg::{slice, string_from_ptr, string_from_ptr_or_null},
};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Default)]
#[repr(i32)]
pub enum ValueType {
    #[default]
    Null,
    Boolean,
    Number,
    String,
    List,
    Object,
}

#[derive(Debug)]
#[repr(C)]
pub struct Value {
    pub name: *const c_char,
    pub r#type: ValueType,
    pub boolean: bool,
    pub number: f64,
    pub string: *const c_char,
    pub values: *const Value,
    pub n_values: usize,
}

impl Value {
    fn as_json_value(&self) -> Result<serde_json::Value, Error> {
        match self.r#type {
            ValueType::Null => Ok(serde_json::Value::Null),
            ValueType::Boolean => Ok(self.boolean.into()),
            ValueType::Number => Ok(self.number.into()),
            ValueType::String => {
                Ok(unsafe { string_from_ptr_or_null("metadata value", self.string) }?.into())
            }
            ValueType::Object => {
                Self::values_as_json_map(self.values, self.n_values).map(serde_json::Value::Object)
            }
            ValueType::List => {
                Self::values_as_json_vec(self.values, self.n_values).map(serde_json::Value::Array)
            }
        }
    }

    fn as_json_pair(&self) -> Result<(String, serde_json::Value), Error> {
        let name = unsafe { string_from_ptr("value.name", self.name) }?;
        Ok((name, self.as_json_value()?))
    }

    fn values_as_json_vec(
        values: *const Value,
        n_values: usize,
    ) -> Result<Vec<serde_json::Value>, Error> {
        slice!(values, n_values)?
            .iter()
            .map(Self::as_json_value)
            .collect()
    }

    pub fn values_as_json_map(
        values: *const Value,
        n_values: usize,
    ) -> Result<serde_json::Map<String, serde_json::Value>, Error> {
        slice!(values, n_values)?
            .iter()
            .map(Self::as_json_pair)
            .collect()
    }
}

impl Default for Value {
    fn default() -> Self {
        Self {
            name: null(),
            r#type: Default::default(),
            boolean: false,
            number: 0.0,
            string: null(),
            n_values: 0,
            values: null(),
        }
    }
}
