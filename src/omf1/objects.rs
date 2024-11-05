#![allow(dead_code)] // Many attributes in here exist for the JSON loading, but aren't used.
use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::model::{ColorArrays, Data, Elements, LegendArrays, SurfaceGeometries};

/// Stores a string, while `T` can control what types of model as accepted.
#[derive(Debug)]
pub struct Key<T> {
    pub value: String,
    _phantom: PhantomData<T>,
}

impl Key<Project> {
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        let mut value = format!("{:032x}", u128::from_be_bytes(bytes));
        for i in [20, 16, 12, 8] {
            value.insert(i, '-');
        }
        Self {
            value,
            _phantom: Default::default(),
        }
    }
}

struct KeyVisitor<T> {
    _phantom: PhantomData<T>,
}

impl<T> serde::de::Visitor<'_> for KeyVisitor<T> {
    type Value = Key<T>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("an id string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Key {
            value: v.to_owned(),
            _phantom: Default::default(),
        })
    }
}

impl<'de, T> Deserialize<'de> for Key<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(KeyVisitor::<T> {
            _phantom: Default::default(),
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct UidModel {
    #[serde(default)]
    pub date_created: String,
    #[serde(default)]
    pub date_modified: String,
}

#[derive(Debug, Deserialize)]
pub struct ContentModel {
    #[serde(flatten)]
    pub uid: UidModel,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    #[serde(flatten)]
    pub content: ContentModel,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub revision: String,
    #[serde(default)]
    pub date: DateTime<Utc>,
    #[serde(default)]
    pub units: String,
    #[serde(default)]
    pub origin: [f64; 3],
    #[serde(default)]
    pub elements: Vec<Key<Elements>>,
}

// Elements

#[derive(Debug, Deserialize)]
pub struct PointSetElement {
    #[serde(flatten)]
    pub content: ContentModel,
    #[serde(default)]
    pub color: Option<[u8; 3]>,
    #[serde(default)]
    pub data: Vec<Key<Data>>,
    #[serde(default)]
    pub textures: Vec<Key<ImageTexture>>,
    #[serde(default)]
    pub subtype: PointSetSubtype,
    pub geometry: Key<PointSetGeometry>,
}

#[derive(Debug, Deserialize)]
pub struct PointSetGeometry {
    #[serde(flatten)]
    pub uid: UidModel,
    #[serde(default)]
    pub origin: [f64; 3],
    pub vertices: Key<Vector3Array>,
}

#[derive(Debug, Deserialize)]
pub struct LineSetElement {
    #[serde(flatten)]
    pub content: ContentModel,
    #[serde(default)]
    pub color: Option<[u8; 3]>,
    #[serde(default)]
    pub data: Vec<Key<Data>>,
    #[serde(default)]
    pub subtype: LineSetSubtype,
    pub geometry: Key<LineSetGeometry>,
}

#[derive(Debug, Deserialize)]
pub struct LineSetGeometry {
    #[serde(flatten)]
    pub uid: UidModel,
    #[serde(default)]
    pub origin: [f64; 3],
    pub vertices: Key<Vector3Array>,
    pub segments: Key<Int2Array>,
}

#[derive(Debug, Deserialize)]
pub struct SurfaceElement {
    #[serde(flatten)]
    pub content: ContentModel,
    #[serde(default)]
    pub color: Option<[u8; 3]>,
    #[serde(default)]
    pub data: Vec<Key<Data>>,
    #[serde(default)]
    pub textures: Vec<Key<ImageTexture>>,
    #[serde(default)]
    pub subtype: SurfaceSubtype,
    pub geometry: Key<SurfaceGeometries>,
}

#[derive(Debug, Deserialize)]
pub struct SurfaceGeometry {
    #[serde(flatten)]
    pub uid: UidModel,
    #[serde(default)]
    pub origin: [f64; 3],
    pub vertices: Key<Vector3Array>,
    pub triangles: Key<Int3Array>,
}

#[derive(Debug, Deserialize)]
pub struct SurfaceGridGeometry {
    #[serde(flatten)]
    pub uid: UidModel,
    #[serde(default)]
    pub origin: [f64; 3],
    pub tensor_u: Vec<f64>,
    pub tensor_v: Vec<f64>,
    #[serde(default = "i")]
    pub axis_u: [f64; 3],
    #[serde(default = "j")]
    pub axis_v: [f64; 3],
    #[serde(default)]
    pub offset_w: Option<Key<ScalarArray>>,
}

#[derive(Debug, Deserialize)]
pub struct VolumeElement {
    #[serde(flatten)]
    pub content: ContentModel,
    #[serde(default)]
    pub color: Option<[u8; 3]>,
    #[serde(default)]
    pub data: Vec<Key<Data>>,
    #[serde(default)]
    pub subtype: VolumeSubtype,
    pub geometry: Key<VolumeGridGeometry>,
}

#[derive(Debug, Deserialize)]
pub struct VolumeGridGeometry {
    #[serde(flatten)]
    pub uid: UidModel,
    #[serde(default)]
    pub origin: [f64; 3],
    pub tensor_u: Vec<f64>,
    pub tensor_v: Vec<f64>,
    pub tensor_w: Vec<f64>,
    #[serde(default = "i")]
    pub axis_u: [f64; 3],
    #[serde(default = "j")]
    pub axis_v: [f64; 3],
    #[serde(default = "k")]
    pub axis_w: [f64; 3],
    #[serde(default)]
    pub offset_w: Option<Key<ScalarArray>>,
}

// Colormaps

#[derive(Debug, Deserialize)]
pub struct ScalarColormap {
    #[serde(flatten)]
    pub content: ContentModel,
    pub gradient: Key<ColorArray>,
    pub limits: [f64; 2],
}

#[derive(Debug, Deserialize)]
pub struct DateTimeColormap {
    #[serde(flatten)]
    pub content: ContentModel,
    pub gradient: Key<ColorArray>,
    pub limits: [DateTime<Utc>; 2],
}

#[derive(Debug, Deserialize)]
pub struct Legend {
    #[serde(flatten)]
    pub content: ContentModel,
    pub values: Key<LegendArrays>,
}

// Data

#[derive(Debug, Deserialize)]
pub struct ScalarData {
    #[serde(flatten)]
    pub content: ContentModel,
    pub location: DataLocation,
    pub colormap: Option<Key<ScalarColormap>>,
    pub array: Key<ScalarArray>,
}

#[derive(Debug, Deserialize)]
pub struct DateTimeData {
    #[serde(flatten)]
    pub content: ContentModel,
    pub location: DataLocation,
    pub colormap: Option<Key<DateTimeColormap>>,
    pub array: Key<DateTimeArray>,
}

#[derive(Debug, Deserialize)]
pub struct Vector2Data {
    #[serde(flatten)]
    pub content: ContentModel,
    pub location: DataLocation,
    pub array: Key<Vector2Array>,
}

#[derive(Debug, Deserialize)]
pub struct Vector3Data {
    #[serde(flatten)]
    pub content: ContentModel,
    pub location: DataLocation,
    pub array: Key<Vector3Array>,
}

#[derive(Debug, Deserialize)]
pub struct ColorData {
    #[serde(flatten)]
    pub content: ContentModel,
    pub location: DataLocation,
    pub array: Key<ColorArrays>,
}

#[derive(Debug, Deserialize)]
pub struct StringData {
    #[serde(flatten)]
    pub content: ContentModel,
    pub location: DataLocation,
    pub array: Key<StringArray>,
}

#[derive(Debug, Deserialize)]
pub struct MappedData {
    #[serde(flatten)]
    pub content: ContentModel,
    pub location: DataLocation,
    pub array: Key<ScalarArray>,
    #[serde(default)]
    pub legends: Vec<Key<Legend>>,
}

// Texture

#[derive(Debug, Deserialize)]
pub struct ImageTexture {
    #[serde(flatten)]
    pub content: ContentModel,
    #[serde(default)]
    pub origin: [f64; 3],
    #[serde(default = "i")]
    pub axis_u: [f64; 3],
    #[serde(default = "j")]
    pub axis_v: [f64; 3],
    pub image: Image,
}

// Arrays

#[derive(Debug, Deserialize)]
pub struct Array {
    pub start: u64,
    pub length: u64,
    pub dtype: DataType,
}

#[derive(Debug, Deserialize)]
pub struct Image {
    pub start: u64,
    pub length: u64,
    pub dtype: ImageType,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
pub enum DataType {
    #[serde(rename = "<f8")]
    Float,
    #[serde(rename = "<i8")]
    Int,
}

#[derive(Debug, Deserialize)]
pub enum ImageType {
    #[serde(rename = "image/png")]
    Png,
}

#[derive(Debug, Deserialize)]
pub struct ScalarArray {
    #[serde(flatten)]
    pub uid: UidModel,
    pub array: Array,
}

#[derive(Debug, Deserialize)]
pub struct Vector2Array {
    #[serde(flatten)]
    pub uid: UidModel,
    pub array: Array,
}

#[derive(Debug, Deserialize)]
pub struct Vector3Array {
    #[serde(flatten)]
    pub uid: UidModel,
    pub array: Array,
}

#[derive(Debug, Deserialize)]
pub struct Int2Array {
    #[serde(flatten)]
    pub uid: UidModel,
    pub array: Array,
}

#[derive(Debug, Deserialize)]
pub struct Int3Array {
    #[serde(flatten)]
    pub uid: UidModel,
    pub array: Array,
}

#[derive(Debug, Deserialize)]
pub struct StringArray {
    #[serde(flatten)]
    pub uid: UidModel,
    pub array: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DateTimeArray {
    #[serde(flatten)]
    pub uid: UidModel,
    pub array: Vec<Option<DateTime<Utc>>>,
}

#[derive(Debug, Deserialize)]
pub struct ColorArray {
    #[serde(flatten)]
    pub uid: UidModel,
    pub array: Vec<[u8; 3]>,
}

// Enums

#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq)]
pub enum PointSetSubtype {
    #[serde(rename = "point")]
    #[default]
    Point,
    #[serde(rename = "collar")]
    Collar,
    #[serde(rename = "blasthole")]
    BlastHole,
}

#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq)]
pub enum LineSetSubtype {
    #[serde(rename = "line")]
    #[default]
    Line,
    #[serde(rename = "borehole")]
    BoreHole,
}

#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq)]
pub enum SurfaceSubtype {
    #[serde(rename = "surface")]
    #[default]
    Surface,
}

#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq)]
pub enum VolumeSubtype {
    #[serde(rename = "volume")]
    #[default]
    Volume,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
pub enum DataLocation {
    #[serde(rename = "vertices")]
    Vertices,
    #[serde(rename = "segments")]
    Segments,
    #[serde(rename = "faces")]
    Faces,
    #[serde(rename = "cells")]
    Cells,
}

// Default factories

fn i() -> [f64; 3] {
    [1.0, 0.0, 0.0]
}

fn j() -> [f64; 3] {
    [0.0, 1.0, 0.0]
}

fn k() -> [f64; 3] {
    [0.0, 0.0, 1.0]
}
