use std::{ffi::c_char, ptr::null};

use crate::{arrays::Array, attributes::Attribute, metadata::Value};

#[derive(Debug, Default)]
#[repr(C)]
pub struct FileVersion {
    pub major: u32,
    pub minor: u32,
}

impl From<[u32; 2]> for FileVersion {
    fn from([major, minor]: [u32; 2]) -> Self {
        Self { major, minor }
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct Limits {
    pub json_bytes: u64,
    pub image_bytes: u64,
    pub image_dim: u32,
    pub validation: u32,
}

impl From<omf::file::Limits> for Limits {
    fn from(value: omf::file::Limits) -> Self {
        Limits {
            json_bytes: value.json_bytes.unwrap_or(0),
            image_bytes: value.image_bytes.unwrap_or(0),
            image_dim: value.image_dim.unwrap_or(0),
            validation: value.validation.unwrap_or(0),
        }
    }
}

impl From<Limits> for omf::file::Limits {
    fn from(value: Limits) -> Self {
        fn none_if_default<T: Default + PartialEq>(x: T) -> Option<T> {
            if x == Default::default() {
                Some(x)
            } else {
                None
            }
        }
        Self {
            json_bytes: none_if_default(value.json_bytes),
            image_bytes: none_if_default(value.image_bytes),
            image_dim: none_if_default(value.image_dim),
            validation: none_if_default(value.validation),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Project {
    pub name: *const c_char,
    pub description: *const c_char,
    pub coordinate_reference_system: *const c_char,
    pub units: *const c_char,
    pub author: *const c_char,
    pub application: *const c_char,
    pub date: i64,
    pub origin: [f64; 3],
    pub n_metadata: usize,
    pub metadata: *const Value,
    pub n_elements: usize,
    pub elements: *const Element,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: null(),
            description: null(),
            coordinate_reference_system: null(),
            units: null(),
            author: null(),
            application: null(),
            date: omf::date_time::utc_now().timestamp_micros(),
            origin: [0.0, 0.0, 0.0],
            n_metadata: 0,
            metadata: null(),
            n_elements: 0,
            elements: null(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Element {
    pub name: *const c_char,
    pub description: *const c_char,
    pub color_set: bool,
    pub color: [u8; 4],
    pub n_metadata: usize,
    pub metadata: *const Value,
    pub n_attributes: usize,
    pub attributes: *const Attribute,
    pub point_set: *const PointSet,
    pub line_set: *const LineSet,
    pub surface: *const Surface,
    pub grid_surface: *const GridSurface,
    pub block_model: *const BlockModel,
    pub composite: *const Composite,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            name: null(),
            description: null(),
            color_set: false,
            color: [0, 0, 0, u8::MAX],
            n_metadata: 0,
            metadata: null(),
            n_attributes: 0,
            attributes: null(),
            point_set: null(),
            line_set: null(),
            surface: null(),
            grid_surface: null(),
            block_model: null(),
            composite: null(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct PointSet {
    pub origin: [f64; 3],
    pub vertices: *const Array,
}

impl Default for PointSet {
    fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            vertices: null(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct LineSet {
    pub origin: [f64; 3],
    pub vertices: *const Array,
    pub segments: *const Array,
}

impl Default for LineSet {
    fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            vertices: null(),
            segments: null(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Surface {
    pub origin: [f64; 3],
    pub vertices: *const Array,
    pub triangles: *const Array,
}

impl Default for Surface {
    fn default() -> Self {
        Self {
            origin: [0.0, 0.0, 0.0],
            vertices: null(),
            triangles: null(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct GridSurface {
    pub orient: omf::Orient2,
    pub regular_grid: *const RegularGrid2,
    pub tensor_grid: *const TensorGrid2,
    pub heights: *const Array,
}

impl Default for GridSurface {
    fn default() -> Self {
        Self {
            orient: Default::default(),
            regular_grid: null(),
            tensor_grid: null(),
            heights: null(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct BlockModel {
    pub orient: omf::Orient3,
    pub regular_grid: *const RegularGrid3,
    pub tensor_grid: *const TensorGrid3,
    pub regular_subblocks: *const RegularSubblocks,
    pub freeform_subblocks: *const FreeformSubblocks,
}

impl Default for BlockModel {
    fn default() -> Self {
        Self {
            orient: Default::default(),
            regular_grid: null(),
            tensor_grid: null(),
            regular_subblocks: null(),
            freeform_subblocks: null(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Composite {
    pub n_elements: usize,
    pub elements: *const Element,
}

impl Default for Composite {
    fn default() -> Self {
        Self {
            n_elements: 0,
            elements: null(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
#[repr(i32)]
pub enum SubblockMode {
    None = 0,
    Octree,
    Full,
}

impl From<Option<omf::SubblockMode>> for SubblockMode {
    fn from(value: Option<omf::SubblockMode>) -> Self {
        match value {
            Some(omf::SubblockMode::Full) => Self::Full,
            Some(omf::SubblockMode::Octree) => Self::Octree,
            None => Self::None,
        }
    }
}

impl From<SubblockMode> for Option<omf::SubblockMode> {
    fn from(value: SubblockMode) -> Self {
        match value {
            SubblockMode::Octree => Some(omf::SubblockMode::Octree),
            SubblockMode::Full => Some(omf::SubblockMode::Full),
            _ => None,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct RegularSubblocks {
    pub count: [u32; 3],
    pub subblocks: *const Array,
    pub mode: SubblockMode,
}

#[derive(Debug)]
#[repr(C)]
pub struct FreeformSubblocks {
    pub subblocks: *const Array,
}

#[derive(Debug)]
#[repr(C)]
pub struct RegularGrid2 {
    pub size: [f64; 2],
    pub count: [u32; 2],
}

impl Default for RegularGrid2 {
    fn default() -> Self {
        Self {
            size: [1.0, 1.0],
            count: [10, 10],
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TensorGrid2 {
    pub u: *const Array,
    pub v: *const Array,
}

impl Default for TensorGrid2 {
    fn default() -> Self {
        Self {
            u: null(),
            v: null(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct RegularGrid3 {
    pub size: [f64; 3],
    pub count: [u32; 3],
}

impl Default for RegularGrid3 {
    fn default() -> Self {
        Self {
            size: [1.0, 1.0, 1.0],
            count: [10, 10, 10],
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TensorGrid3 {
    pub u: *const Array,
    pub v: *const Array,
    pub w: *const Array,
}

impl Default for TensorGrid3 {
    fn default() -> Self {
        Self {
            u: null(),
            v: null(),
            w: null(),
        }
    }
}
