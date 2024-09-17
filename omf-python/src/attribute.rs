use omf::{Attribute, Location};
use pyo3::exceptions::PyValueError;
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
    fn metadata(&self) -> PyResult<String> {
        let metadata = serde_json::to_string(&self.inner.metadata)
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))?;
        Ok(metadata)
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
        }
        .to_string())
    }

    #[getter]
    fn data(&self) -> PyResult<String> {
        let data = serde_json::to_string(&self.inner.data)
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))?;
        Ok(data)
    }
}
