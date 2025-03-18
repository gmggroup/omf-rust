//! Python reader and writer for Open Mining Format version 2,
//! a standard for mining data interchange backed by the
//! [Global Mining Guidelines Group](https://gmggroup.org).
//!
//! > **The Python bindings are incomplete:**
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

use block_model::{PyBlockModel, PyFreeformSubblocks, PyRegularSubblocks, PySubblockMode};
use grid::{PyGrid2Regular, PyGrid2Tensor, PyGrid3Regular, PyGrid3Tensor, PyOrient2, PyOrient3};
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

use array::{
    PyBooleanArray, PyBoundaryArray, PyColorArray, PyFreeformSubblockArray, PyGradientArray,
    PyImageArray, PyIndexArray, PyNameArray, PyNumberArray, PyRegularSubblockArray, PyScalarArray,
    PySegmentArray, PyTexcoordArray, PyTextArray, PyTriangleArray, PyVectorArray, PyVertexArray,
};
use attribute::{
    PyAttribute, PyAttributeDataBoolean, PyAttributeDataCategory, PyAttributeDataColor,
    PyAttributeDataMappedTexture, PyAttributeDataNumber, PyAttributeDataProjectedTexture,
    PyAttributeDataText, PyAttributeDataVector, PyLocation,
};
use colormap::{PyNumberColormapContinuous, PyNumberColormapDiscrete};
use element::PyElement;
use errors::{
    OmfException, OmfFileIoException, OmfInvalidDataException, OmfJsonException,
    OmfLimitExceededException, OmfNotSupportedException, OmfValidationFailedException,
};
use file::reader::{PyBoundaryType, PyLimits, PyReader};
use geometry::{PyGridSurface, PyLineSet, PyPointSet, PySurface};
use omf1::converter::{detect_omf1, PyOmf1Converter};
use project::PyProject;
use validate::PyProblem;

/// Returns the version of the library
#[gen_stub_pyfunction]
#[pyfunction]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// This module provides python bindings for omf-rust.
#[pymodule]
fn omf_python(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAttribute>()?;
    m.add_class::<PyAttributeDataBoolean>()?;
    m.add_class::<PyAttributeDataCategory>()?;
    m.add_class::<PyAttributeDataColor>()?;
    m.add_class::<PyAttributeDataMappedTexture>()?;
    m.add_class::<PyAttributeDataNumber>()?;
    m.add_class::<PyAttributeDataProjectedTexture>()?;
    m.add_class::<PyAttributeDataText>()?;
    m.add_class::<PyAttributeDataVector>()?;
    m.add_class::<PyBooleanArray>()?;
    m.add_class::<PyBoundaryArray>()?;
    m.add_class::<PyColorArray>()?;
    m.add_class::<PyImageArray>()?;
    m.add_class::<PyIndexArray>()?;
    m.add_class::<PyGradientArray>()?;
    m.add_class::<PyLocation>()?;
    m.add_class::<PyNumberArray>()?;
    m.add_class::<PyNumberColormapContinuous>()?;
    m.add_class::<PyNumberColormapDiscrete>()?;
    m.add_class::<PyTextArray>()?;
    m.add_class::<PyVectorArray>()?;
    m.add_class::<PyScalarArray>()?;
    m.add_class::<PySegmentArray>()?;
    m.add_class::<PyVertexArray>()?;
    m.add_class::<PyTexcoordArray>()?;
    m.add_class::<PyTriangleArray>()?;
    m.add_class::<PyRegularSubblockArray>()?;
    m.add_class::<PyFreeformSubblockArray>()?;
    m.add_class::<PyRegularSubblocks>()?;
    m.add_class::<PyFreeformSubblocks>()?;
    m.add_class::<PySubblockMode>()?;
    m.add_class::<PyNameArray>()?;
    m.add_class::<PyElement>()?;
    m.add_class::<PyGrid2Regular>()?;
    m.add_class::<PyGrid2Tensor>()?;
    m.add_class::<PyGrid3Regular>()?;
    m.add_class::<PyGrid3Tensor>()?;
    m.add_class::<PyOrient2>()?;
    m.add_class::<PyOrient3>()?;
    m.add_class::<PyBlockModel>()?;
    m.add_class::<PyGridSurface>()?;
    m.add_class::<PyPointSet>()?;
    m.add_class::<PyLineSet>()?;
    m.add_class::<PyProject>()?;
    m.add_class::<PyProblem>()?;
    m.add_class::<PyReader>()?;
    m.add_class::<PySurface>()?;
    m.add_class::<PyLimits>()?;
    m.add_class::<PyBoundaryType>()?;
    m.add_class::<PyOmf1Converter>()?;

    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(detect_omf1, m)?)?;

    m.add("OmfException", py.get_type::<OmfException>())?;
    m.add("OmfFileIoException", py.get_type::<OmfFileIoException>())?;
    m.add(
        "OmfLimitExceededException",
        py.get_type::<OmfLimitExceededException>(),
    )?;
    m.add("OmfJsonException", py.get_type::<OmfJsonException>())?;
    m.add(
        "OmfInvalidDataException",
        py.get_type::<OmfInvalidDataException>(),
    )?;
    m.add(
        "OmfValidationFailedException",
        py.get_type::<OmfValidationFailedException>(),
    )?;
    m.add(
        "OmfNotSupportedException",
        py.get_type::<OmfNotSupportedException>(),
    )?;

    Ok(())
}

define_stub_info_gatherer!(stub_info);
