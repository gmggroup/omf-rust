use std::fmt::Display;

use chrono::{DateTime, NaiveDate, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    Array,
    array::Constraint,
    array_type,
    validate::{Validate, Validator},
};

/// Specifies the minimum and maximum values of a number colormap.
///
/// Values outside this range will use the color at the ends of the gradient.
/// The variant used should match the type of the number array.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum NumberRange {
    Float {
        min: f64,
        max: f64,
    },
    Integer {
        min: i64,
        max: i64,
    },
    Date {
        min: NaiveDate,
        max: NaiveDate,
    },
    DateTime {
        min: DateTime<Utc>,
        max: DateTime<Utc>,
    },
}

impl Display for NumberRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberRange::Float { min, max } => write!(f, "[{min}, {max}]"),
            NumberRange::Integer { min, max } => write!(f, "[{min}, {max}]"),
            NumberRange::Date { min, max } => write!(f, "[{min}, {max}]"),
            NumberRange::DateTime { min, max } => write!(f, "[{min}, {max}]"),
        }
    }
}

impl From<(f64, f64)> for NumberRange {
    fn from((min, max): (f64, f64)) -> Self {
        Self::Float { min, max }
    }
}

impl From<(i64, i64)> for NumberRange {
    fn from((min, max): (i64, i64)) -> Self {
        Self::Integer { min, max }
    }
}

impl From<(i32, i32)> for NumberRange {
    fn from((min, max): (i32, i32)) -> Self {
        Self::Integer {
            min: min.into(),
            max: max.into(),
        }
    }
}

impl From<(NaiveDate, NaiveDate)> for NumberRange {
    fn from((min, max): (NaiveDate, NaiveDate)) -> Self {
        Self::Date { min, max }
    }
}

impl From<(DateTime<Utc>, DateTime<Utc>)> for NumberRange {
    fn from((min, max): (DateTime<Utc>, DateTime<Utc>)) -> Self {
        Self::DateTime { min, max }
    }
}

/// Describes a mapping of floating-point value to colors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum NumberColormap {
    /// A continuous colormap linearly samples a color gradient within a defined range.
    ///
    /// A value X% of way between `min` and `max` should use the color from X% way down
    /// gradient. When that X doesn't land directly on a color use the average of
    /// the colors on either side, inverse-weighted by the distance to each.
    ///
    /// Values below the minimum use the first color in the gradient array. Values above
    /// the maximum use the last.
    ///
    /// ![Diagram of a continuous colormap](../images/colormap_continuous.svg "Continuous colormap")
    Continuous {
        /// Value range.
        range: NumberRange,
        /// Array with `Gradient` type storing the smooth color gradient.
        gradient: Array<array_type::Gradient>,
    },
    /// A discrete colormap divides the number line into adjacent but non-overlapping
    /// ranges and gives a flat color to each range.
    ///
    /// Values above the last boundary use `end_color`.
    Discrete {
        /// Array with `Boundary` type storing the smooth color gradient, containing the value
        /// and inclusiveness of each boundary. Values must increase along the array.
        /// Boundary values type should match the type of the number array.
        boundaries: Array<array_type::Boundary>,
        /// Array with `Gradient` type storing the colors of the discrete ranges.
        /// Length must be one more than `boundaries`, with the extra color used for values above
        /// the last boundary.
        gradient: Array<array_type::Gradient>,
    },
}

impl Validate for NumberColormap {
    fn validate_inner(&mut self, val: &mut Validator) {
        match self {
            NumberColormap::Continuous { range, gradient } => {
                val.enter("NumberColormap::Continuous")
                    .min_max(*range)
                    .array(gradient, Constraint::Gradient, "gradient");
            }
            NumberColormap::Discrete {
                boundaries,
                gradient,
            } => {
                val.enter("NumberColormap::Discrete")
                    .array(boundaries, Constraint::Boundary, "boundaries")
                    .array(gradient, Constraint::Gradient, "gradient")
                    .array_size(
                        gradient.item_count(),
                        boundaries.item_count() + 1,
                        "gradient",
                    );
            }
        }
    }
}
