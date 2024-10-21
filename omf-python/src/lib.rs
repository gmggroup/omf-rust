use block_model::{PyBlockModel, PyFreeformSubblocks, PyRegularSubblocks, PySubblockMode};
use grid::{PyGrid2Regular, PyGrid2Tensor, PyGrid3Regular, PyGrid3Tensor, PyOrient2, PyOrient3};
/// Python bindings.
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
use file::reader::{PyLimits, PyReader};
use geometry::{PyGridSurface, PyLineSet, PyPointSet, PySurface};
use omf1::converter::{detect_omf1, PyOmf1Converter};
use project::PyProject;
use validate::PyProblem;

/// Returns the version of the library
#[gen_stub_pyfunction]
#[pyfunction]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
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
    m.add_class::<PyOmf1Converter>()?;

    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(detect_omf1, m)?)?;

    m.add("OmfException", py.get_type_bound::<OmfException>())?;
    m.add(
        "OmfFileIoException",
        py.get_type_bound::<OmfFileIoException>(),
    )?;
    m.add(
        "OmfLimitExceededException",
        py.get_type_bound::<OmfLimitExceededException>(),
    )?;
    m.add("OmfJsonException", py.get_type_bound::<OmfJsonException>())?;
    m.add(
        "OmfInvalidDataException",
        py.get_type_bound::<OmfInvalidDataException>(),
    )?;
    m.add(
        "OmfValidationFailedException",
        py.get_type_bound::<OmfValidationFailedException>(),
    )?;
    m.add(
        "OmfNotSupportedException",
        py.get_type_bound::<OmfNotSupportedException>(),
    )?;

    Ok(())
}

define_stub_info_gatherer!(stub_info);
