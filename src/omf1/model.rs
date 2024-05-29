use std::fmt::Display;

use serde::Deserialize;

use super::{objects::*, Omf1Error};

/// Converts a `&Model` into either a reference to the individual item, or into
/// a subset enum.
///
/// This is used by `Omf1Root::get` to check variants on load.
pub trait FromModel {
    type Output<'a>;

    fn from_model(model: &Model) -> Result<Self::Output<'_>, Omf1Error>;
}

/// Creates enums and `FromModel` implementations for the UidModel objects in OMF v1.
macro_rules! model {
    ($( $variant:ident )*) => {
        /// Contains an OMF v1 top-level object.
        #[derive(Debug, Deserialize)]
        #[serde(tag = "__class__")]
        pub enum Model {
            $( $variant($variant), )*
        }

        /// The types of object allowed at the top level of OMF v1.
        #[derive(Debug)]
        pub enum ModelType {
            $( $variant, )*
        }

        impl Model {
            /// Return the model type.
            fn model_type(&self) -> ModelType {
                match self {
                    $( Self::$variant(_) => ModelType::$variant, )*
                }
            }
        }

        $(
            impl FromModel for $variant {
                type Output<'a> = &'a $variant;

                fn from_model(model: &Model) -> Result<Self::Output<'_>, Omf1Error> {
                    match model {
                        Model::$variant(x) => Ok(x),
                        _ => Err(Omf1Error::WrongType {
                            found: model.model_type(),
                            expected: &[ModelType::$variant],
                        }),
                    }
                }
            }
        )*
    };
}

/// Creates marker type, a subset of `Model`, and a `FromModel` implementation to tie them
/// together.
///
/// This lets us have type-tagged keys for a subset of model types in the objects that
/// `Omf1Root::get` can load and check automatically. The loading code can then match
/// exhaustively without worrying about the incorrect types.
macro_rules! model_subset {
    ($model_name:ident $enum_name:ident { $( $variant:ident )* }) => {
        #[derive(Debug)]
        pub struct $model_name {}

        #[derive(Debug, Clone, Copy)]
        #[allow(clippy::enum_variant_names)]
        pub enum $enum_name<'a> {
            $( $variant(&'a $variant), )*
        }

        impl FromModel for $model_name {
            type Output<'a> = $enum_name<'a>;

            fn from_model(model: &Model) -> Result<Self::Output<'_>, Omf1Error> {
                match model {
                    $( Model::$variant(x) => Ok($enum_name::$variant(x)), )*
                    _ => Err(Omf1Error::WrongType {
                        found: model.model_type(),
                        expected: &[$( ModelType::$variant ),*],
                    }),
                }
            }
        }
    };
}

model! {
    Project
    PointSetElement
    PointSetGeometry
    LineSetElement
    LineSetGeometry
    SurfaceElement
    SurfaceGeometry
    SurfaceGridGeometry
    VolumeElement
    VolumeGridGeometry
    ScalarColormap
    DateTimeColormap
    Legend
    ScalarData
    DateTimeData
    Vector2Data
    Vector3Data
    ColorData
    StringData
    MappedData
    ImageTexture
    ScalarArray
    Vector2Array
    Vector3Array
    Int2Array
    Int3Array
    StringArray
    DateTimeArray
    ColorArray
}

impl Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

model_subset! {
    Elements ElementModel {
        PointSetElement
        LineSetElement
        SurfaceElement
        VolumeElement
    }
}

model_subset! {
    Data DataModel {
        ScalarData
        DateTimeData
        Vector2Data
        Vector3Data
        ColorData
        StringData
        MappedData
    }
}

model_subset! {
    SurfaceGeometries SurfaceGeometryModel {
        SurfaceGeometry
        SurfaceGridGeometry
    }
}

model_subset! {
    LegendArrays LegendArrayModel {
        ColorArray
        DateTimeArray
        StringArray
        ScalarArray
    }
}

model_subset! {
    ColorArrays ColorArrayModel {
        Int3Array
        ColorArray
    }
}
