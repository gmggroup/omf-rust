//! Python reader and writer for Open Mining Format version 2,
//! a standard for mining data interchange backed by the
//! [Global Mining Guidelines Group](https://gmggroup.org).
//!
//! > **Python bindings are incomplete:**
//! > The ability to read OMF2 files is present, and to convert OMF1 files to OMF2,
//! > but not the ability to write files. The composite element is also not yet supported.
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
//! The [Reader](crate::file::reader::PyReader) object is the starting points for reading files.
//! [Project](crate::project::PyProject) is the root object of the data contained within the file,
//! storing a list of [elements](crate::element::PyElement),
//! each containing some geometry and a list of attributes.
//!
//! Supported element geometries are:
//!
//! - [Points](crate::geometry::PyPointSet).
//! - [Line segments](crate::geometry::PyLineSet).
//! - [Triangulated surfaces](crate::geometry::PySurface).
//! - [Grid surfaces](crate::geometry::PyGridSurface).
//!     - [Regular](crate::grid::PyGrid2Regular) or [tensor](crate::grid::PyGrid2Tensor).
//!     - Any [orientation](crate::grid::PyOrient2).
//! - [Block models](crate::block_model::PyBlockModel), with optional sub-blocks.
//!     - [Regular](crate::grid::PyGrid3Regular) or [tensor](crate::grid::PyGrid3Tensor).
//!     - Any [orientation](crate::grid::PyOrient3).
//!     - [Regular sub-blocks](crate::block_model::PyRegularSubblocks) that lie on a grid within
//!       their parent, with octree or arbitrary layout.
//!     - [Free-form sub-blocks](crate::block_model::PyFreeformSubblocks) that don't lie on any grid.
//! - Composite elements made out of any of the above. **Not yet supported under Python.**
//!
//! Supported attribute data types are:
//!
//! - [Floating-point or signed integer](crate::attribute::PyAttributeDataNumber) values,
//!   including date and date-time values.
//! - [Category](crate::attribute::PyAttributeDataCategory) values,
//!   storing an index used to look up name, color, or other sub-attributes.
//! - [Boolean](crate::attribute::PyAttributeDataBoolean) or filter values.
//! - 2D and 3D [vector](crate::attribute::PyAttributeDataVector) values.
//! - [Text](crate::attribute::PyAttributeDataText) values.
//! - [Color](crate::attribute::PyAttributeDataColor) values.
//! - [Projected texture](crate::attribute::PyAttributeDataProjectedTexture) images.
//! - [UV mapped texture](crate::attribute::PyAttributeDataMappedTexture) images.
//!
//! Attributes values can be valid or null.
//! They can be attached to different [parts](crate::attribute::PyLocation) of each element type,
//! such as the vertices vs. faces of a surface,
//! or the parent blocks vs. sub-blocks of a block model.

use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};

mod array;
mod attribute;
mod block_model;
mod colormap;
mod element;
mod errors;
mod file;
mod geometry;
mod grid;
mod omf1;
mod project;
mod validate;

/// Returns the version of the library
#[gen_stub_pyfunction]
#[pyfunction]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// This module provides python bindings for omf-rust.
#[pymodule]
fn omf2(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<array::PyBooleanArray>()?;
    m.add_class::<array::PyBoundaryArray>()?;
    m.add_class::<array::PyColorArray>()?;
    m.add_class::<array::PyFreeformSubblockArray>()?;
    m.add_class::<array::PyGradientArray>()?;
    m.add_class::<array::PyImageArray>()?;
    m.add_class::<array::PyIndexArray>()?;
    m.add_class::<array::PyNameArray>()?;
    m.add_class::<array::PyNumberArray>()?;
    m.add_class::<array::PyRegularSubblockArray>()?;
    m.add_class::<array::PyScalarArray>()?;
    m.add_class::<array::PySegmentArray>()?;
    m.add_class::<array::PyTexcoordArray>()?;
    m.add_class::<array::PyTextArray>()?;
    m.add_class::<array::PyTriangleArray>()?;
    m.add_class::<array::PyVectorArray>()?;
    m.add_class::<array::PyVertexArray>()?;
    m.add_class::<attribute::PyAttribute>()?;
    m.add_class::<attribute::PyAttributeDataBoolean>()?;
    m.add_class::<attribute::PyAttributeDataCategory>()?;
    m.add_class::<attribute::PyAttributeDataColor>()?;
    m.add_class::<attribute::PyAttributeDataMappedTexture>()?;
    m.add_class::<attribute::PyAttributeDataNumber>()?;
    m.add_class::<attribute::PyAttributeDataProjectedTexture>()?;
    m.add_class::<attribute::PyAttributeDataText>()?;
    m.add_class::<attribute::PyAttributeDataVector>()?;
    m.add_class::<attribute::PyLocation>()?;
    m.add_class::<block_model::PyBlockModel>()?;
    m.add_class::<block_model::PyFreeformSubblocks>()?;
    m.add_class::<block_model::PyRegularSubblocks>()?;
    m.add_class::<block_model::PySubblockMode>()?;
    m.add_class::<colormap::PyNumberColormapContinuous>()?;
    m.add_class::<colormap::PyNumberColormapDiscrete>()?;
    m.add_class::<element::PyElement>()?;
    m.add_class::<file::reader::PyBoundaryType>()?;
    m.add_class::<file::reader::PyLimits>()?;
    m.add_class::<file::reader::PyReader>()?;
    m.add_class::<geometry::PyGridSurface>()?;
    m.add_class::<geometry::PyLineSet>()?;
    m.add_class::<geometry::PyPointSet>()?;
    m.add_class::<geometry::PySurface>()?;
    m.add_class::<grid::PyGrid2Regular>()?;
    m.add_class::<grid::PyGrid2Tensor>()?;
    m.add_class::<grid::PyGrid3Regular>()?;
    m.add_class::<grid::PyGrid3Tensor>()?;
    m.add_class::<grid::PyOrient2>()?;
    m.add_class::<grid::PyOrient3>()?;
    m.add_class::<omf1::converter::PyOmf1Converter>()?;
    m.add_class::<project::PyProject>()?;
    m.add_class::<validate::PyProblem>()?;

    m.add_function(wrap_pyfunction!(omf1::converter::detect_omf1, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;

    m.add("OmfException", py.get_type::<errors::OmfException>())?;
    m.add(
        "OmfFileIoException",
        py.get_type::<errors::OmfFileIoException>(),
    )?;
    m.add(
        "OmfInvalidDataException",
        py.get_type::<errors::OmfInvalidDataException>(),
    )?;
    m.add(
        "OmfJsonException",
        py.get_type::<errors::OmfJsonException>(),
    )?;
    m.add(
        "OmfLimitExceededException",
        py.get_type::<errors::OmfLimitExceededException>(),
    )?;
    m.add(
        "OmfNotSupportedException",
        py.get_type::<errors::OmfNotSupportedException>(),
    )?;
    m.add(
        "OmfValidationFailedException",
        py.get_type::<errors::OmfValidationFailedException>(),
    )?;

    Ok(())
}

define_stub_info_gatherer!(stub_info);
