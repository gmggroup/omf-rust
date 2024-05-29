use std::ptr::null;

use omf::date_time::{date_time_to_i64, date_to_i64};

use crate::{
    attributes::*,
    elements::*,
    ffi_tools::{FfiConvert, FfiStorage},
    metadata::*,
};

fn convert_metadata(
    input: impl IntoIterator<Item = (String, serde_json::Value)>,
    st: &mut FfiStorage,
) -> (*const Value, usize) {
    st.convert_iter_term(input)
}

impl FfiConvert<serde_json::Value> for Value {
    fn convert(value: serde_json::Value, st: &mut FfiStorage) -> Self {
        let mut wrap = Value::default();
        match value {
            serde_json::Value::Null => wrap.r#type = ValueType::Null,
            serde_json::Value::Bool(b) => {
                wrap.r#type = ValueType::Boolean;
                wrap.boolean = b;
            }
            serde_json::Value::Number(n) => {
                wrap.r#type = ValueType::Number;
                wrap.number = n.as_f64().unwrap_or(f64::NAN);
            }
            serde_json::Value::String(s) => {
                wrap.r#type = ValueType::String;
                wrap.string = st.keep_string(s);
            }
            serde_json::Value::Array(a) => {
                wrap.r#type = ValueType::List;
                (wrap.values, wrap.n_values) = st.convert_iter_term(a);
            }
            serde_json::Value::Object(map) => {
                wrap.r#type = ValueType::Object;
                (wrap.values, wrap.n_values) = convert_metadata(map, st);
            }
        }
        wrap
    }
}

impl FfiConvert<(String, serde_json::Value)> for Value {
    fn convert((name, value): (String, serde_json::Value), st: &mut FfiStorage) -> Self {
        let mut out: Self = Value::convert(value, st);
        out.name = st.keep_string(name);
        out
    }
}

impl FfiConvert<omf::Project> for Project {
    fn convert(project: omf::Project, st: &mut FfiStorage) -> Self {
        let (metadata, n_metadata) = convert_metadata(project.metadata, st);
        let (elements, n_elements) = st.convert_iter_term(project.elements);
        Project {
            name: st.keep_string(project.name),
            description: st.keep_string(project.description),
            coordinate_reference_system: st.keep_string(project.coordinate_reference_system),
            units: st.keep_string(project.units),
            origin: project.origin,
            author: st.keep_string(project.author),
            application: st.keep_string(project.application),
            date: project.date.timestamp_micros(),
            n_metadata,
            metadata,
            n_elements,
            elements,
        }
    }
}

impl FfiConvert<omf::Element> for Element {
    fn convert(element: omf::Element, st: &mut FfiStorage) -> Self {
        let mut wrap = Self {
            name: st.keep_string(element.name),
            description: st.keep_string(element.description),
            color_set: element.color.is_some(),
            color: element.color.unwrap_or([0; 4]),
            ..Default::default()
        };
        (wrap.metadata, wrap.n_metadata) = convert_metadata(element.metadata, st);
        (wrap.attributes, wrap.n_attributes) = st.convert_iter_term(element.attributes);
        match element.geometry {
            omf::Geometry::PointSet(p) => wrap.point_set = st.convert_ptr(p),
            omf::Geometry::LineSet(l) => wrap.line_set = st.convert_ptr(l),
            omf::Geometry::Surface(s) => wrap.surface = st.convert_ptr(s),
            omf::Geometry::GridSurface(g) => wrap.grid_surface = st.convert_ptr(g),
            omf::Geometry::BlockModel(b) => wrap.block_model = st.convert_ptr(b),
            omf::Geometry::Composite(c) => wrap.composite = st.convert_ptr(c),
        }
        wrap
    }
}

impl FfiConvert<omf::PointSet> for PointSet {
    fn convert(point_set: omf::PointSet, st: &mut FfiStorage) -> Self {
        Self {
            origin: point_set.origin,
            vertices: st.convert_ptr(point_set.vertices),
        }
    }
}

impl FfiConvert<omf::LineSet> for LineSet {
    fn convert(line_set: omf::LineSet, st: &mut FfiStorage) -> Self {
        Self {
            origin: line_set.origin,
            vertices: st.convert_ptr(line_set.vertices),
            segments: st.convert_ptr(line_set.segments),
        }
    }
}

impl FfiConvert<omf::Surface> for Surface {
    fn convert(surface: omf::Surface, st: &mut FfiStorage) -> Self {
        Self {
            origin: surface.origin,
            vertices: st.convert_ptr(surface.vertices),
            triangles: st.convert_ptr(surface.triangles),
        }
    }
}

impl FfiConvert<omf::GridSurface> for GridSurface {
    fn convert(grid_surface: omf::GridSurface, st: &mut FfiStorage) -> Self {
        let mut regular_grid = null();
        let mut tensor_grid = null();
        match grid_surface.grid {
            omf::Grid2::Regular { size, count } => {
                regular_grid = st.keep(RegularGrid2 { size, count })
            }
            omf::Grid2::Tensor { u, v } => {
                let u = st.convert_ptr(u);
                let v = st.convert_ptr(v);
                tensor_grid = st.keep(TensorGrid2 { u, v })
            }
        }
        Self {
            orient: grid_surface.orient,
            regular_grid,
            tensor_grid,
            heights: st.convert_option(grid_surface.heights),
        }
    }
}

impl FfiConvert<omf::BlockModel> for BlockModel {
    fn convert(block_model: omf::BlockModel, st: &mut FfiStorage) -> Self {
        let mut regular_grid = null();
        let mut tensor_grid = null();
        let mut regular_subblocks = null();
        let mut freeform_subblocks = null();
        // Grid.
        match block_model.grid {
            omf::Grid3::Regular { size, count } => {
                regular_grid = st.keep(RegularGrid3 { size, count });
            }
            omf::Grid3::Tensor { u, v, w } => {
                let u = st.convert_ptr(u);
                let v = st.convert_ptr(v);
                let w = st.convert_ptr(w);
                tensor_grid = st.keep(TensorGrid3 { u, v, w })
            }
        }
        // Sub-blocks.
        match block_model.subblocks {
            Some(omf::Subblocks::Regular {
                count,
                subblocks,
                mode,
            }) => {
                let subblocks = st.convert_ptr(subblocks);
                regular_subblocks = st.keep(RegularSubblocks {
                    count,
                    subblocks,
                    mode: mode.into(),
                });
            }
            Some(omf::Subblocks::Freeform { subblocks }) => {
                let subblocks = st.convert_ptr(subblocks);
                freeform_subblocks = st.keep(FreeformSubblocks { subblocks });
            }
            None => {}
        }
        Self {
            orient: block_model.orient,
            regular_grid,
            tensor_grid,
            regular_subblocks,
            freeform_subblocks,
        }
    }
}

impl FfiConvert<omf::Composite> for Composite {
    fn convert(composite: omf::Composite, st: &mut FfiStorage) -> Self {
        let (elements, n_elements) = st.convert_iter_term(composite.elements);
        Self {
            n_elements,
            elements,
        }
    }
}

fn convert_colormap(cmap: Option<omf::NumberColormap>, st: &mut FfiStorage) -> NumberData {
    match cmap {
        Some(omf::NumberColormap::Continuous { range, gradient }) => {
            let mut colormap = ContinuousColormap::default();
            match range {
                omf::NumberRange::Float { min, max } => {
                    colormap.range_type = RangeType::Float;
                    colormap.min = min;
                    colormap.max = max;
                }
                omf::NumberRange::Integer { min, max } => {
                    colormap.range_type = RangeType::Integer;
                    colormap.min_int = min;
                    colormap.max_int = max;
                }
                omf::NumberRange::Date { min, max } => {
                    colormap.range_type = RangeType::Date;
                    colormap.min_int = date_to_i64(min);
                    colormap.max_int = date_to_i64(max);
                }
                omf::NumberRange::DateTime { min, max } => {
                    colormap.range_type = RangeType::DateTime;
                    colormap.min_int = date_time_to_i64(min);
                    colormap.max_int = date_time_to_i64(max);
                }
            }
            colormap.gradient = st.convert_ptr(gradient);
            NumberData {
                continuous_colormap: st.keep(colormap),
                ..Default::default()
            }
        }
        Some(omf::NumberColormap::Discrete {
            boundaries,
            gradient,
        }) => {
            let boundaries = st.convert_ptr(boundaries);
            let gradient = st.convert_ptr(gradient);
            NumberData {
                discrete_colormap: st.keep(DiscreteColormap {
                    boundaries,
                    gradient,
                }),
                ..Default::default()
            }
        }
        None => Default::default(),
    }
}

impl FfiConvert<omf::Attribute> for Attribute {
    fn convert(attribute: omf::Attribute, st: &mut FfiStorage) -> Self {
        let mut wrap = Self {
            name: st.keep_string(attribute.name),
            description: st.keep_string(attribute.description),
            location: attribute.location,
            ..Default::default()
        };
        (wrap.metadata, wrap.n_metadata) = convert_metadata(attribute.metadata, st);
        match attribute.data {
            omf::AttributeData::Number { values, colormap } => {
                let mut number_data = convert_colormap(colormap, st);
                number_data.values = st.convert_ptr(values);
                wrap.number_data = st.keep(number_data);
            }
            omf::AttributeData::Vector { values } => wrap.vector_data = st.convert_ptr(values),
            omf::AttributeData::Text { values } => wrap.text_data = st.convert_ptr(values),
            omf::AttributeData::Category {
                values,
                names,
                gradient,
                attributes,
            } => {
                let values = st.convert_ptr(values);
                let names = st.convert_ptr(names);
                let gradient = st.convert_option(gradient);
                let (attributes, n_attributes) = st.convert_iter_term(attributes);
                wrap.category_data = st.keep(CategoryData {
                    values,
                    names,
                    gradient,
                    attributes,
                    n_attributes,
                });
            }
            omf::AttributeData::Boolean { values } => wrap.boolean_data = st.convert_ptr(values),
            omf::AttributeData::Color { values } => wrap.color_data = st.convert_ptr(values),
            omf::AttributeData::MappedTexture { image, texcoords } => {
                let image = st.convert_ptr(image);
                let texcoords = st.convert_ptr(texcoords);
                wrap.mapped_texture_data = st.keep(MappedTexture { image, texcoords });
            }
            omf::AttributeData::ProjectedTexture {
                image,
                orient,
                width,
                height,
            } => {
                let image = st.convert_ptr(image);
                wrap.projected_texture_data = st.keep(ProjectedTexture {
                    image,
                    orient,
                    width,
                    height,
                });
            }
        }
        wrap
    }
}
