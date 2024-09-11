/// Python bindings.
use pyo3::prelude::*;

mod file;

use file::reader::{PyFileInfo, PyReader};

#[pymodule]
fn omf_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyReader>()?;
    m.add_class::<PyFileInfo>()?;
    Ok(())
}
