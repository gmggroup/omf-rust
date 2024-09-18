use crate::attribute::PyAttribute;
use crate::geometry::PyGeometry;
use omf::Element;
use pyo3::prelude::*;

#[pyclass(name = "Element")]
pub struct PyElement(pub Element);

#[pymethods]
impl PyElement {
    #[getter]
    fn name(&self) -> PyResult<String> {
        Ok(self.0.name.clone())
    }

    #[getter]
    fn description(&self) -> PyResult<String> {
        Ok(self.0.description.clone())
    }

    #[getter]
    fn attributes(&self) -> PyResult<Vec<PyAttribute>> {
        Ok(self
            .0
            .attributes
            .iter()
            .map(|a| PyAttribute(a.clone()))
            .collect())
    }

    #[getter]
    fn geometry(&self) -> PyResult<PyGeometry> {
        Ok(PyGeometry(self.0.geometry.clone()))
    }
}
