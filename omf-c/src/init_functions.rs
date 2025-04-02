use std::ffi::c_char;

use crate::{arrays::Array, attributes::*, elements::*};

macro_rules! create (
    ($ty:ty) => {
        <$ty>::default()
    };
    ($ty:ty $( , $field:ident )* ) => {
        {
            let mut obj = <$ty>::default();
            $( obj.$field = $field; )*
            obj
        }
    };
);

#[unsafe(no_mangle)]
pub extern "C" fn omf_project_init(name: *const c_char) -> Project {
    create!(Project, name)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_element_init(name: *const c_char) -> Element {
    create!(Element, name)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_attribute_init(name: *const c_char, location: omf::Location) -> Attribute {
    create!(Attribute, name, location)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_point_set_init(vertices: *const Array) -> PointSet {
    create!(PointSet, vertices)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_line_set_init(vertices: *const Array, segments: *const Array) -> LineSet {
    create!(LineSet, vertices, segments)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_surface_init(vertices: *const Array, triangles: *const Array) -> Surface {
    create!(Surface, vertices, triangles)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_grid_surface_init() -> GridSurface {
    create!(GridSurface)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_block_model_init() -> BlockModel {
    create!(BlockModel)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_composite_init() -> Composite {
    create!(Composite)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_number_data_init() -> NumberData {
    create!(NumberData)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_category_data_init() -> CategoryData {
    create!(CategoryData)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_discrete_colormap_init() -> DiscreteColormap {
    create!(DiscreteColormap)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_continuous_colormap_init(
    min: f64,
    max: f64,
    gradient: *const Array,
) -> ContinuousColormap {
    create!(ContinuousColormap, min, max, gradient)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_tensor_grid2_init(u: *const Array, v: *const Array) -> TensorGrid2 {
    create!(TensorGrid2, u, v)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_tensor_grid3_init(
    u: *const Array,
    v: *const Array,
    w: *const Array,
) -> TensorGrid3 {
    create!(TensorGrid3, u, v, w)
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_regular_grid2_init(du: f64, dv: f64, nu: u32, nv: u32) -> RegularGrid2 {
    RegularGrid2 {
        size: [du, dv],
        count: [nu, nv],
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_regular_grid3_init(
    du: f64,
    dv: f64,
    dw: f64,
    nu: u32,
    nv: u32,
    nw: u32,
) -> RegularGrid3 {
    RegularGrid3 {
        size: [du, dv, dw],
        count: [nu, nv, nw],
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_regular_subblocks_init(
    nu: u32,
    nv: u32,
    nw: u32,
    subblocks: *const Array,
) -> RegularSubblocks {
    RegularSubblocks {
        count: [nu, nv, nw],
        subblocks,
        mode: SubblockMode::None,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_freeform_subblocks_init(subblocks: *const Array) -> FreeformSubblocks {
    FreeformSubblocks { subblocks }
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_mapped_texture_init(
    image: *const Array,
    texcoords: *const Array,
) -> MappedTexture {
    MappedTexture { image, texcoords }
}

#[unsafe(no_mangle)]
pub extern "C" fn omf_projected_texture_init(image: *const Array) -> ProjectedTexture {
    create!(ProjectedTexture, image)
}
