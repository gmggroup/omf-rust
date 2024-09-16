use crate::PyProject;
use omf::file::Reader;
use std::fs::File;

use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;

#[pyclass(name = "Reader")]
pub struct PyReader {
    pub inner: Reader,
}

#[pymethods]
impl PyReader {
    #[new]
    pub fn new(filepath: &str) -> PyResult<Self> {
        let file = File::open(filepath).map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;
        let reader = Reader::new(file).map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;
        Ok(PyReader { inner: reader })
    }

    #[getter]
    fn project(&self) -> PyResult<PyProject> {
        let (project, problems) = self
            .inner
            .project()
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;

        if !problems.is_empty() {
            println!("Warnings while reading project: {:?}", problems);
        }

        Ok(PyProject { inner: project })
    }

}

