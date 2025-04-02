use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    Array, Vector3,
    array::Constraint,
    array_type,
    validate::{Validate, Validator},
};

/// Defines a 2D grid spacing and size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum Grid2 {
    /// Regularly spaced cells.
    ///
    /// <!--grid2_regular.svg-->
    #[doc = include_str!("../docs/images/grid2_regular.svg")]
    Regular {
        /// The cell size in the U and V axes. Both must be greater than zero.
        size: [f64; 2],
        /// The number of cells in the U and V axes. Both must be greater than zero.
        count: [u32; 2],
    },
    /// Tensor cells, where each row and column can have a different size.
    ///
    /// <!--grid2_tensor.svg-->
    #[doc = include_str!("../docs/images/grid2_tensor.svg")]
    Tensor {
        /// Array with `Scalar` type storing the size of each cell along the U axis.
        /// These sizes must be greater than zero.
        u: Array<array_type::Scalar>,
        /// Array with `Scalar` type storing the size of each cell along the V axis.
        /// These sizes must be greater than zero.
        v: Array<array_type::Scalar>,
    },
}

impl Grid2 {
    /// Create a 2D regular grid from the cell size and count.
    pub fn from_size_and_count(size: [f64; 2], count: [u32; 2]) -> Self {
        Self::Regular { size, count }
    }

    /// Create a 2D tensor grid from the size arrays.
    pub fn from_arrays(u: Array<array_type::Scalar>, v: Array<array_type::Scalar>) -> Self {
        Self::Tensor { u, v }
    }

    /// Returns the number of cells in each axis.
    pub fn count(&self) -> [u32; 2] {
        match self {
            Self::Regular { count, .. } => *count,
            Self::Tensor { u, v } => [u.item_count() as u32, v.item_count() as u32],
        }
    }

    /// Returns the total number of cells.
    pub fn flat_count(&self) -> u64 {
        self.count().into_iter().map(u64::from).product()
    }

    /// Returns the total number of cell corners.
    pub fn flat_corner_count(&self) -> u64 {
        self.count().into_iter().map(|n| u64::from(n) + 1).product()
    }
}

impl Default for Grid2 {
    /// Creates a regular grid with size `[1.0, 1.0]` and count `[1, 1]`.
    fn default() -> Self {
        Self::Regular {
            size: [1.0, 1.0],
            count: [1, 1],
        }
    }
}

/// Defines a 3D grid spacing and size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
pub enum Grid3 {
    /// Regularly spaced cells.
    ///
    /// <!--grid3_regular.svg-->
    #[doc = include_str!("../docs/images/grid3_regular.svg")]
    Regular {
        /// The block size in the U and V axes. All must be greater than zero.
        size: Vector3,
        /// The number of cells in the U, V, and W axes. All must be greater than zero.
        count: [u32; 3],
    },
    /// Tensor cells, where each row, column, and layer can have a different size. All sizes
    /// must be greater than zero.
    ///
    /// <!--grid3_tensor.svg-->
    #[doc = include_str!("../docs/images/grid3_tensor.svg")]
    Tensor {
        /// Array with `Scalar` type storing the size of each cell along the U axis.
        /// These sizes must be greater than zero.
        u: Array<array_type::Scalar>,
        /// Array with `Scalar` type storing the size of each cell along the V axis.
        /// These sizes must be greater than zero.
        v: Array<array_type::Scalar>,
        /// Array with `Scalar` type storing the size of each cell along the W axis.
        /// These sizes must be greater than zero.
        w: Array<array_type::Scalar>,
    },
}

impl Grid3 {
    /// Create a 3D regular grid from the block size and count.
    pub fn from_size_and_count(size: Vector3, count: [u32; 3]) -> Self {
        Self::Regular { size, count }
    }

    /// Create a 3D tensor grid from the size arrays.
    pub fn from_arrays(
        u: Array<array_type::Scalar>,
        v: Array<array_type::Scalar>,
        w: Array<array_type::Scalar>,
    ) -> Self {
        Self::Tensor { u, v, w }
    }

    /// Returns the number of blocks in each axis.
    pub fn count(&self) -> [u32; 3] {
        match self {
            Self::Regular { count, .. } => *count,
            // validation checks that this cast is valid
            Self::Tensor { u, v, w } => [
                u.item_count() as u32,
                v.item_count() as u32,
                w.item_count() as u32,
            ],
        }
    }

    /// Returns the total number of blocks.
    pub fn flat_count(&self) -> u64 {
        self.count().iter().map(|n| u64::from(*n)).product()
    }

    /// Returns the total number of block corners.
    pub fn flat_corner_count(&self) -> u64 {
        self.count().iter().map(|n| u64::from(*n) + 1).product()
    }
}

impl Default for Grid3 {
    fn default() -> Self {
        Self::Regular {
            size: [1.0, 1.0, 1.0],
            count: [1, 1, 1],
        }
    }
}

const fn i() -> Vector3 {
    [1.0, 0.0, 0.0]
}

const fn j() -> Vector3 {
    [0.0, 1.0, 0.0]
}

const fn k() -> Vector3 {
    [0.0, 0.0, 1.0]
}

/// Defines the position and orientation of a 2D plane in 3D space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[repr(C)]
pub struct Orient2 {
    /// Origin point relative to the project origin and coordinate reference.
    pub origin: Vector3,
    /// The direction of the U axis of the plane. Must be a unit vector. Default [1, 0, 0].
    ///
    /// Must also be perpendicular to the 'v' in grid surfaces.
    #[serde(default = "i")]
    pub u: Vector3,
    /// The direction of the V axis of the plane. Must be a unit vector. Default [0, 1, 0].
    ///
    /// Must also be perpendicular to the 'u' in grid surfaces.
    #[serde(default = "j")]
    pub v: Vector3,
}

impl Orient2 {
    /// Creates a new 2D orientation.
    pub fn new(origin: Vector3, u: Vector3, v: Vector3) -> Self {
        Self { origin, u, v }
    }

    /// Creates a new axis-aligned 2D orientation.
    pub fn from_origin(origin: Vector3) -> Self {
        Self::new(origin, i(), j())
    }

    pub(crate) fn validate_ortho(&self, val: &mut Validator) {
        val.enter("Orient2").vectors_ortho2(self.u, self.v);
    }
}

impl Default for Orient2 {
    /// Creates a new axis-aligned 2D orientation at the origin.
    fn default() -> Self {
        Self {
            origin: [0.0; 3],
            u: i(),
            v: j(),
        }
    }
}

/// Defines the position and orientation of a 3D sub-space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[repr(C)]
pub struct Orient3 {
    /// Origin point relative to the project origin and coordinate reference.
    pub origin: Vector3,
    /// The direction of the U axis of the grid. Must be a unit vector perpendicular to
    /// `v` and 'w'. Default [1, 0, 0].
    #[serde(default = "i")]
    pub u: Vector3,
    /// The direction of the V axis of the grid. Must be a unit vector perpendicular to
    /// `u` and 'w'. Default [0, 1, 0].
    #[serde(default = "j")]
    pub v: Vector3,
    /// The direction of the W axis of the grid. Must be a unit vector perpendicular to
    /// `u` and 'v'. Default [0, 0, 1].
    #[serde(default = "k")]
    pub w: Vector3,
}

impl Orient3 {
    /// Creates a new 3D orientation.
    pub fn new(origin: Vector3, u: Vector3, v: Vector3, w: Vector3) -> Self {
        Self { origin, u, v, w }
    }

    /// Creates a new axis-aligned 3D orientation.
    pub fn from_origin(origin: Vector3) -> Self {
        Self::new(origin, i(), j(), k())
    }
}

impl Default for Orient3 {
    fn default() -> Self {
        Self {
            origin: [0.0; 3],
            u: i(),
            v: j(),
            w: k(),
        }
    }
}

impl Validate for Grid2 {
    fn validate_inner(&mut self, val: &mut Validator) {
        match self {
            Grid2::Regular { size, count } => {
                val.enter("Grid2::Regular")
                    .finite_seq(*size, "size")
                    .above_zero_seq(*size, "size")
                    .above_zero_seq(*count, "count");
            }
            Grid2::Tensor { u, v } => {
                val.enter("Grid2::Tensor")
                    .grid_count(&[u.item_count(), v.item_count()])
                    .array(u, Constraint::Size, "u")
                    .array(v, Constraint::Size, "v");
            }
        }
    }
}

impl Validate for Grid3 {
    fn validate_inner(&mut self, val: &mut Validator) {
        match self {
            Grid3::Regular { size, count } => {
                val.enter("Grid3::Regular")
                    .finite_seq(*size, "size")
                    .above_zero_seq(*size, "size")
                    .above_zero_seq(*count, "count");
            }
            Grid3::Tensor { u, v, w } => {
                val.enter("Grid3::Tensor")
                    .grid_count(&[u.item_count(), v.item_count(), w.item_count()])
                    .array(u, Constraint::Size, "u")
                    .array(v, Constraint::Size, "v")
                    .array(w, Constraint::Size, "w");
            }
        }
    }
}

impl Validate for Orient2 {
    fn validate_inner(&mut self, val: &mut Validator) {
        val.enter("Orient2")
            .finite_seq(self.origin, "origin")
            .unit_vector(self.u, "u")
            .unit_vector(self.v, "v");
    }
}

impl Validate for Orient3 {
    fn validate_inner(&mut self, val: &mut Validator) {
        val.enter("Orient3")
            .finite_seq(self.origin, "origin")
            .unit_vector(self.u, "u")
            .unit_vector(self.v, "v")
            .unit_vector(self.w, "w")
            .vectors_ortho3(self.u, self.v, self.w);
    }
}
