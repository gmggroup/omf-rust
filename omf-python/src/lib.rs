/// Python bindings.
use pyo3::prelude::*;

mod element;
mod file;
mod geometry;
mod omf1;
mod project;

use element::PyElement;
use file::reader::{PyFileInfo, PyReader};
use geometry::{PyGeometry, PyPointSet};
use omf1::converter::detect_omf1;
use project::PyProject;

#[pymodule]
fn omf_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyElement>()?;
    m.add_class::<PyGeometry>()?;
    m.add_class::<PyPointSet>()?;
    m.add_class::<PyProject>()?;
    m.add_class::<PyReader>()?;
    m.add_class::<PyFileInfo>()?;

    m.add_function(wrap_pyfunction!(detect_omf1, m)?)?;
    Ok(())
}
