use chrono::{DateTime, NaiveDate, Utc};

use crate::pqarray::PqArrayType;

pub trait FloatType: PqArrayType + Copy + Into<f64> + PartialOrd + Default + 'static {
    const ZERO: Self;
    const ONE: Self;
}

impl FloatType for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
}

impl FloatType for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
}

pub trait NumberType: PqArrayType + Copy + PartialOrd + Default + 'static {}

impl NumberType for f32 {}
impl NumberType for f64 {}
impl NumberType for i64 {}
impl NumberType for NaiveDate {}
impl NumberType for DateTime<Utc> {}

pub trait VectorSource<T: FloatType>: 'static {
    const IS_3D: bool;
    fn into_2d(self) -> [T; 2];
    fn into_3d(self) -> [T; 3];
}

impl<T: FloatType> VectorSource<T> for [T; 2] {
    const IS_3D: bool = false;

    fn into_2d(self) -> [T; 2] {
        [self[0], self[1]]
    }

    fn into_3d(self) -> [T; 3] {
        [self[0], self[1], T::default()]
    }
}

impl<T: FloatType> VectorSource<T> for [T; 3] {
    const IS_3D: bool = true;

    fn into_2d(self) -> [T; 2] {
        [self[0], self[1]]
    }

    fn into_3d(self) -> [T; 3] {
        [self[0], self[1], self[2]]
    }
}
