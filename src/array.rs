use std::marker::PhantomData;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "parquet")]
use crate::data::write_checks::ArrayWriteCheck;
use crate::{validate::Reason, SubblockMode};

pub trait ArrayType {
    const DATA_TYPE: DataType;
}

macro_rules! array_types {
    ($(#[doc = $doc:literal] $name:ident,)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
        pub enum DataType {
            $($name,)*
        }

        pub mod array_type {
            use super::*;
            $(
                #[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
                pub struct $name {}
                impl ArrayType for $name {
                    const DATA_TYPE: DataType = DataType::$name;
                }
            )*
        }
    };
}

array_types! {
    /// An image in PNG or JPEG encoding.
    Image,
    /// Floating-point scalar values.
    Scalar,
    /// Vertex locations in 3D. Add the project and element origins.
    Vertex,
    /// Line segments as indices into a vertex array.
    Segment,
    /// Triangles as indices into a vertex array.
    Triangle,
    /// Non-nullable category names.
    Name,
    /// Non-nullable colormap or category colors.
    Gradient,
    /// UV texture coordinates.
    Texcoord,
    /// Discrete color-map boundaries.
    Boundary,
    /// Parent indices and corners of regular sub-blocks.
    RegularSubblock,
    /// Parent indices and corners of free-form sub-blocks.
    FreeformSubblock,
    /// Nullable number values, floating-point or signed integer.
    Number,
    /// Nullable category index values.
    Index,
    /// Nullable 2D or 3D vectors.
    Vector,
    /// Nullable text.
    Text,
    /// Nullable booleans.
    Boolean,
    /// Nullable colors.
    Color,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Array<T> {
    filename: String,
    item_count: u64,
    #[serde(default, skip_serializing, skip_deserializing)]
    private: ArrayPrivate,
    #[serde(default, skip_serializing, skip_deserializing)]
    _marker: PhantomData<T>,
}

impl<T: ArrayType> Array<T> {
    pub(crate) fn new(filename: String, item_count: u64) -> Self {
        Self {
            filename,
            item_count,
            private: Default::default(),
            _marker: Default::default(),
        }
    }

    pub(crate) fn constrain(&mut self, constraint: Constraint) -> Result<(), Reason> {
        assert_eq!(
            constraint.data_type(),
            self.data_type(),
            "invalid constraint {constraint:?} for {:?} array",
            self.data_type()
        );
        self.private.constraint = Some(constraint);
        Ok(())
    }

    pub(crate) fn constraint(&self) -> &Constraint {
        self.private
            .constraint
            .as_ref()
            .expect("array should have been validated")
    }

    /// The filename of the array data within the zip file.
    pub(crate) fn filename(&self) -> &str {
        &self.filename
    }

    /// Number of items in the decompressed array. Zero for images.
    pub fn item_count(&self) -> u64 {
        self.item_count
    }

    /// The type of the array, based on `T`.
    pub fn data_type(&self) -> DataType {
        T::DATA_TYPE
    }
}

#[cfg(feature = "parquet")]
impl<T: ArrayType> Array<T> {
    pub(crate) fn add_write_checks(mut self, checks: Vec<ArrayWriteCheck>) -> Self {
        self.private.checks.extend(checks);
        self
    }

    pub(crate) fn run_write_checks(&self) -> Vec<Reason> {
        let mut reasons = Vec::new();
        for check in &self.private.checks {
            if let Err(r) = check.check(self) {
                reasons.push(r);
            }
        }
        reasons
    }
}

#[cfg(not(feature = "parquet"))]
impl<T: ArrayType> Array<T> {
    pub(crate) fn run_write_checks(&self) -> Vec<Reason> {
        Vec::new()
    }
}

#[derive(Debug, Default, Clone)]
struct ArrayPrivate {
    constraint: Option<Constraint>,
    #[cfg(feature = "parquet")]
    checks: Vec<ArrayWriteCheck>,
}

impl PartialEq for ArrayPrivate {
    fn eq(&self, _other: &Self) -> bool {
        // Don't let this private data interfere with tests.
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Constraint {
    Image,
    Scalar,
    Size,
    Vertex,
    Segment(u64),
    Triangle(u64),
    Name,
    Gradient,
    Texcoord,
    Boundary,
    RegularSubblock {
        block_count: [u32; 3],
        subblock_count: [u32; 3],
        mode: Option<SubblockMode>,
    },
    FreeformSubblock {
        block_count: [u32; 3],
    },
    Number,
    Index(u64),
    Vector,
    String,
    Boolean,
    Color,
}

impl Constraint {
    pub fn data_type(&self) -> DataType {
        match self {
            Self::Image => DataType::Image,
            Self::Scalar | Self::Size => DataType::Scalar,
            Self::Vertex => DataType::Vertex,
            Self::Segment(_) => DataType::Segment,
            Self::Triangle(_) => DataType::Triangle,
            Self::Name => DataType::Name,
            Self::Gradient => DataType::Gradient,
            Self::Texcoord => DataType::Texcoord,
            Self::Boundary => DataType::Boundary,
            Self::RegularSubblock { .. } => DataType::RegularSubblock,
            Self::FreeformSubblock { .. } => DataType::FreeformSubblock,
            Self::Number => DataType::Number,
            Self::Index(_) => DataType::Index,
            Self::Vector => DataType::Vector,
            Self::String => DataType::Text,
            Self::Boolean => DataType::Boolean,
            Self::Color => DataType::Color,
        }
    }
}
