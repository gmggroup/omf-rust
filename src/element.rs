use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    validate::{Validate, Validator},
    Attribute, Color, Geometry, Location,
};

/// Defines a single "object" or "shape" within the OMF file.
///
/// Each shape has a name plus other optional metadata, a "geometry" that describes
/// a point-set, surface, etc., and a list of attributes that that exist on that geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Element {
    /// The element name. Names should be non-empty and unique.
    pub name: String,
    /// Optional element description.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Optional solid color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    /// Arbitrary metadata.
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub metadata: serde_json::Map<String, Value>,
    /// List of attributes, if any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<Attribute>,
    /// The geometry of the element.
    pub geometry: Geometry,
}

impl Element {
    /// Create a new element with the given name and geometry.
    ///
    /// Geometries will be automatically converted from their objects into enum variants.
    pub fn new(name: impl Into<String>, geometry: impl Into<Geometry>) -> Self {
        Self {
            name: name.into(),
            description: Default::default(),
            metadata: Default::default(),
            attributes: Default::default(),
            geometry: geometry.into(),
            color: None,
        }
    }

    /// Returns the valid locations for attributes on this element.
    pub fn valid_locations(&self) -> &'static [Location] {
        self.geometry.valid_locations()
    }

    /// Returns the number of values needed for the given location.
    pub fn location_len(&self, location: Location) -> Option<u64> {
        self.geometry.location_len(location)
    }
}

impl Validate for Element {
    fn validate_inner(&mut self, val: &mut Validator) {
        val.enter("Element")
            .name(&self.name)
            .obj(&mut self.geometry)
            .objs(&mut self.attributes)
            .unique(
                self.attributes.iter().map(|a| &a.name),
                "attributes[..]::name",
                false,
            )
            .attrs_on_geometry(&self.attributes, &self.geometry);
    }
}
