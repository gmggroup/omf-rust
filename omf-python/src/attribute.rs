use crate::array::PyArrayIndex;
use omf::{Attribute, AttributeData, Location};
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
    fn data_json(&self) -> PyResult<String> {
        let data = serde_json::to_string(&self.inner.data)
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))?;
        Ok(data)
    }

    fn get_data(&self, py: Python<'_>) -> PyResult<PyObject> {
        match &self.inner.data {
            AttributeData::Category { .. } => Ok(PyAttributeDataCategory {
                inner: self.inner.data.clone(),
            }
            .into_py(py)),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }
}

#[pyclass(name = "AttributeDataCategory")]
pub struct PyAttributeDataCategory {
    inner: AttributeData,
}

#[pymethods]
impl PyAttributeDataCategory {
    #[getter]
    fn values(&self) -> PyResult<PyArrayIndex> {
        match &self.inner {
            AttributeData::Category { values, .. } => Ok(PyArrayIndex {
                inner: values.clone(),
            }),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }

    #[getter]
    fn attributes(&self) -> PyResult<Vec<PyAttribute>> {
        match &self.inner {
            AttributeData::Category { attributes, .. } => Ok(attributes
                .iter()
                .map(|a| PyAttribute { inner: a.clone() })
                .collect()),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }
}
