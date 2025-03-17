use std::io::{Seek, Write};

use crate::{
    crate_full_name,
    date_time::utc_now,
    error::Error,
    file::{ReadAt, Writer},
};

use super::{
    array::{scalars_array, segments_array, triangles_array, vertices_array},
    model::*,
    objects::*,
    reader::Omf1Reader,
};

impl UidModel {
    pub fn metadata(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        insert_metadata(&mut map, "date_created", &self.date_created);
        insert_metadata(&mut map, "date_modified", &self.date_modified);
        map
    }
}

impl Project {
    pub fn convert<W: Write + Seek + Send, R: ReadAt>(
        &self,
        r: &Omf1Reader<R>,
        w: &mut Writer<W>,
    ) -> Result<crate::Project, Error> {
        let mut conversion_details = serde_json::Map::new();
        conversion_details.insert("from".to_owned(), r.version().into());
        conversion_details.insert("by".to_owned(), crate_full_name().into());
        conversion_details.insert(
            "on".to_owned(),
            utc_now()
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                .into(),
        );
        let mut metadata = self.content.uid.metadata();
        metadata.insert("OMF1 conversion".to_owned(), conversion_details.into());
        Ok(crate::Project {
            name: self.content.name.clone(),
            description: self.content.description.clone(),
            coordinate_reference_system: Default::default(),
            units: self.units.clone(),
            origin: self.origin,
            author: self.author.clone(),
            application: Default::default(),
            date: self
                .content
                .uid
                .date_created
                .parse()
                .unwrap_or_else(|_| utc_now()),
            metadata,
            elements: self
                .elements
                .iter()
                .map(|k| {
                    let element = r.model(k)?;
                    element.convert(r, w)
                })
                .collect::<Result<_, _>>()?,
        })
    }
}

impl ElementModel<'_> {
    pub fn convert<W: Write + Seek + Send, R: ReadAt>(
        &self,
        r: &Omf1Reader<R>,
        w: &mut Writer<W>,
    ) -> Result<crate::Element, Error> {
        match *self {
            Self::PointSetElement(x) => x.convert(r, w),
            Self::LineSetElement(x) => x.convert(r, w),
            Self::SurfaceElement(x) => x.convert(r, w),
            Self::VolumeElement(x) => x.convert(r, w),
        }
    }
}

impl PointSetElement {
    pub fn convert<W: Write + Seek + Send, R: ReadAt>(
        &self,
        r: &Omf1Reader<R>,
        w: &mut Writer<W>,
    ) -> Result<crate::Element, Error> {
        let geometry = r.model(&self.geometry)?;
        let mut e = element(
            &self.content,
            self.color,
            attributes_and_textures(r, w, &self.data, &self.textures)?,
            crate::PointSet {
                origin: geometry.origin,
                vertices: vertices_array(r, w, &geometry.vertices)?,
            },
        )?;
        e.metadata.insert(
            "subtype".to_owned(),
            match self.subtype {
                PointSetSubtype::Point => "point",
                PointSetSubtype::Collar => "collar",
                PointSetSubtype::BlastHole => "blasthole",
            }
            .into(),
        );
        Ok(e)
    }
}

impl LineSetElement {
    pub fn convert<W: Write + Seek + Send, R: ReadAt>(
        &self,
        r: &Omf1Reader<R>,
        w: &mut Writer<W>,
    ) -> Result<crate::Element, Error> {
        let geometry = r.model(&self.geometry)?;
        let mut e = element(
            &self.content,
            self.color,
            attributes(r, w, &self.data)?,
            crate::LineSet {
                origin: geometry.origin,
                vertices: vertices_array(r, w, &geometry.vertices)?,
                segments: segments_array(r, w, &geometry.segments)?,
            },
        )?;
        e.metadata.insert(
            "subtype".to_owned(),
            match self.subtype {
                LineSetSubtype::Line => "line",
                LineSetSubtype::BoreHole => "borehole",
            }
            .into(),
        );
        Ok(e)
    }
}

impl SurfaceElement {
    pub fn convert<W: Write + Seek + Send, R: ReadAt>(
        &self,
        r: &Omf1Reader<R>,
        w: &mut Writer<W>,
    ) -> Result<crate::Element, Error> {
        match r.model(&self.geometry)? {
            SurfaceGeometryModel::SurfaceGeometry(geometry) => element(
                &self.content,
                self.color,
                attributes_and_textures(r, w, &self.data, &self.textures)?,
                crate::Surface {
                    origin: geometry.origin,
                    vertices: vertices_array(r, w, &geometry.vertices)?,
                    triangles: triangles_array(r, w, &geometry.triangles)?,
                },
            ),
            SurfaceGeometryModel::SurfaceGridGeometry(geometry) => element(
                &self.content,
                self.color,
                attributes(r, w, &self.data)?,
                crate::GridSurface {
                    orient: crate::Orient2 {
                        origin: geometry.origin,
                        u: geometry.axis_u,
                        v: geometry.axis_v,
                    },
                    grid: crate::Grid2::Tensor {
                        u: w.array_scalars(geometry.tensor_u.iter().copied())?,
                        v: w.array_scalars(geometry.tensor_v.iter().copied())?,
                    },
                    heights: geometry
                        .offset_w
                        .as_ref()
                        .map(|a| scalars_array(r, w, a))
                        .transpose()?,
                },
            ),
        }
    }
}

impl VolumeElement {
    pub fn convert<W: Write + Seek + Send, R: ReadAt>(
        &self,
        r: &Omf1Reader<R>,
        w: &mut Writer<W>,
    ) -> Result<crate::Element, Error> {
        let geometry = r.model(&self.geometry)?;
        element(
            &self.content,
            self.color,
            attributes(r, w, &self.data)?,
            crate::BlockModel {
                orient: crate::Orient3 {
                    origin: geometry.origin,
                    u: geometry.axis_u,
                    v: geometry.axis_v,
                    w: geometry.axis_w,
                },
                grid: crate::Grid3::Tensor {
                    u: w.array_scalars(geometry.tensor_u.iter().copied())?,
                    v: w.array_scalars(geometry.tensor_v.iter().copied())?,
                    w: w.array_scalars(geometry.tensor_w.iter().copied())?,
                },
                subblocks: None,
            },
        )
    }
}

impl DataModel<'_> {
    pub fn convert<W: Write + Seek + Send, R: ReadAt>(
        &self,
        r: &Omf1Reader<R>,
        w: &mut Writer<W>,
    ) -> Result<crate::Attribute, Error> {
        match *self {
            DataModel::ScalarData(x) => x.convert(r, w),
            DataModel::DateTimeData(x) => x.convert(r, w),
            DataModel::Vector2Data(x) => x.convert(r, w),
            DataModel::Vector3Data(x) => x.convert(r, w),
            DataModel::ColorData(x) => x.convert(r, w),
            DataModel::StringData(x) => x.convert(r, w),
            DataModel::MappedData(x) => x.convert(r, w),
        }
    }
}

fn element(
    content: &ContentModel,
    color: Option<[u8; 3]>,
    attributes: Vec<crate::Attribute>,
    geometry: impl Into<crate::Geometry>,
) -> Result<crate::Element, Error> {
    Ok(crate::Element {
        name: content.name.clone(),
        description: content.description.clone(),
        color: color.map(|[r, g, b]| [r, g, b, u8::MAX]),
        metadata: content.uid.metadata(),
        attributes,
        geometry: geometry.into(),
    })
}

fn attributes<W: Write + Seek + Send, R: ReadAt>(
    r: &Omf1Reader<R>,
    w: &mut Writer<W>,
    data: &[Key<Data>],
) -> Result<Vec<crate::Attribute>, Error> {
    data.iter()
        .map(|key| r.model(key)?.convert(r, w))
        .collect::<Result<Vec<_>, _>>()
}

fn attributes_and_textures<W: Write + Seek + Send, R: ReadAt>(
    r: &Omf1Reader<R>,
    w: &mut Writer<W>,
    data: &[Key<Data>],
    textures: &[Key<ImageTexture>],
) -> Result<Vec<crate::Attribute>, Error> {
    let mut data_attributes = data
        .iter()
        .map(|key| r.model(key)?.convert(r, w))
        .collect::<Result<Vec<_>, _>>()?;
    let mut texture_attributes = textures
        .iter()
        .map(|key| r.model(key)?.convert(r, w))
        .collect::<Result<Vec<_>, _>>()?;
    data_attributes.append(&mut texture_attributes);
    Ok(data_attributes)
}

fn insert_metadata(map: &mut serde_json::Map<String, serde_json::Value>, key: &str, value: &str) {
    if !value.is_empty() {
        map.insert(key.to_owned(), value.into());
    }
}
