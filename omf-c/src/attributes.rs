use std::{ffi::c_char, ptr::null};

use crate::{arrays::Array, metadata::Value};

#[derive(Debug)]
#[repr(C)]
pub struct Attribute {
    pub name: *const c_char,
    pub description: *const c_char,
    pub units: *const c_char,
    pub n_metadata: usize,
    pub metadata: *const Value,
    pub location: omf::Location,
    pub boolean_data: *const Array,
    pub vector_data: *const Array,
    pub text_data: *const Array,
    pub color_data: *const Array,
    pub number_data: *const NumberData,
    pub category_data: *const CategoryData,
    pub mapped_texture_data: *const MappedTexture,
    pub projected_texture_data: *const ProjectedTexture,
}

impl Default for Attribute {
    fn default() -> Self {
        Self {
            name: null(),
            description: null(),
            units: null(),
            n_metadata: 0,
            metadata: null(),
            location: Default::default(),
            number_data: null(),
            boolean_data: null(),
            vector_data: null(),
            text_data: null(),
            color_data: null(),
            category_data: null(),
            mapped_texture_data: null(),
            projected_texture_data: null(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct NumberData {
    pub values: *const Array,
    pub continuous_colormap: *const ContinuousColormap,
    pub discrete_colormap: *const DiscreteColormap,
}

impl Default for NumberData {
    fn default() -> Self {
        Self {
            values: null(),
            continuous_colormap: null(),
            discrete_colormap: null(),
        }
    }
}

#[derive(Debug, Default)]
#[repr(i32)]
pub enum RangeType {
    #[default]
    Float,
    Integer,
    Date,
    DateTime,
}

#[derive(Debug)]
#[repr(C)]
pub struct ContinuousColormap {
    pub range_type: RangeType,
    pub min: f64,
    pub max: f64,
    pub min_int: i64,
    pub max_int: i64,
    pub gradient: *const Array,
}

impl Default for ContinuousColormap {
    fn default() -> Self {
        Self {
            range_type: RangeType::Float,
            min: 0.0,
            max: 1.0,
            min_int: 0,
            max_int: 100,
            gradient: null(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct DiscreteColormap {
    pub boundaries: *const Array,
    pub gradient: *const Array,
}

impl Default for DiscreteColormap {
    fn default() -> Self {
        Self {
            boundaries: null(),
            gradient: null(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct CategoryData {
    pub values: *const Array,
    pub names: *const Array,
    pub gradient: *const Array,
    pub attributes: *const Attribute,
    pub n_attributes: usize,
}

impl Default for CategoryData {
    fn default() -> Self {
        Self {
            values: null(),
            names: null(),
            gradient: null(),
            attributes: null(),
            n_attributes: 0,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct MappedTexture {
    pub image: *const Array,
    pub texcoords: *const Array,
}

#[derive(Debug)]
#[repr(C)]
pub struct ProjectedTexture {
    pub image: *const Array,
    pub orient: omf::Orient2,
    pub width: f64,
    pub height: f64,
}

impl Default for ProjectedTexture {
    fn default() -> Self {
        Self {
            image: null(),
            orient: Default::default(),
            width: 100.0,
            height: 100.0,
        }
    }
}
