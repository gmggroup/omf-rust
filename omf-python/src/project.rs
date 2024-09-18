use crate::element::PyElement;
use omf::Project;
use pyo3::prelude::*;

#[pyclass(name = "Project")]
pub struct PyProject {
    pub inner: Project,
}

#[pymethods]
impl PyProject {
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    fn description(&self) -> String {
        self.inner.description.clone()
    }

    #[getter]
    fn coordinate_reference_system(&self) -> String {
        self.inner.coordinate_reference_system.clone()
    }

    #[getter]
    fn units(&self) -> String {
        self.inner.units.clone()
    }

    #[getter]
    fn origin(&self) -> [f64; 3] {
        self.inner.origin
    }

    #[getter]
    fn author(&self) -> String {
        self.inner.author.clone()
    }

    #[getter]
    fn application(&self) -> String {
        self.inner.application.clone()
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
