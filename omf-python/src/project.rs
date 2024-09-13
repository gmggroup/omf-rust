use crate::element::PyElement;
use omf::Project;
use pyo3::prelude::*;

#[pyclass(name = "Project")]
pub struct PyProject {
    pub inner: Project,
}

#[pymethods]
impl PyProject {
    #[new]
    pub fn new(name: String) -> Self {
        PyProject {
            inner: Project::new(name),
        }
    }

    #[getter]
    fn name(&self) -> PyResult<String> {
        Ok(self.inner.name.clone())
    }

    #[getter]
    fn elements(&self) -> PyResult<Vec<PyElement>> {
        Ok(self
            .inner
            .elements
            .iter()
            .map(|e| PyElement { inner: e.clone() })
            .collect())
    }
}
