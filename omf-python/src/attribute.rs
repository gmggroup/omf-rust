use crate::array::{
    PyColorArray, PyGradientArray, PyImageArray, PyIndexArray, PyNameArray,
    PyTextureCoordinatesArray,
};
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
            AttributeData::MappedTexture { .. } => {
                Ok(PyAttributeDataMappedTexture(self.0.data.clone()).into_py(py))
            }
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }
}

#[pyclass(name = "AttributeDataCategory")]
/// Category data.
///
/// A name is required for each category, a color is optional, and other values can be attached
/// as sub-attributes.
pub struct PyAttributeDataCategory(AttributeData);

#[pymethods]
impl PyAttributeDataCategory {
    #[getter]
    /// Array with `Index` type storing the category indices.
    ///
    /// Values are indices into the `names` array, `colors` array, and any sub-attributes,
    /// and must be within range for them.
    fn values(&self) -> PyResult<PyIndexArray> {
        match &self.0 {
            AttributeData::Category { values, .. } => Ok(PyIndexArray(values.clone())),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }

    #[getter]
    /// Array with `Name` type storing category names.
    fn names(&self) -> PyResult<PyNameArray> {
        match &self.0 {
            AttributeData::Category { names, .. } => Ok(PyNameArray(names.clone())),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }

    #[getter]
    /// Optional array with `Gradient` type storing category colors.
    ///
    /// If present, must be the same length as `names`. If absent then the importing
    /// application should invent colors.
    fn gradient(&self) -> PyResult<Option<PyGradientArray>> {
        match &self.0 {
            AttributeData::Category { gradient, .. } => {
                Ok(gradient.as_ref().map(|g| PyGradientArray(g.clone())))
            }
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }

    #[getter]
    /// Additional attributes that use the same indices.
    ///
    /// This could be used to store the density of rock types in a lithology attribute for
    /// example. The location field of these attributes must be `Categories`.
    /// They must have the same length as `names`.
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

#[pyclass(name = "AttributeDataMappedTexture")]
pub struct PyAttributeDataMappedTexture(pub AttributeData);
#[pymethods]
impl PyAttributeDataMappedTexture {
    #[getter]
    fn image(&self) -> PyResult<PyImageArray> {
        match &self.0 {
            AttributeData::MappedTexture { image, .. } => Ok(PyImageArray(image.clone())),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }

    #[getter]
    fn texcoords(&self) -> PyResult<PyTextureCoordinatesArray> {
        match &self.0 {
            AttributeData::MappedTexture { texcoords, .. } => {
                Ok(PyTextureCoordinatesArray(texcoords.clone()))
            }
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }
}
