use chrono::prelude::*;
use parquet::{
    basic::{Encoding, LogicalType, Type as PhysicalType},
    data_type::{AsBytes, ByteArray, ByteArrayType, DataType, Int32Type, Int64Type},
    format::{MicroSeconds, TimeUnit},
};

use crate::date_time::*;

pub trait PqArrayType: Default + 'static {
    type DataType: DataType;

    fn physical_type() -> PhysicalType {
        <Self::DataType as DataType>::get_physical_type()
    }

    fn logical_type() -> Option<LogicalType> {
        None
    }

    fn check_logical_type(file_type: Option<LogicalType>) -> bool {
        file_type == Self::logical_type()
    }

    fn encoding() -> Option<Encoding> {
        None
    }

    fn from_parquet(
        value: <Self::DataType as DataType>::T,
        _logical_type: &Option<LogicalType>,
    ) -> Self;

    fn to_parquet(self) -> <Self::DataType as DataType>::T;
}

macro_rules! simple {
    ($t:ty, $dt:ident) => {
        impl PqArrayType for $t {
            type DataType = parquet::data_type::$dt;

            fn to_parquet(self) -> <Self::DataType as DataType>::T {
                self
            }

            fn from_parquet(
                value: <Self::DataType as DataType>::T,
                _logical_type: &Option<LogicalType>,
            ) -> Self {
                value
            }
        }
    };
}

macro_rules! physical_integer {
    ($t:ty, $dt:ident, $bits:literal, $signed:literal) => {
        impl PqArrayType for $t {
            type DataType = parquet::data_type::$dt;

            fn to_parquet(self) -> <Self::DataType as DataType>::T {
                self as <Self::DataType as DataType>::T
            }

            fn from_parquet(
                value: <Self::DataType as DataType>::T,
                _logical_type: &Option<LogicalType>,
            ) -> Self {
                value as Self
            }
        }
    };
}

macro_rules! logical_integer {
    ($t:ty, $dt:ident, $bits:literal, $signed:literal) => {
        impl PqArrayType for $t {
            type DataType = parquet::data_type::$dt;

            fn logical_type() -> Option<LogicalType> {
                Some(LogicalType::Integer {
                    bit_width: $bits,
                    is_signed: $signed,
                })
            }

            fn to_parquet(self) -> <Self::DataType as DataType>::T {
                self as <Self::DataType as DataType>::T
            }

            fn from_parquet(
                value: <Self::DataType as DataType>::T,
                _logical_type: &Option<LogicalType>,
            ) -> Self {
                value as Self
            }
        }
    };
}

simple!(f64, DoubleType);
simple!(f32, FloatType);
simple!(bool, BoolType);
logical_integer!(u8, Int32Type, 8, false);
logical_integer!(u16, Int32Type, 16, false);
logical_integer!(u32, Int32Type, 32, false);
logical_integer!(u64, Int64Type, 64, false);
logical_integer!(i8, Int32Type, 8, true);
logical_integer!(i16, Int32Type, 16, true);
physical_integer!(i32, Int32Type, 32, true);
physical_integer!(i64, Int64Type, 64, true);

impl PqArrayType for String {
    type DataType = ByteArrayType;

    fn logical_type() -> Option<LogicalType> {
        Some(LogicalType::String)
    }

    fn to_parquet(self) -> ByteArray {
        self.into_bytes().into()
    }

    fn from_parquet(value: ByteArray, _logical_type: &Option<LogicalType>) -> Self {
        String::from_utf8_lossy(value.as_bytes()).into_owned()
    }
}

impl PqArrayType for Vec<u8> {
    type DataType = ByteArrayType;

    fn to_parquet(self) -> ByteArray {
        self.into()
    }

    fn from_parquet(value: ByteArray, _logical_type: &Option<LogicalType>) -> Self {
        value.as_bytes().into()
    }
}

impl PqArrayType for NaiveDate {
    type DataType = Int32Type;

    fn logical_type() -> Option<LogicalType> {
        Some(LogicalType::Date)
    }

    fn to_parquet(self) -> i32 {
        date_to_i64(self).clamp(i32::MIN as i64, i32::MAX as i64) as i32
    }

    fn from_parquet(value: i32, _logical_type: &Option<LogicalType>) -> Self {
        i64_to_date(value.into())
    }
}

impl PqArrayType for DateTime<Utc> {
    type DataType = Int64Type;

    fn logical_type() -> Option<LogicalType> {
        Some(LogicalType::Timestamp {
            is_adjusted_to_u_t_c: true,
            unit: TimeUnit::MICROS(MicroSeconds::new()),
        })
    }

    fn check_logical_type(file_type: Option<LogicalType>) -> bool {
        matches!(file_type, Some(LogicalType::Timestamp { .. }))
    }

    fn to_parquet(self) -> i64 {
        self.timestamp_micros()
    }

    fn from_parquet(
        value: <Self::DataType as DataType>::T,
        logical_type: &Option<LogicalType>,
    ) -> Self {
        match logical_type {
            Some(LogicalType::Timestamp { unit, .. }) => match unit {
                TimeUnit::MILLIS(_) => i64_milli_to_date_time(value),
                TimeUnit::MICROS(_) => i64_to_date_time(value),
                TimeUnit::NANOS(_) => i64_nano_to_date_time(value),
            },
            _ => unreachable!("the logical type was checked earlier"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use parquet::format::{MilliSeconds, NanoSeconds};

    use super::*;

    const DATE_TIME_MILLI: Option<LogicalType> = Some(LogicalType::Timestamp {
        is_adjusted_to_u_t_c: true,
        unit: TimeUnit::MILLIS(MilliSeconds {}),
    });
    const DATE_TIME_MICRO: Option<LogicalType> = Some(LogicalType::Timestamp {
        is_adjusted_to_u_t_c: true,
        unit: TimeUnit::MICROS(MicroSeconds {}),
    });
    const DATE_TIME_NANO: Option<LogicalType> = Some(LogicalType::Timestamp {
        is_adjusted_to_u_t_c: true,
        unit: TimeUnit::NANOS(NanoSeconds {}),
    });

    #[test]
    fn source_data_identical() {
        let n: f64 = 13.5_f64.to_parquet();
        assert_eq!(n, 13.5_f64);
    }

    #[test]
    fn source_data_unsigned() {
        let i: i32 = 4_294_967_295_u32.to_parquet();
        let j = u32::from_ne_bytes(i.to_ne_bytes());
        assert_eq!(j, 4_294_967_295_u32);
    }

    #[test]
    fn source_data_smaller_unsigned() {
        let i: i32 = 50000_u16.to_parquet();
        assert_eq!(i, 50000);
    }

    #[test]
    fn source_data_smaller() {
        let i: i32 = 16000_i16.to_parquet();
        assert_eq!(i, 16000);
    }

    #[test]
    fn date_conversion() {
        let zero = NaiveDate::from_ymd_opt(1970, 01, 01).unwrap();
        let one = NaiveDate::from_ymd_opt(1970, 01, 02).unwrap();
        let minus_one = NaiveDate::from_ymd_opt(1969, 12, 31).unwrap();
        assert_eq!(zero.to_parquet(), 0);
        assert_eq!(one.to_parquet(), 1);
        assert_eq!(minus_one.to_parquet(), -1);
        assert_eq!(NaiveDate::from_parquet(0, &None), zero);
        assert_eq!(NaiveDate::from_parquet(1, &None), one);
        assert_eq!(NaiveDate::from_parquet(-1, &None), minus_one);
    }

    #[test]
    fn date_overflow() {
        let min = NaiveDate::from_parquet(i32::MIN, &None);
        assert_eq!(min.to_string(), "-262144-01-01");
        let max = NaiveDate::from_parquet(i32::MAX, &None);
        assert_eq!(max.to_string(), "+262143-12-31");
    }

    #[test]
    fn date_time_conversion() {
        let zero_ms = DateTime::<Utc>::from_parquet(0, &DATE_TIME_MILLI);
        let zero_us = DateTime::<Utc>::from_parquet(0, &DATE_TIME_MICRO);
        let zero_ns = DateTime::<Utc>::from_parquet(0, &DATE_TIME_NANO);
        let e = DateTime::<Utc>::default();
        assert_eq!(e.to_parquet(), 0);
        assert_eq!(zero_ms, e);
        assert_eq!(zero_us, e);
        assert_eq!(zero_ns, e);
        let noon_ms = DateTime::<Utc>::from_parquet(43_200_000, &DATE_TIME_MILLI);
        let noon_us = DateTime::<Utc>::from_parquet(43_200_000_000, &DATE_TIME_MICRO);
        let noon_ns = DateTime::<Utc>::from_parquet(43_200_000_000_000, &DATE_TIME_NANO);
        let n = DateTime::<Utc>::from_str("1970-01-01T12:00:00Z").unwrap();
        assert_eq!(n.to_parquet(), 43_200_000_000);
        assert_eq!(noon_ms, n);
        assert_eq!(noon_us, n);
        assert_eq!(noon_ns, n);
        let bc0 = DateTime::<Utc>::from_str("-1000-08-20 03:00:00.123456 UTC").unwrap();
        assert_eq!(bc0.to_parquet(), -93704158799876544);
        let bc1 = DateTime::<Utc>::from_parquet(bc0.to_parquet(), &DATE_TIME_MICRO);
        assert_eq!(bc0, bc1);
    }

    #[test]
    fn date_time_overflow() {
        let min = DateTime::<Utc>::from_parquet(i64::MIN, &DATE_TIME_MILLI);
        assert_eq!(min.to_string(), "-262144-01-01 00:00:00 UTC");
        let max = DateTime::<Utc>::from_parquet(i64::MAX, &DATE_TIME_MILLI);
        assert_eq!(max.to_string(), "+262143-12-31 23:59:59.999999999 UTC");
    }
}
