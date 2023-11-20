use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    array::Constraint,
    array_type,
    colormap::NumberRange,
    validate::{Validate, Validator},
    Array, NumberColormap, Orient2,
};

/// The various types of data that can be attached to an [`Attribute`](crate::Attribute).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum AttributeData {
    /// Number data with flexible type.
    ///
    /// Values can be stored as 32 or 64-bit signed integers, 32 or 64-bit floating point,
    /// date, or date-time. Valid dates are approximately ±262,000 years
    /// from the common era. Date-time values are written with microsecond accuracy,
    /// and times are always in UTC.
    Number {
        /// Array with `Number` type storing the attribute values.
        values: Array<array_type::Number>,
        /// Optional colormap. If absent then the importing application should invent one.
        ///
        /// Make sure the colormap uses the same number type as `values`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        colormap: Option<NumberColormap>,
    },
    /// 2D or 3D vector data.
    Vector {
        /// Array with `Vector` type storing the attribute values.
        values: Array<array_type::Vector>,
    },
    /// Text data.
    Text {
        /// Array with `Text` type storing the attribute values.
        values: Array<array_type::Text>,
    },
    /// Category data.
    ///
    /// A name is required for each category, a color is optional, and other values can be attached
    /// as sub-attributes.
    Category {
        /// Array with `Index` type storing the category indices.
        ///
        /// Values are indices into the `names` array, `colors` array, and any sub-attributes,
        /// and must be within range for them.
        values: Array<array_type::Index>,
        /// Array with `Name` type storing category names.
        names: Array<array_type::Name>,
        /// Optional array with `Gradient` type storing category colors.
        ///
        /// If present, must be the same length as `names`. If absent then the importing
        /// application should invent colors.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gradient: Option<Array<array_type::Gradient>>,
        /// Additional attributes that use the same indices.
        ///
        /// This could be used to store the density of rock types in a lithology attribute for
        /// example. The location field of these attributes must be
        /// `Categories`[crate::Location::Categories]. They must have the same length as `names`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attributes: Vec<Attribute>,
    },
    /// Boolean or filter data.
    Boolean {
        /// Array with `Boolean` type storing the attribute values.
        ///
        /// These values may be true, false, or null. Applications that don't support
        /// [three-valued logic](https://en.wikipedia.org/wiki/Three-valued_logic) may treat
        /// null as false.
        values: Array<array_type::Boolean>,
    },
    /// Color data.
    Color {
        /// Array with `Color` type storing the attribute values.
        ///
        /// Null values may be replaced by the element color, or a default color as the
        /// application prefers.
        values: Array<array_type::Color>,
    },
    /// A texture applied with [UV mapping](https://en.wikipedia.org/wiki/UV_mapping).
    ///
    /// Typically applied to surface vertices. Applications may ignore other locations.
    MappedTexture {
        /// Array with `Image` type storing the texture image.
        image: Array<array_type::Image>,
        /// Array with `Texcoord` type storing the UV texture coordinates.
        ///
        /// Each item is a normalized (U, V) pair. For values outside [0, 1] the texture wraps.
        texcoords: Array<array_type::Texcoord>,
    },
    /// A texture defined as a rectangle in space projected along its normal.
    ///
    /// Behavior of the texture outside the projected rectangle is not defined. The texture
    /// might repeat, clip the element, or itself be clipped to reveal the flat color of the
    /// element.
    ///
    /// The attribute location must be [`Projected`](crate::Location::Projected).
    ProjectedTexture {
        /// Array with `Image` type storing the texture image.
        image: Array<array_type::Image>,
        /// Orientation of the image.
        orient: Orient2,
        /// Width of the image projection in space.
        width: f64,
        /// Height of the image projection in space.
        height: f64,
    },
}

impl AttributeData {
    /// True if the attribute data length is zero.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Length of the attribute data; zero for projected textures.
    pub fn len(&self) -> u64 {
        match self {
            Self::Number { values, .. } => values.item_count(),
            Self::Vector { values } => values.item_count(),
            Self::Text { values } => values.item_count(),
            Self::Category { values, .. } => values.item_count(),
            Self::Boolean { values } => values.item_count(),
            Self::Color { values, .. } => values.item_count(),
            Self::MappedTexture { texcoords, .. } => texcoords.item_count(),
            Self::ProjectedTexture { .. } => 0,
        }
    }

    pub(crate) fn type_name(&self) -> &'static str {
        match self {
            Self::Number { .. } => "Number",
            Self::Vector { .. } => "Vector",
            Self::Text { .. } => "String",
            Self::Category { .. } => "Category",
            Self::Boolean { .. } => "Boolean",
            Self::Color { .. } => "Color",
            Self::MappedTexture { .. } => "MappedTexture",
            Self::ProjectedTexture { .. } => "ProjectedTexture",
        }
    }
}

/// Describes data attached to an [`Element`](crate::Element).
///
/// Each [`Element`](crate::Element) can have zero or more attributes,
/// each attached to different parts of the element and each containing different types of data.
/// On a set of points, one attribute might contain gold assay results and another rock-type classifications.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Attribute {
    /// Attribute name. Should be unique within the containing element.
    pub name: String,
    /// Optional attribute description.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
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
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub units: String,
    /// Attribute metadata.
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub metadata: serde_json::Map<String, Value>,
    /// Selects which part of the element the attribute is attached to.
    ///
    /// See the documentation for each [`Geometry`](crate::Geometry) variant for a list of what
    /// locations are valid.
    pub location: Location,
    /// The attribute data.
    pub data: AttributeData,
}

impl Attribute {
    pub(crate) fn new(name: impl Into<String>, location: Location, data: AttributeData) -> Self {
        Self {
            name: name.into(),
            description: Default::default(),
            units: Default::default(),
            metadata: Default::default(),
            location,
            data,
        }
    }

    /// Convenience function to create a number attribute.
    pub fn from_numbers(
        name: impl Into<String>,
        location: Location,
        values: Array<array_type::Number>,
    ) -> Self {
        Self::new(
            name,
            location,
            AttributeData::Number {
                values,
                colormap: None,
            },
        )
    }

    /// Convenience function to create a number attribute with a continuous colormap.
    pub fn from_numbers_continuous_colormap(
        name: impl Into<String>,
        location: Location,
        values: Array<array_type::Number>,
        range: impl Into<NumberRange>,
        gradient: Array<array_type::Gradient>,
    ) -> Self {
        Self::new(
            name,
            location,
            AttributeData::Number {
                values,
                colormap: Some(NumberColormap::Continuous {
                    range: range.into(),
                    gradient,
                }),
            },
        )
    }

    /// Convenience function to create a number attribute with a discrete colormap.
    pub fn from_numbers_discrete_colormap(
        name: impl Into<String>,
        location: Location,
        values: Array<array_type::Number>,
        boundaries: Array<array_type::Boundary>,
        gradient: Array<array_type::Gradient>,
    ) -> Self {
        Self::new(
            name,
            location,
            AttributeData::Number {
                values,
                colormap: Some(NumberColormap::Discrete {
                    boundaries,
                    gradient,
                }),
            },
        )
    }

    /// Convenience function to create a vector attribute.
    pub fn from_vectors(
        name: impl Into<String>,
        location: Location,
        values: Array<array_type::Vector>,
    ) -> Self {
        Self::new(name, location, AttributeData::Vector { values })
    }

    /// Convenience function to create a string attribute.
    pub fn from_strings(
        name: impl Into<String>,
        location: Location,
        values: Array<array_type::Text>,
    ) -> Self {
        Self::new(name, location, AttributeData::Text { values })
    }

    /// Convenience function to create a category attribute.
    pub fn from_categories(
        name: impl Into<String>,
        location: Location,
        values: Array<array_type::Index>,
        names: Array<array_type::Name>,
        gradient: Option<Array<array_type::Gradient>>,
        attributes: impl IntoIterator<Item = Attribute>,
    ) -> Self {
        Self::new(
            name,
            location,
            AttributeData::Category {
                values,
                names,
                gradient,
                attributes: attributes.into_iter().collect(),
            },
        )
    }

    /// Convenience function to create a number attribute.
    pub fn from_booleans(
        name: impl Into<String>,
        location: Location,
        values: Array<array_type::Boolean>,
    ) -> Self {
        Self::new(name, location, AttributeData::Boolean { values })
    }

    /// Convenience function to create a color attribute.
    pub fn from_colors(
        name: impl Into<String>,
        location: Location,
        values: Array<array_type::Color>,
    ) -> Self {
        Self::new(name, location, AttributeData::Color { values })
    }

    /// Convenience function to create a mapped texture attribute.
    pub fn from_texture_map(
        name: impl Into<String>,
        image: Array<array_type::Image>,
        location: Location,
        texcoords: Array<array_type::Texcoord>,
    ) -> Self {
        Self::new(
            name,
            location,
            AttributeData::MappedTexture { image, texcoords },
        )
    }

    /// Convenience function to create a projected texture attribute.
    pub fn from_texture_project(
        name: impl Into<String>,
        image: Array<array_type::Image>,
        orient: Orient2,
        width: f64,
        height: f64,
    ) -> Self {
        Self::new(
            name,
            Location::Projected,
            AttributeData::ProjectedTexture {
                image,
                orient,
                width,
                height,
            },
        )
    }

    /// Returns the length of the attribute, or zero for a projected texture.
    pub fn len(&self) -> u64 {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Describes what part of the geometry an attribute attaches to.
///
/// See the documentation for each [`Geometry`](crate::Geometry) variant for a list of what
/// locations are valid.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[repr(i32)]
pub enum Location {
    /// The attribute contains one value for each point, vertex, or block corner.
    #[default]
    Vertices,
    /// The attribute contains one value for each line segment, triangle, or block.
    /// For sub-blocked block models that means parent blocks.
    Primitives,
    /// The attribute contains one value for each sub-block in a block model.
    Subblocks,
    /// The attribute contains one value for each sub-element in a
    /// [`Composite`](crate::Geometry::Composite).
    Elements,
    /// Used by [projected texture](crate::AttributeData::ProjectedTexture) attributes.
    /// The texture is projected onto the element
    Projected,
    /// Used for category sub-attributes.
    /// The attribute contains one value for each category.
    Categories,
}

impl Validate for Attribute {
    fn validate_inner(&mut self, val: &mut Validator) {
        val.enter("Attribute").name(&self.name).obj(&mut self.data);
    }
}

impl Validate for AttributeData {
    fn validate_inner(&mut self, val: &mut Validator) {
        match self {
            Self::Number { values, colormap } => {
                val.enter("AttributeData::Number")
                    .array(values, Constraint::Number, "values")
                    .obj(colormap);
            }
            Self::Vector { values } => {
                val.enter("AttributeData::Vector")
                    .array(values, Constraint::Vector, "values");
            }
            Self::Text { values } => {
                val.enter("AttributeData::String")
                    .array(values, Constraint::String, "values");
            }
            Self::Category {
                names,
                gradient,
                values,
                attributes,
            } => {
                val.enter("AttributeData::Category")
                    .array(values, Constraint::Index(names.item_count()), "values")
                    .array(names, Constraint::Name, "names")
                    .array_opt(gradient.as_mut(), Constraint::Gradient, "colors")
                    .array_size_opt(
                        gradient.as_ref().map(|a| a.item_count()),
                        names.item_count(),
                        "colors",
                    )
                    .objs(attributes.iter_mut())
                    .attrs_on_attribute(attributes, names.item_count());
            }
            Self::Boolean { values } => {
                val.enter("AttributeData::Boolean")
                    .array(values, Constraint::Boolean, "values");
            }
            Self::Color { values } => {
                val.enter("AttributeData::Color")
                    .array(values, Constraint::Color, "values");
            }
            Self::ProjectedTexture {
                orient,
                width,
                height,
                image,
            } => {
                val.enter("AttributeData::ProjectedTexture")
                    .obj(orient)
                    .finite(*width, "width")
                    .finite(*height, "height")
                    .above_zero(*width, "width")
                    .above_zero(*height, "height")
                    .array(image, Constraint::Image, "image");
            }
            Self::MappedTexture { texcoords, image } => {
                val.enter("AttributeData::MappedTexture")
                    .array(texcoords, Constraint::Texcoord, "texcoords")
                    .array(image, Constraint::Image, "image");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{colormap::NumberRange, Array};

    use super::*;

    #[test]
    fn serialize_attribute() {
        let mut attributes = vec![
            Attribute {
                name: "filter".to_owned(),
                description: "description of filter".to_owned(),
                units: Default::default(),
                metadata: Default::default(),
                location: Location::Vertices,
                data: AttributeData::Boolean {
                    values: Array::new("1.parquet".to_owned(), 100).into(),
                },
            },
            Attribute {
                name: "assay".to_owned(),
                description: "description of assay".to_owned(),
                units: "parts per million".to_owned(),
                metadata: Default::default(),
                location: Location::Primitives,
                data: AttributeData::Number {
                    values: Array::new("2.parquet".to_owned(), 100).into(),
                    colormap: Some(NumberColormap::Continuous {
                        range: NumberRange::Float {
                            min: 0.0,
                            max: 100.0,
                        },
                        gradient: Array::new("3.parquet".to_owned(), 128),
                    }),
                },
            },
        ];
        for a in &mut attributes {
            a.validate().unwrap();
            let s = serde_json::to_string(a).unwrap();
            let b = serde_json::from_str(&s).unwrap();
            assert_eq!(a, &b);
        }
    }
}
