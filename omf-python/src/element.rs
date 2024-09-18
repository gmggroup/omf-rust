use crate::attribute::PyAttribute;
use crate::geometry::PyGeometry;
use omf::Element;
use pyo3::prelude::*;

#[pyclass(name = "Element")]
pub struct PyElement(pub Element);

#[pymethods]
impl PyElement {
    #[getter]
    fn name(&self) -> String {
        self.0.name.clone()
    }

    #[getter]
    fn description(&self) -> String {
        self.0.description.clone()
    }

    #[getter]
    fn attributes(&self) -> Vec<PyAttribute> {
        self.0
            .attributes
            .iter()
            .map(|a| PyAttribute(a.clone()))
            .collect()
    }

    #[getter]
    fn geometry(&self) -> PyGeometry {
        PyGeometry(self.0.geometry.clone())
    }
}
