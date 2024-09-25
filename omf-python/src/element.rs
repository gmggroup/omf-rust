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
    fn color(&self) -> Option<PyColor> {
        self.0.color.map(PyColor)
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "Color")]
pub struct PyColor(pub Color);

#[gen_stub_pymethods]
#[pymethods]
impl PyColor {
    const RED: usize = 0;
    const GREEN: usize = 1;
    const BLUE: usize = 2;
    const ALPHA: usize = 3;

    #[new]
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        PyColor([red, green, blue, alpha])
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    #[getter]
    fn red(&self) -> u8 {
        self.0[Self::RED]
    }

    #[getter]
    fn green(&self) -> u8 {
        self.0[Self::GREEN]
    }
    #[getter]
    fn blue(&self) -> u8 {
        self.0[Self::BLUE]
    }
    #[getter]
    fn alpha(&self) -> u8 {
        self.0[Self::ALPHA]
    }

    fn as_list(&self) -> [u8; 4] {
        self.0
    }
}
