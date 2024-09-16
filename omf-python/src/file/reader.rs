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

    fn get_file_info(&self) -> PyResult<PyFileInfo> {
        let (project, problems) = self
            .inner
            .project()
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))?;

        if !problems.is_empty() {
            println!("Warnings while reading project: {:?}", problems);
        }

        Ok(PyFileInfo {
            project_name: project.name,
            project_description: project.description,
            version: self.inner.version(),
        })
    }
}

#[pyclass]
pub struct PyFileInfo {
    version: [u32; 2],
    project_name: String,
    project_description: String,
}

#[pymethods]
impl PyFileInfo {
    #[getter]
    fn project_name(&self) -> PyResult<String> {
        Ok(self.project_name.clone())
    }
    #[getter]
    fn project_description(&self) -> PyResult<String> {
        Ok(self.project_description.clone())
    }
    #[getter]
    fn version(&self) -> PyResult<(u32, u32)> {
        Ok(self.version.into())
    }
}
