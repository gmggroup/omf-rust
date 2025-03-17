use std::fmt::Display;

use chrono::{DateTime, NaiveDate, Utc};

use crate::{
    date_time::{date_time_to_f64, date_time_to_i64, date_to_f64, date_to_i64},
    error::Error,
    file::{ReadAt, SubFile},
    pqarray::read::SimpleIter,
};

use super::{
    BoundaryValues, GenericArrays, GenericFreeformSubblocks, GenericNumbers, GenericOptionalArrays,
    GenericScalars, NumberType,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Boundary<T: NumberType> {
    Less(T),
    LessEqual(T),
}

impl<T: NumberType> Boundary<T> {
    pub fn from<U: NumberType + Into<T>>(other: Boundary<U>) -> Self {
        match other {
            Boundary::Less(value) => Self::Less(value.into()),
            Boundary::LessEqual(value) => Self::LessEqual(value.into()),
        }
    }

    pub fn from_value(value: T, is_inclusive: bool) -> Self {
        if is_inclusive {
            Self::LessEqual(value)
        } else {
            Self::Less(value)
        }
    }

    pub fn value(self) -> T {
        match self {
            Boundary::Less(value) | Boundary::LessEqual(value) => value,
        }
    }

    pub fn is_inclusive(self) -> bool {
        match self {
            Boundary::Less(_) => false,
            Boundary::LessEqual(_) => true,
        }
    }

    pub fn map<U: NumberType>(self, func: impl FnOnce(T) -> U) -> Boundary<U> {
        match self {
            Boundary::Less(t) => Boundary::Less(func(t)),
            Boundary::LessEqual(t) => Boundary::LessEqual(func(t)),
        }
    }
}

impl<T: NumberType + Display> Display for Boundary<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Less(value) => write!(f, "< {value}"),
            Self::LessEqual(value) => write!(f, "≤ {value}"),
        }
    }
}

/// Iterator for reading scalar data.
///
/// Casts to `f64` by default or you can access the variants directly.
#[derive(Debug)]
pub enum Scalars<R: ReadAt> {
    F32(GenericScalars<f32, R>),
    F64(GenericScalars<f64, R>),
}

impl<R: ReadAt> Iterator for Scalars<R> {
    type Item = Result<f64, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::F64(iter) => iter.next(),
            Self::F32(iter) => iter.next().map(|r| r.map(Into::into)),
        }
    }
}

/// Iterator for reading vertex data of various types.
///
/// Can be used as an iterator that casts to `[f64; 3]` or you can access the variants directly.
#[derive(Debug)]
pub enum Vertices<R: ReadAt> {
    F32(GenericArrays<f32, 3, R>),
    F64(GenericArrays<f64, 3, R>),
}

impl<R: ReadAt> Iterator for Vertices<R> {
    type Item = Result<[f64; 3], Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::F64(iter) => iter.next(),
            Self::F32(iter) => array_item_cast(iter.next()),
        }
    }
}

fn array_item_cast<T, U: From<T>, const N: usize>(
    input: Option<Result<[T; N], Error>>,
) -> Option<Result<[U; N], Error>> {
    input.map(|r| r.map(|a| a.map(Into::into)))
}

/// Iterator for reading texture coordinate data of various types.
///
/// Can be used as an iterator that casts to `[f64; 2]` or you can access the variants directly.
#[derive(Debug)]
pub enum Texcoords<R: ReadAt> {
    F32(GenericArrays<f32, 2, R>),
    F64(GenericArrays<f64, 2, R>),
}

impl<R: ReadAt> Iterator for Texcoords<R> {
    type Item = Result<[f64; 2], Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::F64(iter) => iter.next(),
            Self::F32(iter) => array_item_cast(iter.next()),
        }
    }
}

/// Iterator for reading number data of various types.
///
/// You can access the variants directly or use the `try_into_f64` and `try_into_i64` methods.
/// These methods can both fail so aren't automatic.
#[derive(Debug)]
pub enum Numbers<R: ReadAt> {
    F32(GenericNumbers<f32, R>),
    F64(GenericNumbers<f64, R>),
    I64(GenericNumbers<i64, R>),
    Date(GenericNumbers<NaiveDate, R>),
    DateTime(GenericNumbers<DateTime<Utc>, R>),
}

impl<R: ReadAt> Numbers<R> {
    /// Turns this into an `f64` iterator, casting values.
    ///
    /// If the numbers use type `i64` this will fail with `Error::UnsafeCast`. Dates will become
    /// days since the '1970-01-01' epoch. Date-times will become seconds since the
    /// '1970-01-01T00:00:00Z' epoch with a small loss of precision.
    ///
    /// Currently can't fail but future number types might yield `Error::UnsafeCast`.
    pub fn try_into_f64(self) -> Result<NumbersF64<R>, Error> {
        match &self {
            Numbers::I64(_) => Err(Error::UnsafeCast("64-bit integer", "64-bit float")),
            Numbers::F32(_) | Numbers::F64(_) | Numbers::Date(_) | Numbers::DateTime(_) => {
                Ok(NumbersF64(self))
            }
        }
    }

    /// Turns this into an `i64` iterator, casting values.
    ///
    /// Floating-point types will be rejected with `Error::UnsafeCast`. Dates will become
    /// days since the '1970-01-01' epoch. Date-times will become microseconds since the
    /// '1970-01-01T00:00:00Z' epoch.
    pub fn try_into_i64(self) -> Result<NumbersI64<R>, Error> {
        match self {
            Numbers::F32(_) => Err(Error::UnsafeCast("32-bit float", "64-bit integer")),
            Numbers::F64(_) => Err(Error::UnsafeCast("64-bit float", "64-bit integer")),
            Numbers::I64(_) | Numbers::Date(_) | Numbers::DateTime(_) => Ok(NumbersI64(self)),
        }
    }
}

pub struct NumbersF64<R: ReadAt>(Numbers<R>);

impl<R: ReadAt> Iterator for NumbersF64<R> {
    type Item = Result<Option<f64>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            Numbers::F32(i) => i.next().map(|r| r.map(|o| o.map(Into::into))),
            Numbers::F64(i) => i.next(),
            Numbers::Date(i) => i.next().map(|r| r.map(|o| o.map(date_to_f64))),
            Numbers::DateTime(i) => i.next().map(|r| r.map(|o| o.map(date_time_to_f64))),
            Numbers::I64(_) => None,
        }
    }
}

pub struct NumbersI64<R: ReadAt>(Numbers<R>);

impl<R: ReadAt> Iterator for NumbersI64<R> {
    type Item = Result<Option<i64>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            Numbers::F32(_) | Numbers::F64(_) => None,
            Numbers::I64(i) => i.next(),
            Numbers::Date(i) => i.next().map(|r| r.map(|o| o.map(date_to_i64))),
            Numbers::DateTime(i) => i.next().map(|r| r.map(|o| o.map(date_time_to_i64))),
        }
    }
}

/// Iterator for reading vector data.
///
/// Casts to `Option<[f64; 3]>` by default or you can access the variants directly.
/// 2D vectors are cast to a 3D vector with zero in the Z component.
#[derive(Debug)]
pub enum Vectors<R: ReadAt> {
    F32x2(GenericOptionalArrays<f32, 2, R>),
    F64x2(GenericOptionalArrays<f64, 2, R>),
    F32x3(GenericOptionalArrays<f32, 3, R>),
    F64x3(GenericOptionalArrays<f64, 3, R>),
}

impl<R: ReadAt> Iterator for Vectors<R> {
    type Item = Result<Option<[f64; 3]>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::F32x2(iter) => iter
                .next()
                .map(|r| r.map(|o| o.map(|[x, y]| [x.into(), y.into(), 0.0]))),
            Self::F64x2(iter) => iter.next().map(|r| r.map(|o| o.map(|[x, y]| [x, y, 0.0]))),
            Self::F32x3(iter) => {
                let input = iter.next();
                input.map(|r| r.map(|o| o.map(|a| a.map(Into::into))))
            }
            Self::F64x3(iter) => iter.next(),
        }
    }
}

/// Generic iterator for boundary data.
pub struct GenericBoundaries<T: NumberType, R: ReadAt> {
    value: BoundaryValues<T, R>,
    inclusive: SimpleIter<bool, SubFile<R>>,
}

impl<T: NumberType + std::fmt::Debug, R: ReadAt + std::fmt::Debug> std::fmt::Debug
    for GenericBoundaries<T, R>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericBoundaries")
            .field("value", &self.value)
            .field("inclusive", &"...")
            .finish()
    }
}

impl<T: NumberType, R: ReadAt> GenericBoundaries<T, R> {
    pub fn new(value: SimpleIter<T, SubFile<R>>, inclusive: SimpleIter<bool, SubFile<R>>) -> Self {
        Self {
            value: BoundaryValues::new(value),
            inclusive,
        }
    }
}

impl<T: NumberType, R: ReadAt> Iterator for GenericBoundaries<T, R> {
    type Item = Result<Boundary<T>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.value.next(), self.inclusive.next()) {
            (Some(Err(e)), _) | (_, Some(Err(e))) => Some(Err(e)),
            (None, _) | (_, None) => None,
            (Some(Ok(value)), Some(Ok(false))) => Some(Ok(Boundary::Less(value))),
            (Some(Ok(value)), Some(Ok(true))) => Some(Ok(Boundary::LessEqual(value))),
        }
    }
}

/// Iterator for reading color data.
///
/// Casting is the same as [`Numbers`](Numbers).
#[derive(Debug)]
pub enum Boundaries<R: ReadAt> {
    F32(GenericBoundaries<f32, R>),
    F64(GenericBoundaries<f64, R>),
    I64(GenericBoundaries<i64, R>),
    Date(GenericBoundaries<NaiveDate, R>),
    DateTime(GenericBoundaries<DateTime<Utc>, R>),
}

impl<R: ReadAt> Boundaries<R> {
    /// Turns this into an `f64` boundary iterator, casting values.
    ///
    /// If the numbers use type `i64` this will fail with `Error::UnsafeCast`. Dates will become
    /// days since the '1970-01-01' epoch. Date-times will become seconds since the
    /// '1970-01-01T00:00:00Z' epoch with a small loss of precision.
    ///
    /// Currently can't fail but future number types might yield `Error::UnsafeCast`.
    pub fn try_into_f64(self) -> Result<BoundariesF64<R>, Error> {
        match &self {
            Boundaries::I64(_) => Err(Error::UnsafeCast("64-bit integer", "64-bit float")),
            Boundaries::F32(_)
            | Boundaries::F64(_)
            | Boundaries::Date(_)
            | Boundaries::DateTime(_) => Ok(BoundariesF64(self)),
        }
    }

    /// Turns this into an `i64` boundary iterator, casting values.
    ///
    /// Floating-point types will be rejected with `Error::UnsafeCast`. Dates will become
    /// days since the '1970-01-01' epoch. Date-times will become microseconds since the
    /// '1970-01-01T00:00:00Z' epoch.
    pub fn try_into_i64(self) -> Result<BoundariesI64<R>, Error> {
        match self {
            Boundaries::F32(_) => Err(Error::UnsafeCast("32-bit float", "64-bit integer")),
            Boundaries::F64(_) => Err(Error::UnsafeCast("64-bit float", "64-bit integer")),
            Boundaries::I64(_) | Boundaries::Date(_) | Boundaries::DateTime(_) => {
                Ok(BoundariesI64(self))
            }
        }
    }
}

pub struct BoundariesF64<R: ReadAt>(Boundaries<R>);

impl<R: ReadAt> Iterator for BoundariesF64<R> {
    type Item = Result<Boundary<f64>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            Boundaries::F32(i) => i.next().map(|r| r.map(|o| o.map(Into::into))),
            Boundaries::F64(i) => i.next(),
            Boundaries::Date(i) => i.next().map(|r| r.map(|o| o.map(date_to_f64))),
            Boundaries::DateTime(i) => i.next().map(|r| r.map(|o| o.map(date_time_to_f64))),
            Boundaries::I64(_) => None,
        }
    }
}

pub struct BoundariesI64<R: ReadAt>(Boundaries<R>);

impl<R: ReadAt> Iterator for BoundariesI64<R> {
    type Item = Result<Boundary<i64>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            Boundaries::F32(_) | Boundaries::F64(_) => None,
            Boundaries::I64(i) => i.next(),
            Boundaries::Date(i) => i.next().map(|r| r.map(|o| o.map(date_to_i64))),
            Boundaries::DateTime(i) => i.next().map(|r| r.map(|o| o.map(date_time_to_i64))),
        }
    }
}

/// Iterator for reading regular sub-block corner min/max data.
///
/// Casts to `[f64; 6]` by default or you can access the variants directly.
/// Each item is `[min_x, min_y, min_z, max_x, max_y, max_z]`.
#[derive(Debug)]
pub enum FreeformSubblocks<R: ReadAt> {
    F32(GenericFreeformSubblocks<f32, R>),
    F64(GenericFreeformSubblocks<f64, R>),
}

impl<R: ReadAt> Iterator for FreeformSubblocks<R> {
    type Item = Result<([u32; 3], [f64; 6]), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::F64(iter) => iter.next(),
            Self::F32(iter) => match iter.next() {
                Some(Ok((parent, corners))) => Some(Ok((parent, corners.map(Into::into)))),
                Some(Err(e)) => Some(Err(e)),
                None => None,
            },
        }
    }
}
