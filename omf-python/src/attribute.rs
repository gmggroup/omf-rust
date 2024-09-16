use omf::{Attribute, Location};
use pyo3::prelude::*;

#[pyclass(name = "Attribute")]
pub struct PyAttribute {
    pub inner: Attribute,
}

#[pymethods]
impl PyAttribute {

    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    fn description(&self) -> String {
        self.inner.description.clone()
    }

    #[getter]
    fn units(&self) -> String {
        self.inner.units.clone()
    }

    #[getter]
    fn metadata(&self) -> String {
        "Hello world".to_string()
    }

    #[getter]
    fn location(&self) -> PyResult<String> {
        Ok(match self.inner.location {
            Location::Vertices => "Vertices",
            Location::Primitives => "Primitives",
            Location::Subblocks => "Subblocks",
            Location::Elements => "Elements",
            Location::Projected => "Projected",
            Location::Categories => "Categories",
        }.to_string())
    }

    #[getter]
    fn data(&self) -> PyResult<String> {
        Ok("Hello world".to_string())
    }

}
