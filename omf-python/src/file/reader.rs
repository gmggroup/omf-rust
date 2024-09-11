use std::fs::File;
use omf::file::Reader;

use pyo3::prelude::*;


#[pyclass]
pub struct PyReader {
    inner: Reader,
}

#[pymethods]
impl PyReader {
    #[new]
    fn new(filepath: &str) -> PyResult<Self> {
        let inner = Reader::new(File::open(filepath).unwrap())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(PyReader { inner })
    }

    fn get_file_info(&self) -> PyResult<PyFileInfo> {
        let (project, problems) = self.inner.project().map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

        if !problems.is_empty() {
            println!("Warnings while reading project: {:?}", problems);
        }

        Ok(PyFileInfo {
            project_name: project.name,
            project_description: project.description,
        })
    }
}

#[pyclass]
pub struct PyFileInfo {
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
}
