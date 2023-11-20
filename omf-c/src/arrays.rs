use crate::{
    error::Error,
    ffi_tools::{FfiConvert, FfiStorage},
};

pub enum Array {
    Image(omf::Array<omf::array_type::Image>),
    Scalar(omf::Array<omf::array_type::Scalar>),
    Vertex(omf::Array<omf::array_type::Vertex>),
    Segment(omf::Array<omf::array_type::Segment>),
    Triangle(omf::Array<omf::array_type::Triangle>),
    Name(omf::Array<omf::array_type::Name>),
    Gradient(omf::Array<omf::array_type::Gradient>),
    Texcoord(omf::Array<omf::array_type::Texcoord>),
    Boundary(omf::Array<omf::array_type::Boundary>),
    RegularSubblock(omf::Array<omf::array_type::RegularSubblock>),
    FreeformSubblock(omf::Array<omf::array_type::FreeformSubblock>),
    Number(omf::Array<omf::array_type::Number>),
    Index(omf::Array<omf::array_type::Index>),
    Vector(omf::Array<omf::array_type::Vector>),
    Text(omf::Array<omf::array_type::Text>),
    Boolean(omf::Array<omf::array_type::Boolean>),
    Color(omf::Array<omf::array_type::Color>),
}

macro_rules! define_array {
    ($( $name:ident, )*) => {
        impl Array {
            pub(crate) fn data_type(&self) -> omf::DataType {
                match self { $(
                    Self::$name(_) => omf::DataType::$name,
                )* }
            }
        }

        $(
            impl TryFrom<&Array> for omf::Array<omf::array_type::$name> {
                type Error = Error;

                fn try_from(value: &Array) -> Result<Self, Self::Error> {
                    use omf::ArrayType;
                    match value {
                        Array::$name(a) => Ok(a.clone()),
                        _ => Err(Error::ArrayTypeWrong {
                            src: "",
                            found: value.data_type(),
                            expected: omf::array_type::$name::DATA_TYPE,
                        }),
                    }
                }
            }

            impl FfiConvert<omf::Array<omf::array_type::$name>> for Array {
                fn convert(omf_array: omf::Array<omf::array_type::$name>, _st: &mut FfiStorage) -> Self {
                    Self::$name(omf_array)
                }
            }
        )*
    };
}

define_array! {
    Image,
    Scalar,
    Vertex,
    Segment,
    Triangle,
    Name,
    Gradient,
    Texcoord,
    Boundary,
    RegularSubblock,
    FreeformSubblock,
    Number,
    Index,
    Vector,
    Text,
    Boolean,
    Color,
}

macro_rules! array_action {
    ($input:expr, |$name:ident| $( $action:tt )*) => {
        match $input {
            Array::Image($name) => $($action)*,
            Array::Scalar($name) => $($action)*,
            Array::Vertex($name) => $($action)*,
            Array::Segment($name) => $($action)*,
            Array::Triangle($name) => $($action)*,
            Array::Name($name) => $($action)*,
            Array::Gradient($name) => $($action)*,
            Array::Texcoord($name) => $($action)*,
            Array::Boundary($name) => $($action)*,
            Array::RegularSubblock($name) => $($action)*,
            Array::FreeformSubblock($name) => $($action)*,
            Array::Number($name) => $($action)*,
            Array::Index($name) => $($action)*,
            Array::Vector($name) => $($action)*,
            Array::Text($name) => $($action)*,
            Array::Boolean($name) => $($action)*,
            Array::Color($name) => $($action)*,
        }
    };
}

pub(crate) fn array_from_ptr<'a, T>(
    ptr: *const Array,
    src: &'static str,
) -> Result<omf::Array<T>, Error>
where
    T: omf::ArrayType,
    omf::Array<T>: TryFrom<&'a Array, Error = Error>,
{
    let array = unsafe { crate::ffi_tools::arg::ref_from_ptr(src, ptr) }?;
    omf::Array::<T>::try_from(array)
}

pub(crate) fn array_from_ptr_opt<'a, T>(
    ptr: *const Array,
    src: &'static str,
) -> Result<Option<omf::Array<T>>, Error>
where
    T: omf::ArrayType,
    omf::Array<T>: TryFrom<&'a Array, Error = Error>,
{
    if ptr.is_null() {
        Ok(None)
    } else {
        array_from_ptr(ptr, src).map(Some)
    }
}

macro_rules! array {
    ($arg:ident) => {
        crate::arrays::array_from_ptr($arg, stringify!($arg))
    };
}

pub(crate) use {array, array_action};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(i32)]
pub enum ArrayType {
    #[default]
    Invalid = -1,
    Image,
    Scalars32,
    Scalars64,
    Vertices32,
    Vertices64,
    Segments,
    Triangles,
    Names,
    Gradient,
    Texcoords32,
    Texcoords64,
    BoundariesFloat32,
    BoundariesFloat64,
    BoundariesInt64,
    BoundariesDate,
    BoundariesDateTime,
    RegularSubblocks,
    FreeformSubblocks32,
    FreeformSubblocks64,
    NumbersFloat32,
    NumbersFloat64,
    NumbersInt64,
    NumbersDate,     // accessed as i64 days since the epoch
    NumbersDateTime, // accessed as i64 microseconds since the epoch
    Indices,
    Vectors32x2,
    Vectors64x2,
    Vectors32x3,
    Vectors64x3,
    Text,
    Booleans,
    Colors,
}
