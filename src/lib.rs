//! Reader and writer for Open Mining Format version 2,
//! a standard for mining data interchange backed by the
//! [Global Mining Guidelines Group](https://gmggroup.org).
//!
//! > **Warning:**
//! > This is an alpha release of OMF 2. The storage format and libraries might be changed in
//! > backward-incompatible ways and are not subject to any SLA or deprecation policy.
//! > Further, this code is unfinished and may not be secure.
//! > Don't use it to open files you don't trust, and don't use it in production yet.
//!
//! # What is OMF
//!
//! OMF is an open-source serialization format and library to support data interchange
//! across the entire mining community.
//! Its goal is to standardize file formats and promote collaboration.
//!
//! This repository provides a file format specification and a Rust library for reading and writing files,
//! plus wrappers to use that library from C and Python.
//!
//! # Getting Started
//!
//! The [Reader](crate::file::Reader) and [Writer](crate::file::Writer)
//! objects are the starting points for reading and writing files.
//! [Error](crate::error::Error) is the combined error type for everything.
//! [Project] is the root object of the data contained within the file,
//! storing a list of [elements](crate::Element),
//! each containing some [geometry](crate::Geometry) and a list of [attributes](crate::Attribute).
//!
//! Supported element geometries are:
//!
//! - [Points](crate::PointSet).
//! - [Line segments](crate::LineSet).
//! - [Triangulated surfaces](crate::Surface).
//! - [Grid surfaces](crate::GridSurface).
//!     - Regular or tensor [grid spacing](crate::Grid2).
//!     - Any [orientation](crate::Orient2).
//! - [Block models](crate::BlockModel), with optional [sub-blocks](crate::Subblocks).
//!     - Regular or tensor [grid spacing](crate::Grid3).
//!     - Any [orientation](crate::Orient3).
//!     - Regular sub-blocks that lie on a grid within their parent, with octree or arbitrary layout.
//!     - Free-form sub-blocks that don't lie on any grid.
//! - [Composite] elements made out of any of the above.
//!
//! Supported attribute data types are:
//!
//! - [Floating-point or signed integer](crate::AttributeData::Number) values,
//!   including date and date-time values.
//! - [Category](crate::AttributeData::Category) values,
//!   storing an index used to look up name, color, or other sub-attributes.
//! - [Boolean](crate::AttributeData::Boolean) or filter values.
//! - 2D and 3D [vector](crate::AttributeData::Vector) values.
//! - [Text](crate::AttributeData::Text) values.
//! - [Color](crate::AttributeData::Color) values.
//! - [Projected texture](crate::AttributeData::ProjectedTexture) images.
//! - [UV mapped texture](crate::AttributeData::MappedTexture) images.
//!
//! Attributes values can be valid or null.
//! They can be attached to different [parts](crate::Location) of each element type,
//! such as the vertices vs. faces of a surface,
//! or the parent blocks vs. sub-blocks of a block model.

#![deny(unsafe_code)]

mod array;
mod attribute;
mod block_model;
mod colormap;
#[cfg(feature = "parquet")]
pub mod data;
pub mod date_time;
mod element;
pub mod error;
pub mod file;
mod geometry;
mod grid;
#[cfg(feature = "omf1")]
pub mod omf1;
#[cfg(feature = "parquet")]
mod pqarray;
mod project;
mod schema;
#[cfg(test)]
mod schema_doc;
pub mod validate;
mod version;

pub use array::{Array, ArrayType, DataType, array_type};
pub use attribute::{Attribute, AttributeData, Location};
pub use block_model::{BlockModel, SubblockMode, Subblocks};
pub use colormap::{NumberColormap, NumberRange};
pub use element::Element;
pub use geometry::{Composite, Geometry, GridSurface, LineSet, PointSet, Surface};
pub use grid::{Grid2, Grid3, Orient2, Orient3};
pub use project::Project;
pub use schema::json_schema;
pub use version::{
    CRATE_NAME, CRATE_VERSION, FORMAT_EXTENSION, FORMAT_NAME, FORMAT_VERSION_MAJOR,
    FORMAT_VERSION_MINOR, FORMAT_VERSION_PRERELEASE, crate_full_name, format_full_name,
    format_version,
};

/// A 3D vector with `f64` components.
pub type Vector3 = [f64; 3];

/// RGBA color with components from 0 to 255.
pub type Color = [u8; 4];
