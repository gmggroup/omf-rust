/// Python bindings.
use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};

mod array;
mod attribute;
mod colormap;
mod element;
mod file;
mod geometry;
mod grid;
mod omf1;
mod project;
mod validate;

use array::{
    PyBooleanArray, PyBoundaryArray, PyColorArray, PyGradientArray, PyImageArray, PyIndexArray,
    PyNameArray, PyNumberArray, PyTexcoordArray, PyTextArray, PyTriangleArray, PyVectorArray,
    PyVertexArray,
};
use attribute::{
    PyAttribute, PyAttributeDataBoolean, PyAttributeDataCategory, PyAttributeDataColor,
    PyAttributeDataMappedTexture, PyAttributeDataNumber, PyAttributeDataText,
    PyAttributeDataVector,
};
use colormap::{
    PyNumberColormapContinuous, PyNumberColormapDiscrete, PyNumberRangeDate, PyNumberRangeDateTime,
    PyNumberRangeFloat, PyNumberRangeInteger,
};
use element::PyElement;
use file::reader::{PyLimits, PyReader};
use geometry::{PyLineSet, PyPointSet, PySurface};
use omf1::converter::{detect_omf1, PyOmf1Converter};
use project::PyProject;

/// Returns the version of the library
#[gen_stub_pyfunction]
#[pyfunction]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// This module provides python bindings for omf-rust.
#[pymodule]
fn omf_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAttribute>()?;
    m.add_class::<PyAttributeDataBoolean>()?;
    m.add_class::<PyAttributeDataCategory>()?;
    m.add_class::<PyAttributeDataColor>()?;
    m.add_class::<PyAttributeDataMappedTexture>()?;
    m.add_class::<PyAttributeDataNumber>()?;
    m.add_class::<PyAttributeDataText>()?;
    m.add_class::<PyAttributeDataVector>()?;
    m.add_class::<PyBooleanArray>()?;
    m.add_class::<PyBoundaryArray>()?;
    m.add_class::<PyColorArray>()?;
    m.add_class::<PyImageArray>()?;
    m.add_class::<PyIndexArray>()?;
    m.add_class::<PyGradientArray>()?;
    m.add_class::<PyNumberArray>()?;
    m.add_class::<PyNumberColormapContinuous>()?;
    m.add_class::<PyNumberColormapDiscrete>()?;
    m.add_class::<PyNumberRangeDate>()?;
    m.add_class::<PyNumberRangeDateTime>()?;
    m.add_class::<PyNumberRangeFloat>()?;
    m.add_class::<PyNumberRangeInteger>()?;
    m.add_class::<PyTextArray>()?;
    m.add_class::<PyVectorArray>()?;
    m.add_class::<PyVertexArray>()?;
    m.add_class::<PyTexcoordArray>()?;
    m.add_class::<PyTriangleArray>()?;
    m.add_class::<PyNameArray>()?;
    m.add_class::<PyElement>()?;
    m.add_class::<PyPointSet>()?;
    m.add_class::<PyLineSet>()?;
    m.add_class::<PyProject>()?;
    m.add_class::<PyReader>()?;
    m.add_class::<PySurface>()?;
    m.add_class::<PyLimits>()?;
    m.add_class::<PyOmf1Converter>()?;

    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(detect_omf1, m)?)?;

    Ok(())
}

define_stub_info_gatherer!(stub_info);
