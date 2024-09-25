use crate::attribute::PyAttribute;
use crate::geometry::PyGeometry;
use omf::Color;
use omf::Element;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(name = "Element")]
pub struct PyElement(pub Element);

#[gen_stub_pymethods]
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

    #[getter]
    fn color(&self) -> Option<Color> {
        self.0.color
    }
}
