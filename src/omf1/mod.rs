//! Convert existing OMF v1 files to OMF v2.
//!
//! ## Conversion details
//!
//! There are a few parts of OMF1 that don't map directly to OMF2.
//!
//! ### Elements
//!
//! - The `date_created` and `date_modified` fields are moved into the metadata.
//! - The `subtype` field on point-sets and line-sets is moved into the metadata.
//!   On other elements, where it only had one valid value, it is discarded.
//! - Line-sets and surfaces with invalid vertex indices will cause conversion to fail.
//! - Line-sets and surfaces with more than 4,294,967,295 vertices will cause conversion to fail.
//!
//! ### Data to Attributes
//!
//! - Scalar data becomes a number attribute, preserving the float/int type of the array.
//! - In number data, NaN becomes null.
//! - In 2D or 3D vector data, if any component is NaN the vector becomes null.
//! - In string data, empty strings become nulls.
//!   OMF2 supports both null and empty string so we can only guess which was intended.
//! - In date-time data, empty strings become null.
//! - Date-times outside the range of approximately ±262,000 years CE will cause conversion to fail.
//!
//! ### Mapped Data to Category Attribute
//!
//! The exact layout of mapped data from OMF v1 can't be stored in OMF v2.
//! It is transformed to a category attribute by following these rules:
//!
//! - Indices equal to minus one become null.
//! - Indices outside the range 0 to 4,294,967,295 will cause conversion to fail.
//! - The most unique, least empty, and shortest string legend becomes the category names,
//!   padded with empty strings if necessary.
//! - The most unique and least empty color legend becomes the category colors, padded with
//!   gray if necessary.
//! - Other legends become extra attributes, padded with nulls if necessary.

mod array;
mod attributes;
mod category_handler;
mod converter;
mod elements;
mod error;
mod model;
mod objects;
mod reader;

#[cfg(not(target_family = "wasm"))]
pub use converter::detect_open;
pub use converter::{detect, Converter};
pub use error::Omf1Error;
pub use model::ModelType;
