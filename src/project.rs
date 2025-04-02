use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    Element, Vector3,
    date_time::utc_now,
    geometry::zero_origin,
    validate::{Validate, Validator},
};

/// Root object of an OMF file.
///
/// This is the root element of an OMF file, holding global metadata and a list of
/// [Elements](crate::Element) that describe the objects or shapes within the file.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Project {
    /// Project name.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Optional project description.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Optional [EPSG](https://epsg.io/) or [PROJ](https://proj.org/) local transformation
    /// string, default empty.
    ///
    /// Exactly what is supported depends on the application reading the file.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub coordinate_reference_system: String,
    /// Optional unit for distances and locations within the file.
    ///
    /// Typically "meters", "metres", "feet", or empty because the coordinate reference system
    /// defines it. If both are empty then applications may assume meters.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub units: String,
    /// Optional project origin, default [0, 0, 0].
    ///
    /// Most geometries also have their own origin field. To get the real location add this
    /// origin and the geometry origin to all locations within each element.
    #[serde(default, skip_serializing_if = "zero_origin")]
    pub origin: Vector3,
    /// Optional name or email address of the person that created the file, default empty.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub author: String,
    /// Optional name and version of the application that created the file, default empty.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub application: String,
    /// File or data creation date. Defaults to the current date and time on creation.
    pub date: DateTime<Utc>,
    /// Arbitrary metadata.
    ///
    /// This is the place to put anything that doesn't fit in the other fields.
    /// Application-specific data should use a prefix that identifies the application, like
    /// `"lf-something"` for Leapfrog.
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub metadata: serde_json::Map<String, Value>,
    /// List of elements.
    #[serde(default)]
    pub elements: Vec<Element>,
}

impl Project {
    /// Create a new project with just the name set.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: Default::default(),
            description: Default::default(),
            coordinate_reference_system: Default::default(),
            units: Default::default(),
            origin: Default::default(),
            author: Default::default(),
            application: Default::default(),
            date: utc_now(),
            metadata: Default::default(),
            elements: Default::default(),
        }
    }
}

impl Validate for Project {
    fn validate_inner(&mut self, val: &mut Validator) {
        val.enter("Project")
            .name(&self.name)
            .finite_seq(self.origin, "origin")
            .objs(&mut self.elements)
            .unique(
                self.elements.iter().map(|e| &e.name),
                "elements[..]::name",
                false,
            );
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use serde_json::Value;

    use super::*;

    #[test]
    fn serde_empty_project() {
        let mut p = Project::new("Test");
        p.name = "Foo".to_owned();
        p.units = "meters".to_owned();
        p.origin = [1e6, 0.0, 0.0];
        p.date = chrono::DateTime::from_str("2022-10-31T09:00:00.594Z").unwrap();
        p.metadata.insert("other".to_owned(), Value::Bool(true));
        let s = serde_json::to_string(&p).unwrap();
        let q = serde_json::from_str(&s).unwrap();
        assert_eq!(p, q);
        assert_eq!(
            s,
            concat!(
                r#"{"name":"Foo","units":"meters","origin":[1000000.0,0.0,0.0],"#,
                r#""date":"2022-10-31T09:00:00.594Z","metadata":{"other":true},"#,
                r#""elements":[]}"#
            )
        );
    }
}
