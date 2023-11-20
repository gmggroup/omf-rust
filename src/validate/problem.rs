use std::{fmt::Debug, fmt::Display};

use crate::{colormap::NumberRange, error::InvalidData, Location};

/// Validation failure reason.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Reason {
    /// A floating-point number is NaN, Inf, or -Inf.
    #[error("must be finite")]
    NotFinite,
    /// A size is zero or less.
    #[error("must be greater than zero")]
    NotGreaterThanZero,
    /// Vector must have length one.
    #[error("must be a unit vector but {0:?} length is {1}")]
    NotUnitVector([f64; 3], f64),
    /// Vectors must be at right angles.
    #[error("vectors are not orthogonal: {0:?} {1:?}")]
    NotOrthogonal([f64; 3], [f64; 3]),
    /// A sub-blocked model says it uses octree mode but the sub-block counts are not
    /// powers of two.
    #[error("sub-block counts {0:?} must be powers of two for octree mode")]
    OctreeNotPowerOfTwo([u32; 3]),
    /// A grid or block model has size greater than 2³² in any direction.
    #[error("grid count {0:?} exceeds maximum of 4,294,967,295")]
    GridTooLarge(Vec<u64>),
    /// Attribute using a location that doesn't exist on the containing geometry.
    #[error("is {0:?} which is not valid on {1} geometry")]
    AttrLocationWrongForGeom(Location, &'static str),
    /// Attribute using a location that is impossible for the attribute data.
    #[error("is {0:?} which is not valid on {1} attributes")]
    AttrLocationWrongForAttr(Location, &'static str),
    /// Attribute length doesn't match the geometry and location.
    #[error("length {0} does not match geometry ({1})")]
    AttrLengthMismatch(u64, u64),
    /// Minimum is greater than maximum.
    #[error("minimum is greater than maximum in {0}")]
    MinMaxOutOfOrder(NumberRange),
    /// The data inside an array is invalid.
    #[error("array contains invalid data: {0}")]
    InvalidData(InvalidData),
    /// A data file or index is missing from the zip.
    #[error("refers to non-existent archive member '{0}'")]
    ZipMemberMissing(String),
    /// A field that must be unique is duplicated.
    #[error("must be unique but {0} is repeated")]
    NotUnique(String),
    /// A field that should be unique is duplicated.
    #[error("contains duplicate of {0}")]
    SoftNotUnique(String),
    /// Ran into the validation message limit.
    #[error("{0} more errors")]
    MoreErrors(u32),
    /// Ran into the validation message limit.
    #[error("{0} more warnings")]
    MoreWarnings(u32),
}

impl Reason {
    /// True if the reason is an error, false if it is a warning.
    pub fn is_error(&self) -> bool {
        !matches!(self, Self::SoftNotUnique(_) | Reason::MoreWarnings(_))
    }
}

/// A single validation problem.
#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    /// Reason for the problem.
    pub reason: Reason,
    /// Type name of the failed object.
    pub ty: &'static str,
    /// Optional field name where the failure is.
    pub field: Option<&'static str>,
    /// Optional name of the containing object.
    pub name: Option<String>,
}

impl Problem {
    /// True if the reason is an error, false if it is a warning.
    pub fn is_error(&self) -> bool {
        self.reason.is_error()
    }
}

impl Display for Problem {
    /// Formats a validation problem.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let severity = if self.reason.is_error() {
            "Error"
        } else {
            "Warning"
        };
        write!(f, "{severity}: '{}", self.ty)?;
        if let Some(field) = self.field {
            write!(f, "::{field}'")?;
        } else {
            write!(f, "'")?;
        }
        write!(f, " {}", self.reason)?;
        if let Some(name) = &self.name {
            write!(f, ", inside '{name}'")?;
        }
        Ok(())
    }
}

/// A container of validation problems.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Problems(Vec<Problem>);

impl Problems {
    /// True if there are no problems.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The number of problems.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Converts to a vec without copying.
    pub fn into_vec(self) -> Vec<Problem> {
        self.0
    }

    /// Iterates over the problems.
    pub fn iter(&self) -> impl Iterator<Item = &Problem> {
        self.0.iter()
    }

    /// Ok if there are only warnings, Err if there are errors.
    pub(crate) fn into_result(self) -> Result<Self, Self> {
        if self.0.iter().any(|p| p.is_error()) {
            Err(self)
        } else {
            Ok(self)
        }
    }

    pub(crate) fn push(
        &mut self,
        reason: Reason,
        ty: &'static str,
        field: Option<&'static str>,
        name: Option<String>,
    ) {
        self.0.push(Problem {
            reason,
            ty,
            field,
            name,
        })
    }
}

impl Display for Problems {
    /// Formats a list of validation problems.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n_errors = self.0.iter().filter(|p| p.reason.is_error()).count();
        let n_warnings = self.0.len() - n_errors;
        match (n_errors, n_warnings) {
            (0, 0) => write!(f, "OMF validation passed")?,
            (0, _) => write!(f, "OMF validation passed with warnings:")?,
            _ => write!(f, "OMF validation failed:")?,
        }
        for problem in self {
            write!(f, "\n  {problem}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Problems {}

impl IntoIterator for Problems {
    type Item = Problem;
    type IntoIter = std::vec::IntoIter<Problem>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Problems {
    type Item = &'a Problem;
    type IntoIter = std::slice::Iter<'a, Problem>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl From<Problems> for Vec<Problem> {
    fn from(value: Problems) -> Self {
        value.into_vec()
    }
}
