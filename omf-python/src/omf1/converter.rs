use omf::omf1::detect_open;
use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;
use std::path::Path;

#[pyfunction]
pub fn is_omf1(path: String) -> PyResult<bool> {
    let path = Path::new(&path);
    match detect_open(path) {
        Ok(result) => Ok(result),
        Err(e) => Err(PyErr::new::<PyIOError, _>(e.to_string())),
    }
}
