use crate::array::{
    PyBooleanArray, PyColorArray, PyGradientArray, PyImageArray, PyIndexArray, PyNameArray,
    PyNumberArray, PyTexcoordArray, PyTextArray, PyVectorArray,
};
use crate::colormap::{PyNumberColormapContinuous, PyNumberColormapDiscrete};
use crate::errors::OmfJsonException;
use crate::grid::PyOrient2;
use omf::{Attribute, AttributeData, Location, NumberColormap};
use pyo3::{prelude::*, IntoPyObjectExt};
use pyo3_stub_gen::derive::*;
use serde_pyobject::to_pyobject;

#[gen_stub_pyclass_enum]
#[pyclass(eq, eq_int, name = "Location")]
#[derive(PartialEq, Eq)]
/// Describes what part of the geometry an attribute attaches to.
///
/// See the documentation for each Geometry variant for a list of what
/// locations are valid.
pub enum PyLocation {
    /// The attribute contains one value for each point, vertex, or block corner.
    Vertices,
    /// The attribute contains one value for each line segment, triangle, or block.
    /// For sub-blocked block models that means parent blocks.
    Primitives,
    /// The attribute contains one value for each sub-block in a block model.
    Subblocks,
    /// The attribute contains one value for each sub-element in Composite geometry.
    Elements,
    /// Used by ProjectedTexture attributes. The texture is projected onto the element.
    Projected,
    /// Used for category sub-attributes. The attribute contains one value for each category.
    Categories,
}

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
    fn metadata<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        to_pyobject(py, &self.0.metadata).map_err(|e| OmfJsonException::new_err(e.to_string()))
    }

    #[getter]
    /// Selects which part of the element the attribute is attached to.
    ///
    /// See the documentation for each Geometry variant for a list of what locations are valid.
    fn location(&self) -> PyLocation {
        match self.0.location {
            Location::Vertices => PyLocation::Vertices,
            Location::Primitives => PyLocation::Primitives,
            Location::Subblocks => PyLocation::Subblocks,
            Location::Elements => PyLocation::Elements,
            Location::Projected => PyLocation::Projected,
            Location::Categories => PyLocation::Categories,
        }
    }

    /// The attribute data.
    fn get_data(&self, py: Python<'_>) -> PyResult<PyObject> {
        match &self.0.data {
            AttributeData::Category { .. } => {
                PyAttributeDataCategory(self.0.data.clone()).into_py_any(py)
            }
            AttributeData::Color { .. } => {
                PyAttributeDataColor(self.0.data.clone()).into_py_any(py)
            }
            AttributeData::MappedTexture { .. } => {
                PyAttributeDataMappedTexture(self.0.data.clone()).into_py_any(py)
            }
            AttributeData::ProjectedTexture { .. } => {
                PyAttributeDataProjectedTexture(self.0.data.clone()).into_py_any(py)
            }
            AttributeData::Number { .. } => {
                PyAttributeDataNumber(self.0.data.clone()).into_py_any(py)
            }
            AttributeData::Vector { .. } => {
                PyAttributeDataVector(self.0.data.clone()).into_py_any(py)
            }
            AttributeData::Boolean { .. } => {
                PyAttributeDataBoolean(self.0.data.clone()).into_py_any(py)
            }
            AttributeData::Text { .. } => PyAttributeDataText(self.0.data.clone()).into_py_any(py),
        }
    }
}

macro_rules! attribute_data_field {
    ($self:ident, $variant:ident :: $field:ident) => {
        match &$self.0 {
            AttributeData::$variant { $field, .. } => $field,
            _ => unreachable!("AttributeData variant is not supported"),
        }
    };
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
    fn values(&self) -> PyIndexArray {
        PyIndexArray(attribute_data_field!(self, Category::values).clone())
    }

    #[getter]
    /// Array with `Name` type storing category names.
    fn names(&self) -> PyNameArray {
        PyNameArray(attribute_data_field!(self, Category::names).clone())
    }

    #[getter]
    /// Optional array with `Gradient` type storing category colors.
    ///
    /// If present, must be the same length as `names`. If absent then the importing
    /// application should invent colors.
    fn gradient(&self) -> Option<PyGradientArray> {
        attribute_data_field!(self, Category::gradient)
            .as_ref()
            .map(|g| PyGradientArray(g.clone()))
    }

    #[getter]
    /// Additional attributes that use the same indices.
    ///
    /// This could be used to store the density of rock types in a lithology attribute for
    /// example. The location field of these attributes must be `Categories`.
    /// They must have the same length as `names`.
    fn attributes(&self) -> Vec<PyAttribute> {
        attribute_data_field!(self, Category::attributes)
            .iter()
            .map(|a| PyAttribute(a.clone()))
            .collect()
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
    fn values(&self) -> PyColorArray {
        PyColorArray(attribute_data_field!(self, Color::values).clone())
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
    fn image(&self) -> PyImageArray {
        PyImageArray(attribute_data_field!(self, MappedTexture::image).clone())
    }

    #[getter]
    /// Array with Texcoord type storing the UV texture coordinates.
    ///
    /// Each item is a normalized (U, V) pair. For values outside [0, 1] the texture wraps.
    fn texcoords(&self) -> PyTexcoordArray {
        PyTexcoordArray(attribute_data_field!(self, MappedTexture::texcoords).clone())
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
    fn image(&self) -> PyImageArray {
        PyImageArray(attribute_data_field!(self, ProjectedTexture::image).clone())
    }

    #[getter]
    /// Orientation of the image.
    fn orient(&self) -> PyOrient2 {
        PyOrient2(*attribute_data_field!(self, ProjectedTexture::orient))
    }

    #[getter]
    /// Width of the image projection in space.
    fn width(&self) -> f64 {
        *attribute_data_field!(self, ProjectedTexture::width)
    }

    #[getter]
    /// Height of the image projection in space.
    fn height(&self) -> f64 {
        *attribute_data_field!(self, ProjectedTexture::height)
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
    fn values(&self) -> PyNumberArray {
        PyNumberArray(attribute_data_field!(self, Number::values).clone())
    }

    #[getter]
    /// Optional colormap. If absent then the importing application should invent one.
    ///
    /// Make sure the colormap uses the same number type as `values`.
    fn colormap(&self, py: Python) -> PyResult<Option<PyObject>> {
        match attribute_data_field!(self, Number::colormap).clone() {
            None => Ok(None),
            Some(colormap) => Ok(Some(match colormap {
                NumberColormap::Continuous { .. } => {
                    PyNumberColormapContinuous(colormap).into_py_any(py)?
                }
                NumberColormap::Discrete { .. } => {
                    PyNumberColormapDiscrete(colormap).into_py_any(py)?
                }
            })),
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
    fn values(&self) -> PyVectorArray {
        PyVectorArray(attribute_data_field!(self, Vector::values).clone())
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
    fn values(&self) -> PyTextArray {
        PyTextArray(attribute_data_field!(self, Text::values).clone())
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
    fn values(&self) -> PyBooleanArray {
        PyBooleanArray(attribute_data_field!(self, Boolean::values).clone())
    }
}
