/// Python bindings.
use pyo3::prelude::*;

mod file;

use file::reader::{PyReader, PyFileInfo};

#[pymodule]
fn omf_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyReader>()?;
    m.add_class::<PyFileInfo>()?;
    Ok(())
}
