use crate::{
    error::{Error, InvalidArg},
    ffi_tools::arg::not_null,
};

#[derive(Debug, Clone)]
pub enum HandleComponent {
    Nested { key: String },
    Array { index: usize },
    Element { index: usize },
    Attribute { index: usize },
}

pub enum HandleMetadata<'a> {
    Map(&'a mut serde_json::Map<String, serde_json::Value>),
    Vec(&'a mut Vec<serde_json::Value>),
}

#[derive(Debug, Clone, Default)]
pub struct Handle(Vec<HandleComponent>);

impl Handle {
    pub fn from_ptr(handle: *mut Handle) -> Result<&'static Self, Error> {
        not_null!(handle)
    }

    pub fn join(&self, comp: HandleComponent) -> Self {
        let mut new_handle = self.clone();
        new_handle.0.push(comp);
        new_handle
    }

    pub fn metadata<'a>(&self, project: &'a mut omf::Project) -> Result<HandleMetadata<'a>, Error> {
        match HandleObject::new(project, self)? {
            HandleObject::Project(omf::Project { metadata, .. })
            | HandleObject::Element(omf::Element { metadata, .. })
            | HandleObject::Attribute(omf::Attribute { metadata, .. })
            | HandleObject::MetadataMap(metadata) => Ok(HandleMetadata::Map(metadata)),
            HandleObject::MetadataVec(vec) => Ok(HandleMetadata::Vec(vec)),
        }
    }

    pub fn elements<'a>(
        &self,
        project: &'a mut omf::Project,
    ) -> Result<&'a mut Vec<omf::Element>, Error> {
        match HandleObject::new(project, self)? {
            HandleObject::Project(omf::Project { elements, .. })
            | HandleObject::Element(omf::Element {
                geometry: omf::Geometry::Composite(omf::Composite { elements, .. }),
                ..
            }) => Ok(elements),
            _ => Err(
                InvalidArg::HandleType("a project or composite element handle is required").into(),
            ),
        }
    }

    pub fn attributes<'a>(
        &self,
        project: &'a mut omf::Project,
    ) -> Result<&'a mut Vec<omf::Attribute>, Error> {
        match HandleObject::new(project, self)? {
            HandleObject::Element(omf::Element { attributes, .. }) => Ok(attributes),
            HandleObject::Attribute(omf::Attribute {
                data: omf::AttributeData::Category { attributes, .. },
                ..
            }) => Ok(attributes),
            _ => Err(
                InvalidArg::HandleType("an element or category attribute handle is required")
                    .into(),
            ),
        }
    }
}

enum HandleObject<'a> {
    Project(&'a mut omf::Project),
    MetadataMap(&'a mut serde_json::Map<String, serde_json::Value>),
    MetadataVec(&'a mut Vec<serde_json::Value>),
    Element(&'a mut omf::Element),
    Attribute(&'a mut omf::Attribute),
}

impl<'a> HandleObject<'a> {
    fn new(project: &'a mut omf::Project, handle: &Handle) -> Result<Self, Error> {
        let mut obj = Self::Project(project);
        for comp in &handle.0 {
            obj = obj.next(comp).ok_or(InvalidArg::Handle)?;
        }
        Ok(obj)
    }

    fn next(self, comp: &HandleComponent) -> Option<Self> {
        match (&comp, self) {
            // nested metadata
            (
                HandleComponent::Nested { key },
                Self::MetadataMap(metadata)
                | Self::Project(omf::Project { metadata, .. })
                | Self::Element(omf::Element { metadata, .. })
                | Self::Attribute(omf::Attribute { metadata, .. }),
            ) => match metadata.get_mut(key)? {
                serde_json::Value::Array(v) => Some(Self::MetadataVec(v)),
                serde_json::Value::Object(m) => Some(Self::MetadataMap(m)),
                _ => None,
            },
            // nested metadata within array
            (HandleComponent::Array { index }, Self::MetadataVec(vec)) => {
                match vec.get_mut(*index)? {
                    serde_json::Value::Array(v) => Some(Self::MetadataVec(v)),
                    serde_json::Value::Object(m) => Some(Self::MetadataMap(m)),
                    _ => None,
                }
            }
            // element within project
            (HandleComponent::Element { index }, Self::Project(p)) => {
                Some(Self::Element(p.elements.get_mut(*index)?))
            }
            // element within composite element
            (
                HandleComponent::Element { index },
                Self::Element(omf::Element {
                    geometry: omf::Geometry::Composite(c),
                    ..
                }),
            ) => Some(Self::Element(c.elements.get_mut(*index)?)),
            // attribute within element
            (HandleComponent::Attribute { index }, Self::Element(e)) => {
                Some(Self::Attribute(e.attributes.get_mut(*index)?))
            }
            // attribute within category attribute
            (
                HandleComponent::Attribute { index },
                Self::Attribute(omf::Attribute {
                    data: omf::AttributeData::Category { attributes, .. },
                    ..
                }),
            ) => Some(Self::Attribute(attributes.get_mut(*index)?)),
            // otherwise invalid
            _ => None,
        }
    }
}
