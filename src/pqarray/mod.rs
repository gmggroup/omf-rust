mod array_type;
pub mod read;
pub mod schema;
mod source;
#[cfg(test)]
mod tests;
mod write;

pub(crate) use array_type::PqArrayType;
pub(crate) use read::PqArrayReader;
pub(crate) use schema::{
    PqArrayMatcher, schema, schema_field, schema_fields, schema_logical_type, schema_match,
    schema_physical_type, schema_repetition,
};
pub(crate) use write::{PqArrayWriter, PqWriteOptions};
