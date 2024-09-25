use crate::array::{
    PyBooleanArray, PyColorArray, PyGradientArray, PyImageArray, PyIndexArray, PyNameArray,
    PyNumberArray, PyTexcoordArray, PyTextArray, PyVectorArray,
};
use crate::colormap::{PyNumberColormapContinuous, PyNumberColormapDiscrete};
use crate::grid::PyOrient2;
use omf::{Attribute, AttributeData, Location, NumberColormap};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;

#[gen_stub_pyclass]
#[pyclass(name = "Attribute")]
/// Describes data attached to an Element.
///
/// Each Element can have zero or more attributes,
/// each attached to different parts of the element and each containing different types of data.
/// On a set of points, one attribute might contain gold assay results and another rock-type classifications.
pub struct PyAttribute(pub Attribute);

#[gen_stub_pymethods]
#[pymethods]
impl PyAttribute {
    #[getter]
    /// Attribute name. Should be unique within the containing element.
    fn name(&self) -> String {
        self.0.name.clone()
    }

    #[getter]
    /// Optional attribute description.
    fn description(&self) -> String {
        self.0.description.clone()
    }

    #[getter]
    /// Optional unit of the attribute data, default empty.
    ///
    /// OMF does not currently attempt to standardize the strings you can use here, but our
    /// recommendations are:
    ///
    /// - Use full names, so "kilometers" rather than "km". The abbreviations for non-metric units
    ///   aren't consistent and complex units can be confusing.
    ///
    /// - Use plurals, so "feet" rather than "foot".
    ///
    /// - Avoid ambiguity, so "long tons" rather than just "tons".
    ///
    /// - Accept American and British spellings, so "meter" and "metre" are the same.
    fn units(&self) -> String {
        self.0.units.clone()
    }

    #[getter]
    /// Attribute metadata.
    fn metadata(&self) -> PyResult<String> {
        let metadata = serde_json::to_string(&self.0.metadata)
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))?;
        Ok(metadata)
    }

    #[getter]
    /// Selects which part of the element the attribute is attached to.
    ///
    /// See the documentation for each Geometry variant for a list of what locations are valid.
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
    /// The attribute data as a JSON string.
    fn data_json(&self) -> PyResult<String> {
        let data = serde_json::to_string(&self.0.data)
            .map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))?;
        Ok(data)
    }

    /// The attribute data.
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
            AttributeData::ProjectedTexture { .. } => {
                Ok(PyAttributeDataProjectedTexture(self.0.data.clone()).into_py(py))
            }
            AttributeData::Number { .. } => {
                Ok(PyAttributeDataNumber(self.0.data.clone()).into_py(py))
            }
            AttributeData::Vector { .. } => {
                Ok(PyAttributeDataVector(self.0.data.clone()).into_py(py))
            }
            AttributeData::Boolean { .. } => {
                Ok(PyAttributeDataBoolean(self.0.data.clone()).into_py(py))
            }
            AttributeData::Text { .. } => Ok(PyAttributeDataText(self.0.data.clone()).into_py(py)),
        }
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "AttributeDataCategory")]
/// Category data.
///
/// A name is required for each category, a color is optional, and other values can be attached
/// as sub-attributes.
pub struct PyAttributeDataCategory(AttributeData);

#[gen_stub_pymethods]
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

#[gen_stub_pyclass]
#[pyclass(name = "AttributeDataColor")]
/// Color data.
pub struct PyAttributeDataColor(pub AttributeData);

#[gen_stub_pymethods]
#[pymethods]
impl PyAttributeDataColor {
    #[getter]
    /// Array with Color type storing the attribute values.
    ///
    /// Null values may be replaced by the element color, or a default color as the application prefers.
    fn values(&self) -> PyResult<PyColorArray> {
        match &self.0 {
            AttributeData::Color { values, .. } => Ok(PyColorArray(values.clone())),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "AttributeDataMappedTexture")]
/// A texture applied with UV mapping.
///
/// Typically applied to surface vertices. Applications may ignore other locations.
pub struct PyAttributeDataMappedTexture(pub AttributeData);

#[gen_stub_pymethods]
#[pymethods]
impl PyAttributeDataMappedTexture {
    #[getter]
    /// Array with Image type storing the texture image.
    fn image(&self) -> PyResult<PyImageArray> {
        match &self.0 {
            AttributeData::MappedTexture { image, .. } => Ok(PyImageArray(image.clone())),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }

    #[getter]
    /// Array with Texcoord type storing the UV texture coordinates.
    ///
    /// Each item is a normalized (U, V) pair. For values outside [0, 1] the texture wraps.
    fn texcoords(&self) -> PyResult<PyTexcoordArray> {
        match &self.0 {
            AttributeData::MappedTexture { texcoords, .. } => {
                Ok(PyTexcoordArray(texcoords.clone()))
            }
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "AttributeDataProjectedTexture")]
/// A texture defined as a rectangle in space projected along its normal.
///
/// Behavior of the texture outside the projected rectangle is not defined.
/// The texture might repeat, clip the element, or itself be clipped to reveal the flat color of the element.
///
/// The attribute location must be Projected.
pub struct PyAttributeDataProjectedTexture(pub AttributeData);

#[gen_stub_pymethods]
#[pymethods]
impl PyAttributeDataProjectedTexture {
    #[getter]
    /// Array with Image type storing the texture image.
    fn image(&self) -> PyResult<PyImageArray> {
        match &self.0 {
            AttributeData::ProjectedTexture { image, .. } => Ok(PyImageArray(image.clone())),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }

    #[getter]
    /// Orientation of the image.
    fn orient(&self) -> PyResult<PyOrient2> {
        match self.0 {
            AttributeData::ProjectedTexture { orient, .. } => Ok(PyOrient2(orient)),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }

    #[getter]
    /// Width of the image projection in space.
    fn width(&self) -> PyResult<f64> {
        match self.0 {
            AttributeData::ProjectedTexture { width, .. } => Ok(width),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }

    #[getter]
    /// Height of the image projection in space.
    fn height(&self) -> PyResult<f64> {
        match self.0 {
            AttributeData::ProjectedTexture { height, .. } => Ok(height),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "AttributeDataNumber")]
/// Number data with flexible type.
///
/// Values can be stored as 32 or 64-bit signed integers, 32 or 64-bit floating point,
/// date, or date-time. Valid dates are approximately ±262,000 years
/// from the common era. Date-time values are written with microsecond accuracy,
/// and times are always in UTC.
pub struct PyAttributeDataNumber(pub AttributeData);

#[gen_stub_pymethods]
#[pymethods]
impl PyAttributeDataNumber {
    #[getter]
    /// Array with `Number` type storing the attribute values.
    fn values(&self) -> PyResult<PyNumberArray> {
        match &self.0 {
            AttributeData::Number { values, .. } => Ok(PyNumberArray(values.clone())),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }

    #[getter]
    /// Optional colormap. If absent then the importing application should invent one.
    ///
    /// Make sure the colormap uses the same number type as `values`.
    fn colormap(&self, py: Python) -> PyResult<Option<PyObject>> {
        match &self.0 {
            AttributeData::Number { colormap, .. } => match colormap {
                Some(colormap) => match colormap {
                    NumberColormap::Continuous { .. } => {
                        let cmap = PyNumberColormapContinuous(colormap.clone());
                        Ok(Some(cmap.into_py(py)))
                    }
                    NumberColormap::Discrete { .. } => {
                        let cmap = PyNumberColormapDiscrete(colormap.clone());
                        Ok(Some(cmap.into_py(py)))
                    }
                },
                None => Ok(None),
            },
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "AttributeDataVector")]
/// 2D or 3D vector data.
pub struct PyAttributeDataVector(pub AttributeData);

#[gen_stub_pymethods]
#[pymethods]
impl PyAttributeDataVector {
    #[getter]
    /// Array with `Vector` type storing the attribute values.
    fn values(&self) -> PyResult<PyVectorArray> {
        match &self.0 {
            AttributeData::Vector { values, .. } => Ok(PyVectorArray(values.clone())),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "AttributeDataText")]
/// Text data.
pub struct PyAttributeDataText(pub AttributeData);

#[gen_stub_pymethods]
#[pymethods]
impl PyAttributeDataText {
    #[getter]
    /// Array with `Text` type storing the attribute values.
    fn values(&self) -> PyResult<PyTextArray> {
        match &self.0 {
            AttributeData::Text { values, .. } => Ok(PyTextArray(values.clone())),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "AttributeDataBoolean")]
/// Boolean or filter data.
pub struct PyAttributeDataBoolean(pub AttributeData);

#[gen_stub_pymethods]
#[pymethods]
impl PyAttributeDataBoolean {
    #[getter]
    /// Array with `Boolean` type storing the attribute values.
    ///
    /// These values may be true, false, or null. Applications that don't support
    /// [three-valued logic](https://en.wikipedia.org/wiki/Three-valued_logic) may treat
    /// null as false.
    fn values(&self) -> PyResult<PyBooleanArray> {
        match &self.0 {
            AttributeData::Boolean { values, .. } => Ok(PyBooleanArray(values.clone())),
            _ => Err(PyValueError::new_err(
                "AttributeData variant is not supported",
            )),
        }
    }
}
