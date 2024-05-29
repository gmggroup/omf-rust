use std::sync::Arc;

use parquet::schema::types::Type;

use crate::error::Error;

#[derive(Debug, Default, Clone)]
pub struct PqArrayMatcher<T> {
    values: Vec<T>,
    expected: Arc<Vec<Type>>,
}

impl<T: Copy> PqArrayMatcher<T> {
    /// Creates a matcher from a list of field types.
    ///
    /// Types with the same name will be grouped together.
    pub fn new<const N: usize>(items: [(T, Type); N]) -> Self {
        let (values, expected) = items.into_iter().unzip();
        Self {
            values,
            expected: Arc::new(expected),
        }
    }

    /// Returns the schemas of this matcher.
    #[cfg(test)]
    pub fn schemas(&self) -> &[Type] {
        &self.expected
    }

    /// Checks a file schema against this matcher, returning the matched index on success.
    pub fn check(&self, schema: Arc<Type>) -> Result<T, Error> {
        let index = self
            .expected
            .iter()
            .enumerate()
            .find_map(|(i, s)| if schema.as_ref() == s { Some(i) } else { None })
            .ok_or_else(|| Error::ParquetSchemaMismatch(schema, self.expected.clone()))?;
        Ok(self.values[index])
    }
}

/// Creates a schema matcher. If a field can have multiple types then repeat the name in
/// multiple fields.
macro_rules! schema_match {
    // -> PqArrayMatcher
    ($($value:expr => schema $body:tt)*) => {
        crate::pqarray::PqArrayMatcher::new(
            [
                $( ($value, crate::pqarray::schema! $body), )*
            ],
        )
    };
}

/// Creates a schema.
macro_rules! schema { // -> Type
    ($($token:tt)*) => {
        parquet::schema::types::Type::group_type_builder("schema")
            .with_fields(
                crate::pqarray::schema_fields!($($token)*)
                    .into_iter()
                    .map(std::sync::Arc::new)
                    .collect()
            )
            .build()
            .expect("valid type")
    };
}

macro_rules! schema_fields { // -> Vec<Type>
    ($( $rep:ident $phys:ident $name:ident $( ($($log:tt)*) )? ; )+) => {
        vec![
            $( crate::pqarray::schema_field!(
                $rep $phys $name
                $(( $($log)* ))?
            ) ),*
        ]
    };
    ($( $rep:ident group $name:ident $( ($($log:tt)*) )? { $($gr:tt)* } )+) => {
        vec![
            $( crate::pqarray::schema_field!(
                $rep group $name
                $(( $($log)* ))?
                { $($gr)* }
            ) ),*
        ]
    };
}

macro_rules! schema_field { // -> Type (single field)
    ($rep:ident $phys:ident $name:ident $( ($log:ident $($log_detail:tt)?) )?) => {
        parquet::schema::types::Type::primitive_type_builder(
            stringify!($name),
            crate::pqarray::schema_physical_type!($phys),
        )
        .with_repetition(crate::pqarray::schema_repetition!($rep))
        .with_logical_type(crate::pqarray::schema_logical_type!($( $log $($log_detail)? )?))
        .build()
        .expect("valid type")
    };
    ($rep:ident group $name:ident { $($token:tt)* }) => {
        parquet::schema::types::Type::group_type_builder(stringify!($name))
            .with_fields(
                crate::pqarray::schema_fields!($($token)*)
                    .into_iter()
                    .map(std::sync::Arc::new)
                    .collect()
            )
            .with_repetition(crate::pqarray::schema_repetition!($rep))
            .build()
            .expect("valid type")
    };
}

macro_rules! schema_repetition {
    (required) => {
        parquet::basic::Repetition::REQUIRED
    };
    (optional) => {
        parquet::basic::Repetition::OPTIONAL
    };
}

macro_rules! schema_physical_type {
    (int32) => {
        parquet::basic::Type::INT32
    };
    (int64) => {
        parquet::basic::Type::INT64
    };
    (float) => {
        parquet::basic::Type::FLOAT
    };
    (double) => {
        parquet::basic::Type::DOUBLE
    };
    (boolean) => {
        parquet::basic::Type::BOOLEAN
    };
    (byte_array) => {
        parquet::basic::Type::BYTE_ARRAY
    };
}

macro_rules! schema_logical_type {
    () => {
        None
    };
    (string) => {
        Some(parquet::basic::LogicalType::String)
    };
    (integer($bits:literal, $signed:literal)) => {
        Some(parquet::basic::LogicalType::Integer {
            bit_width: $bits,
            is_signed: $signed,
        })
    };
    (date) => {
        Some(parquet::basic::LogicalType::Date)
    };
    (timestamp(millis, true)) => {
        Some(parquet::basic::LogicalType::Timestamp {
            is_adjusted_to_u_t_c: true,
            unit: TimeUnit::MILLIS(Default::default()),
        })
    };
    (timestamp(micros, true)) => {
        Some(parquet::basic::LogicalType::Timestamp {
            is_adjusted_to_u_t_c: true,
            unit: parquet::basic::TimeUnit::MICROS(Default::default()),
        })
    };
    (timestamp(nanos, true)) => {
        Some(Timestamp {
            is_adjusted_to_u_t_c: true,
            unit: TimeUnit::NANOS(Default::default()),
        })
    };
}

pub(crate) use {
    schema, schema_field, schema_fields, schema_logical_type, schema_match, schema_physical_type,
    schema_repetition,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parquet_schema_match() {
        let matcher = schema_match! {
            0 => schema {
                required float x;
                optional int32 y (date);
            }
            1 => schema {
                required double x;
                optional int32 y (date);
            }
        };
        let schema = schema! {
            required double x;
            optional int32 y (date);
        };
        let index = matcher.check(schema.into()).unwrap();
        assert_eq!(index, 1);
    }

    #[test]
    fn parquet_schema_no_match() {
        let matcher = schema_match! {
            0 => schema {
                required float x;
            }
            1 => schema {
                required double x;
            }
        };
        let schema = schema! {
            required int32 x;
        };
        let err = matcher.check(schema.into()).unwrap_err();
        assert_eq!(
            err.to_string(),
            "\
            Parquet schema mismatch, found:
message schema {
  REQUIRED INT32 x;
}

Expected one of:
message schema {
  REQUIRED FLOAT x;
}
message schema {
  REQUIRED DOUBLE x;
}
"
        );
    }
}
