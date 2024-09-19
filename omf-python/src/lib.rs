/// Python bindings.
use pyo3::prelude::*;

mod array;
mod attribute;
mod element;
mod file;
mod geometry;
mod omf1;
mod project;

use array::{PyIndexArray, PyTriangleArray, PyVertexArray};
use attribute::{PyAttribute, PyAttributeDataCategory};
use element::PyElement;
use file::reader::PyReader;
use geometry::{PyGeometry, PyPointSet, PySurface};
use omf1::converter::is_omf1;
use project::PyProject;

/// Returns the version of the library
#[pyfunction]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// This module provides python bindings for omf-rust.
#[pymodule]
fn omf_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAttribute>()?;
    m.add_class::<PyAttributeDataCategory>()?;
    m.add_class::<PyIndexArray>()?;
    m.add_class::<PyVertexArray>()?;
    m.add_class::<PyTriangleArray>()?;
    m.add_class::<PyElement>()?;
    m.add_class::<PyGeometry>()?;
    m.add_class::<PyPointSet>()?;
    m.add_class::<PyProject>()?;
    m.add_class::<PyReader>()?;
    m.add_class::<PySurface>()?;

    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(is_omf1, m)?)?;

    Ok(())
}
