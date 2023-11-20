use crate::{error::Error, file::Writer};

use super::{
    array::{color_array, index_array, numbers_array, vectors2_array, vectors3_array},
    category_handler::CategoryHandler,
    model::ColorArrayModel,
    objects::*,
    reader::Omf1Reader,
};

impl ScalarData {
    pub fn convert(&self, r: &Omf1Reader, w: &mut Writer) -> Result<crate::Attribute, Error> {
        attribute(
            &self.content,
            self.location,
            crate::AttributeData::Number {
                values: numbers_array(r, w, &self.array)?,
                colormap: self
                    .colormap
                    .as_ref()
                    .map(|key| r.model(key)?.convert(r, w))
                    .transpose()?,
            },
        )
    }
}

impl DateTimeData {
    pub fn convert(&self, r: &Omf1Reader, w: &mut Writer) -> Result<crate::Attribute, Error> {
        attribute(
            &self.content,
            self.location,
            crate::AttributeData::Number {
                values: w.array_numbers(r.model(&self.array)?.array.iter().copied())?,
                colormap: self
                    .colormap
                    .as_ref()
                    .map(|key| r.model(key)?.convert(r, w))
                    .transpose()?,
            },
        )
    }
}

impl Vector2Data {
    pub fn convert(&self, r: &Omf1Reader, w: &mut Writer) -> Result<crate::Attribute, Error> {
        attribute(
            &self.content,
            self.location,
            crate::AttributeData::Vector {
                values: vectors2_array(r, w, &self.array)?,
            },
        )
    }
}

impl Vector3Data {
    pub fn convert(&self, r: &Omf1Reader, w: &mut Writer) -> Result<crate::Attribute, Error> {
        attribute(
            &self.content,
            self.location,
            crate::AttributeData::Vector {
                values: vectors3_array(r, w, &self.array)?,
            },
        )
    }
}

impl ColorData {
    pub fn convert(&self, r: &Omf1Reader, w: &mut Writer) -> Result<crate::Attribute, Error> {
        match r.model(&self.array)? {
            ColorArrayModel::Int3Array(array) => attribute(
                &self.content,
                self.location,
                crate::AttributeData::Color {
                    values: color_array(r, w, array)?,
                },
            ),
            ColorArrayModel::ColorArray(ColorArray { array, .. }) => attribute(
                &self.content,
                self.location,
                crate::AttributeData::Color {
                    values: w
                        .array_colors(array.iter().map(|&[r, g, b]| Some([r, g, b, u8::MAX])))?,
                },
            ),
        }
    }
}

impl StringData {
    pub fn convert(&self, r: &Omf1Reader, w: &mut Writer) -> Result<crate::Attribute, Error> {
        let strings = &r.model(&self.array)?.array;
        attribute(
            &self.content,
            self.location,
            crate::AttributeData::Text {
                values: w.array_text(strings.iter().cloned().map(|s| {
                    if s.is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                }))?,
            },
        )
    }
}

impl MappedData {
    pub fn convert(&self, r: &Omf1Reader, w: &mut Writer) -> Result<crate::Attribute, Error> {
        let (max_index, values) = index_array(r, w, &self.array)?;
        let mut handler = CategoryHandler::new(max_index);
        for key in &self.legends {
            let legend = r.model(key)?;
            handler.add(
                r,
                &legend.content.name,
                &legend.content.description,
                &legend.values,
            )?;
        }
        attribute(&self.content, self.location, handler.write(w, values)?)
    }
}

impl ImageTexture {
    pub fn convert(&self, r: &Omf1Reader, w: &mut Writer) -> Result<crate::Attribute, Error> {
        let (u, width) = projection_axis(self.axis_u);
        let (v, height) = projection_axis(self.axis_v);
        Ok(crate::Attribute {
            name: self.content.name.clone(),
            description: self.content.description.clone(),
            units: Default::default(),
            metadata: self.content.uid.metadata(),
            location: crate::Location::Projected,
            data: crate::AttributeData::ProjectedTexture {
                image: w.image_bytes_from(r.image(&self.image)?)?,
                orient: crate::Orient2 {
                    origin: self.origin,
                    u,
                    v,
                },
                width,
                height,
            },
        })
    }
}

impl ScalarColormap {
    pub fn convert(&self, r: &Omf1Reader, w: &mut Writer) -> Result<crate::NumberColormap, Error> {
        let [min, max] = self.limits;
        Ok(crate::NumberColormap::Continuous {
            range: (min, max).into(),
            gradient: gradient(r, w, &self.gradient)?,
        })
    }
}

impl DateTimeColormap {
    pub fn convert(&self, r: &Omf1Reader, w: &mut Writer) -> Result<crate::NumberColormap, Error> {
        let [min, max] = self.limits;
        Ok(crate::NumberColormap::Continuous {
            range: (min, max).into(),
            gradient: gradient(r, w, &self.gradient)?,
        })
    }
}

fn gradient(
    r: &Omf1Reader,
    w: &mut Writer,
    colors: &Key<ColorArray>,
) -> Result<crate::Array<crate::array_type::Gradient>, Error> {
    w.array_gradient(
        r.model(colors)?
            .array
            .iter()
            .map(|&[r, g, b]| [r, g, b, u8::MAX]),
    )
}

pub(super) fn attribute(
    content: &ContentModel,
    location: DataLocation,
    data: crate::AttributeData,
) -> Result<crate::Attribute, Error> {
    Ok(crate::Attribute {
        name: content.name.clone(),
        description: content.description.clone(),
        units: Default::default(),
        metadata: content.uid.metadata(),
        location: match location {
            DataLocation::Vertices => crate::Location::Vertices,
            DataLocation::Segments | DataLocation::Faces | DataLocation::Cells => {
                crate::Location::Primitives
            }
        },
        data,
    })
}

fn projection_axis(axis: [f64; 3]) -> ([f64; 3], f64) {
    let length: f64 = axis.iter().map(|x| x * x).sum();
    if length == 0.0 {
        ([0.0, 0.0, 0.0], 0.0)
    } else {
        (axis.map(|x| x / length), length)
    }
}
