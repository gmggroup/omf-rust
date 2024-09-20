use crate::array::{PyColorArray, PyIndexArray, PyNameArray};
use omf::{Attribute, AttributeData, Location};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(name = "Attribute")]
pub struct PyAttribute(pub Attribute);

#[gen_stub_pymethods]
#[pymethods]
impl PyAttribute {
    #[getter]
    fn name(&self) -> String {
        self.0.name.clone()
    }

    #[getter]
    fn description(&self) -> String {
        self.0.description.clone()
    }

    #[getter]
    fn units(&self) -> String {
        self.0.units.clone()
    }

    #[getter]
    fn metadata(&self) -> PyResult<String> {
        let metadata = serde_json::to_string(&self.0.metadata)
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))?;
        Ok(metadata)
    }

    #[getter]
    fn location(&self) -> String {
        match self.0.location {
            Location::Vertices => "Vertices",
            Location::Primitives => "Primitives",
            Location::Subblocks => "Subblocks",
            Location::Elements => "Elements",
            Location::Projected => "Projected",
            Location::Categories => "Categories",
        }
        .to_string()
    }

    #[getter]
    fn data_json(&self) -> PyResult<String> {
        let data = serde_json::to_string(&self.0.data)
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))?;
        Ok(data)
    }

    fn get_data(&self, py: Python<'_>) -> PyResult<PyObject> {
        match &self.0.data {
            AttributeData::Category { .. } => {
                Ok(PyAttributeDataCategory(self.0.data.clone()).into_py(py))
            }
            AttributeData::Color { .. } => {
                Ok(PyAttributeDataColor(self.0.data.clone()).into_py(py))
            }
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }
}

#[pyclass(name = "AttributeDataCategory")]
pub struct PyAttributeDataCategory(AttributeData);

#[pymethods]
impl PyAttributeDataCategory {
    #[getter]
    fn values(&self) -> PyResult<PyIndexArray> {
        match &self.0 {
            AttributeData::Category { values, .. } => Ok(PyIndexArray(values.clone())),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }

    #[getter]
    fn names(&self) -> PyResult<PyNameArray> {
        match &self.0 {
            AttributeData::Category { names, .. } => Ok(PyNameArray(names.clone())),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }

    #[getter]
    fn attributes(&self) -> PyResult<Vec<PyAttribute>> {
        match &self.0 {
            AttributeData::Category { attributes, .. } => {
                Ok(attributes.iter().map(|a| PyAttribute(a.clone())).collect())
            }
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }
}

#[pyclass(name = "AttributeDataColor")]
pub struct PyAttributeDataColor(pub AttributeData);
#[pymethods]
impl PyAttributeDataColor {
    #[getter]
    fn values(&self) -> PyResult<PyColorArray> {
        match &self.0 {
            AttributeData::Color { values, .. } => Ok(PyColorArray(values.clone())),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }
}
