use omf::date_time::{i64_to_date, i64_to_date_time};

use crate::{
    arrays::{array_from_ptr, array_from_ptr_opt},
    attributes::*,
    elements::*,
    error::{Error, InvalidArg},
    ffi_tools::arg::{slice_from_ptr, string_from_ptr_or_null},
    metadata::Value,
};

fn option<T>(is_set: bool, value: T) -> Option<T> {
    if is_set { Some(value) } else { None }
}

impl Project {
    pub fn to_omf(&self) -> Result<omf::Project, Error> {
        let slice = unsafe {
            slice_from_ptr(
                "element.attributes",
                "element.n_attributes",
                self.elements,
                self.n_elements,
            )?
        };
        let elements = slice
            .iter()
            .map(|attr| attr.to_omf())
            .collect::<Result<_, _>>()?;
        Ok(omf::Project {
            name: unsafe { string_from_ptr_or_null("project.name", self.name) }?,
            description: unsafe {
                string_from_ptr_or_null("project.description", self.description)
            }?,
            coordinate_reference_system: unsafe {
                string_from_ptr_or_null(
                    "project.coordinate_reference_system",
                    self.coordinate_reference_system,
                )
            }?,
            units: unsafe { string_from_ptr_or_null("project.units", self.units) }?,
            origin: self.origin,
            author: unsafe { string_from_ptr_or_null("project.author", self.author) }?,
            application: unsafe {
                string_from_ptr_or_null("project.application", self.application)
            }?,
            date: i64_to_date_time(self.date),
            metadata: Value::values_as_json_map(self.metadata, self.n_metadata)?,
            elements,
        })
    }
}

impl Element {
    pub fn to_omf(&self) -> Result<omf::Element, Error> {
        let geometry = if let Some(p) = unsafe { self.point_set.as_ref() } {
            p.to_omf()?
        } else if let Some(l) = unsafe { self.line_set.as_ref() } {
            l.to_omf()?
        } else if let Some(s) = unsafe { self.surface.as_ref() } {
            s.to_omf()?
        } else if let Some(g) = unsafe { self.grid_surface.as_ref() } {
            g.to_omf()?
        } else if let Some(b) = unsafe { self.block_model.as_ref() } {
            b.to_omf()?
        } else if let Some(c) = unsafe { self.composite.as_ref() } {
            c.to_omf()?
        } else {
            return Err(InvalidArg::NoOptionSet("OmfElement geometry").into());
        };
        let slice = unsafe {
            slice_from_ptr(
                "element.attributes",
                "element.n_attributes",
                self.attributes,
                self.n_attributes,
            )?
        };
        let attributes = slice
            .iter()
            .map(|attr| attr.to_omf())
            .collect::<Result<_, _>>()?;
        Ok(omf::Element {
            name: unsafe { string_from_ptr_or_null("element.name", self.name) }?,
            description: unsafe {
                string_from_ptr_or_null("element.description", self.description)
            }?,
            color: option(self.color_set, self.color),
            metadata: Value::values_as_json_map(self.metadata, self.n_metadata)?,
            geometry,
            attributes,
        })
    }
}

impl RegularGrid2 {
    pub fn to_omf(&self) -> omf::Grid2 {
        omf::Grid2::Regular {
            size: self.size,
            count: self.count,
        }
    }
}

impl RegularGrid3 {
    pub fn to_omf(&self) -> omf::Grid3 {
        omf::Grid3::Regular {
            size: self.size,
            count: self.count,
        }
    }
}

impl TensorGrid2 {
    pub fn to_omf(&self) -> Result<omf::Grid2, Error> {
        Ok(omf::Grid2::Tensor {
            u: array_from_ptr(self.u, "OmfGrid2.u")?,
            v: array_from_ptr(self.v, "OmfGrid2.u")?,
        })
    }
}

impl TensorGrid3 {
    pub fn to_omf(&self) -> Result<omf::Grid3, Error> {
        Ok(omf::Grid3::Tensor {
            u: array_from_ptr(self.u, "OmfGrid3.u")?,
            v: array_from_ptr(self.v, "OmfGrid3.u")?,
            w: array_from_ptr(self.w, "OmfGrid3.u")?,
        })
    }
}

impl PointSet {
    pub fn to_omf(&self) -> Result<omf::Geometry, Error> {
        Ok(omf::PointSet {
            origin: self.origin,
            vertices: array_from_ptr(self.vertices, "OmfPointSet.vertices")?,
        }
        .into())
    }
}

impl LineSet {
    pub fn to_omf(&self) -> Result<omf::Geometry, Error> {
        Ok(omf::LineSet {
            origin: self.origin,
            vertices: array_from_ptr(self.vertices, "OmfLineSet.vertices")?,
            segments: array_from_ptr(self.segments, "OmfLineSet.segments")?,
        }
        .into())
    }
}

impl Surface {
    pub fn to_omf(&self) -> Result<omf::Geometry, Error> {
        Ok(omf::Surface {
            origin: self.origin,
            vertices: array_from_ptr(self.vertices, "OmfSurface.vertices")?,
            triangles: array_from_ptr(self.triangles, "OmfSurface.triangles")?,
        }
        .into())
    }
}

impl GridSurface {
    pub fn to_omf(&self) -> Result<omf::Geometry, Error> {
        let grid = if let Some(reg) = unsafe { self.regular_grid.as_ref() } {
            reg.to_omf()
        } else if let Some(ten) = unsafe { self.tensor_grid.as_ref() } {
            ten.to_omf()?
        } else {
            return Err(InvalidArg::NoOptionSet("OmfGridSurface.*_grid").into());
        };
        Ok(omf::GridSurface {
            orient: self.orient,
            grid,
            heights: array_from_ptr_opt(self.heights, "GridSurface.heights")?,
        }
        .into())
    }
}

impl Composite {
    pub fn to_omf(&self) -> Result<omf::Geometry, Error> {
        let slice = unsafe {
            slice_from_ptr(
                "composite.elements",
                "composite.n_elements",
                self.elements,
                self.n_elements,
            )?
        };
        let elements = slice.iter().map(|e| e.to_omf()).collect::<Result<_, _>>()?;
        Ok(omf::Composite { elements }.into())
    }
}

impl BlockModel {
    pub fn to_omf(&self) -> Result<omf::Geometry, Error> {
        let grid = if let Some(reg) = unsafe { self.regular_grid.as_ref() } {
            reg.to_omf()
        } else if let Some(ten) = unsafe { self.tensor_grid.as_ref() } {
            ten.to_omf()?
        } else {
            return Err(InvalidArg::NoOptionSet("OmfBlockModel.*_grid").into());
        };
        let subblocks = if let Some(reg) = unsafe { self.regular_subblocks.as_ref() } {
            Some(reg.to_omf()?)
        } else if let Some(free) = unsafe { self.freeform_subblocks.as_ref() } {
            Some(free.to_omf()?)
        } else {
            None
        };
        Ok(omf::BlockModel {
            orient: self.orient,
            grid,
            subblocks,
        }
        .into())
    }
}

impl RegularSubblocks {
    pub fn to_omf(&self) -> Result<omf::Subblocks, Error> {
        Ok(omf::Subblocks::Regular {
            count: self.count,
            subblocks: array_from_ptr(self.subblocks, "OmfRegularSubblocks.subblocks")?,
            mode: self.mode.into(),
        })
    }
}

impl FreeformSubblocks {
    pub fn to_omf(&self) -> Result<omf::Subblocks, Error> {
        Ok(omf::Subblocks::Freeform {
            subblocks: array_from_ptr(self.subblocks, "OmfFreeformSubblocks.subblocks")?,
        })
    }
}

impl Attribute {
    pub fn to_omf(&self) -> Result<omf::Attribute, Error> {
        let data = if let Some(arr) = unsafe { self.boolean_data.as_ref() } {
            omf::AttributeData::Boolean {
                values: array_from_ptr(arr, "OmfAttribute.boolean_data")?,
            }
        } else if let Some(arr) = unsafe { self.vector_data.as_ref() } {
            omf::AttributeData::Vector {
                values: array_from_ptr(arr, "OmfAttribute.vector_data")?,
            }
        } else if let Some(arr) = unsafe { self.text_data.as_ref() } {
            omf::AttributeData::Text {
                values: array_from_ptr(arr, "OmfAttribute.text_data")?,
            }
        } else if let Some(arr) = unsafe { self.color_data.as_ref() } {
            omf::AttributeData::Color {
                values: array_from_ptr(arr, "OmfAttribute.color_data")?,
            }
        } else if let Some(n) = unsafe { self.number_data.as_ref() } {
            let colormap = if let Some(c) = unsafe { n.continuous_colormap.as_ref() } {
                Some(c.to_omf()?)
            } else if let Some(d) = unsafe { n.discrete_colormap.as_ref() } {
                Some(d.to_omf()?)
            } else {
                None
            };
            omf::AttributeData::Number {
                values: array_from_ptr(n.values, "OmfNumberData.values")?,
                colormap,
            }
        } else if let Some(cat) = unsafe { self.category_data.as_ref() } {
            let attributes = unsafe {
                slice_from_ptr(
                    "CategoryData.attributes",
                    "CategoryData.n_attributes",
                    cat.attributes,
                    cat.n_attributes,
                )
            }?;
            omf::AttributeData::Category {
                names: array_from_ptr(cat.names, "OmfCategoryData.names")?,
                values: array_from_ptr(cat.values, "OmfCategoryData.names")?,
                gradient: array_from_ptr_opt(cat.gradient, "OmfCategoryData.gradient")?,
                attributes: attributes
                    .iter()
                    .map(|a| a.to_omf())
                    .collect::<Result<_, _>>()?,
            }
        } else if let Some(map) = unsafe { self.mapped_texture_data.as_ref() } {
            omf::AttributeData::MappedTexture {
                image: array_from_ptr(map.image, "OmfMappedTexture.image")?,
                texcoords: array_from_ptr(map.texcoords, "OmfMappedTexture.texcoords")?,
            }
        } else if let Some(proj) = unsafe { self.projected_texture_data.as_ref() } {
            omf::AttributeData::ProjectedTexture {
                image: array_from_ptr(proj.image, "OmfProjectedTexture.image")?,
                orient: proj.orient,
                width: proj.width,
                height: proj.height,
            }
        } else {
            return Err(InvalidArg::NoOptionSet("OmfAttribute.*_data").into());
        };
        Ok(omf::Attribute {
            name: unsafe { string_from_ptr_or_null("attribute name", self.name) }?,
            description: unsafe {
                string_from_ptr_or_null("attribute description", self.description)
            }?,
            units: unsafe { string_from_ptr_or_null("attribute units", self.units) }?,
            metadata: Value::values_as_json_map(self.metadata, self.n_metadata)?,
            location: self.location,
            data,
        })
    }
}

impl ContinuousColormap {
    pub fn to_omf(&self) -> Result<omf::NumberColormap, Error> {
        let range = match self.range_type {
            RangeType::Integer => (self.min_int, self.max_int).into(),
            RangeType::Date => (i64_to_date(self.min_int), i64_to_date(self.max_int)).into(),
            RangeType::DateTime => (
                i64_to_date_time(self.min_int),
                i64_to_date_time(self.max_int),
            )
                .into(),
            _ => (self.min, self.max).into(),
        };
        Ok(omf::NumberColormap::Continuous {
            range,
            gradient: array_from_ptr(self.gradient, "ContinuousColormap.gradient")?,
        })
    }
}

impl DiscreteColormap {
    pub fn to_omf(&self) -> Result<omf::NumberColormap, Error> {
        Ok(omf::NumberColormap::Discrete {
            boundaries: array_from_ptr(self.boundaries, "OmfDiscreteColormap.boundaries")?,
            gradient: array_from_ptr(self.gradient, "OmfDiscreteColormap.gradient")?,
        })
    }
}
