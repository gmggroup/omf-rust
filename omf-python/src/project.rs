use crate::element::PyElement;
use omf::Project;
use pyo3::prelude::*;

#[pyclass(name = "Project")]
pub struct PyProject(pub Project);

#[pymethods]
impl PyProject {
    #[getter]
    fn name(&self) -> String {
        self.0.name.clone()
    }

    #[getter]
    fn description(&self) -> String {
        self.0.description.clone()
    }

    #[getter]
    fn coordinate_reference_system(&self) -> String {
        self.0.coordinate_reference_system.clone()
    }

    #[getter]
    fn units(&self) -> String {
        self.0.units.clone()
    }

    #[getter]
    fn origin(&self) -> [f64; 3] {
        self.0.origin
    }

    #[getter]
    fn author(&self) -> String {
        self.0.author.clone()
    }

    #[getter]
    fn application(&self) -> String {
        self.0.application.clone()
    }

    #[getter]
    fn elements(&self) -> Vec<PyElement> {
        self.0
            .elements
            .iter()
            .map(|e| PyElement(e.clone()))
            .collect()
    }
}
