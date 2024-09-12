/// Python bindings.
use pyo3::prelude::*;

mod element;
mod geometry;
mod file;
mod project;

use element::PyElement;
use geometry::{PyGeometry, PyPointSet};
use file::reader::{PyFileInfo, PyReader};
use project::PyProject;

#[pymodule]
fn omf_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyElement>()?;
    m.add_class::<PyGeometry>()?;
    m.add_class::<PyPointSet>()?;
    m.add_class::<PyProject>()?;
    m.add_class::<PyReader>()?;
    m.add_class::<PyFileInfo>()?;
    Ok(())
}
