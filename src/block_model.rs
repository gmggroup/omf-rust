use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    Array, Grid3, Location, Orient3,
    array::Constraint,
    array_type,
    validate::{Validate, Validator},
};

/// Block model geometry with optional sub-blocks.
///
/// First, the `orient` field defines the position and orientation of a (U, V, W) space relative
/// to the project, which could be just an offset or a full rotation as well. Then the `grid`
/// field defines the size and number of parent blocks aligned with that space and starting at
/// (0, 0, 0). [Sub-blocks](crate::Subblocks) can then optionally be added inside those parent
/// blocks using a variety of layouts.
///
/// While sub-blocks are supported on tensor grids it isn't a common arrangement and many
/// applications won't load them.
///
/// ### Attribute Locations
///
/// - [`Vertices`](crate::Location::Vertices) puts attribute values on the corners of the
///   parent blocks. If the block count is $(N_0, N_1, N_2)$ then there must be
///   $(N_0 + 1) · (N_1 + 1) · (N_2 + 1)$ values. Ordering increases U first, then V, then W.
///
/// - [`Blocks`](crate::Location::Primitives) puts attribute values on the centroids of the
///   parent block. If the block count is $(N_0, N_1, N_2)$ then there must be
///   $N_0 · N_1 · N_2$ values. Ordering increases U first, then V, then W.
///
/// - [`Subblocks`](crate::Location::Subblocks) puts attribute values on sub-block centroids.
///   The number and values and their ordering matches the `parents` and `corners` arrays.
///
///   To have attribute values on undivided parent blocks in this mode there must be a sub-block
///   that covers the whole parent block.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BlockModel {
    /// Orientation of the block model.
    pub orient: Orient3,
    /// Block sizes.
    pub grid: Grid3,
    /// Optional sub-blocks, which can be regular or free-form divisions of the parent blocks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subblocks: Option<Subblocks>,
}

impl BlockModel {
    pub fn new(orient: Orient3, grid: Grid3) -> Self {
        Self {
            orient,
            grid,
            subblocks: None,
        }
    }

    pub fn with_subblocks(orient: Orient3, grid: Grid3, subblocks: Subblocks) -> Self {
        Self {
            orient,
            grid,
            subblocks: Some(subblocks),
        }
    }

    pub fn with_regular_subblocks(
        orient: Orient3,
        grid: Grid3,
        subblock_count: [u32; 3],
        subblocks: Array<array_type::RegularSubblock>,
        mode: Option<SubblockMode>,
    ) -> Self {
        Self {
            orient,
            grid,
            subblocks: Some(Subblocks::Regular {
                count: subblock_count,
                subblocks,
                mode,
            }),
        }
    }

    pub fn with_freeform_subblocks(
        orient: Orient3,
        grid: Grid3,
        subblocks: Array<array_type::FreeformSubblock>,
    ) -> Self {
        Self {
            orient,
            grid,
            subblocks: Some(Subblocks::Freeform { subblocks }),
        }
    }

    /// Returns true if the model has sub-blocks.
    pub fn has_subblocks(&self) -> bool {
        self.subblocks.is_some()
    }

    pub fn location_len(&self, location: Location) -> Option<u64> {
        match (&self.subblocks, location) {
            (_, Location::Vertices) => Some(self.grid.flat_corner_count()),
            (_, Location::Primitives) => Some(self.grid.flat_count()),
            (Some(s), Location::Subblocks) => Some(s.len()),
            _ => None,
        }
    }
}

/// Stores sub-blocks of a block model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum Subblocks {
    /// Divide each parent block into a regular grid of `count` cells. Sub-blocks each covers
    /// a non-overlapping cuboid subset of that grid.
    ///
    /// Sub-blocks are described by the `parents` and `corners` arrays. Those arrays must be the
    /// same length and matching rows in each describe the same sub-block. Each row in `parents`
    /// is an IJK index on the block model grid. Each row of
    /// `corners` is $(i_{min}, j_{min}, k_{min}, i_{max}, j_{max}, k_{max})$, all integers, that
    /// refer to the *vertices* of the sub-block grid within the parent block. For example:
    ///
    /// - A block with minimum size in the corner of the parent block would be (0, 0, 0, 1, 1, 1).
    ///
    /// - If the `subblock_count` is (5, 5, 3) then a sub-block covering the whole parent would
    ///   be (0, 0, 0, 5, 5, 3).
    ///
    /// Sub-blocks must stay within their parent, must have a non-zero size in all directions, and
    /// should not overlap. Further restrictions can be applied by the `mode` field, see
    /// [`SubblockMode`](crate::SubblockMode) for details.
    ///
    /// ![Example of regular sub-blocks](../images/subblocks_regular.svg "Regular sub-bloks")
    Regular {
        /// The sub-block grid size.
        ///
        /// Must be greater than zero in all directions. If `mode` is octree then these must also
        /// be powers of two but they don't have to be equal.
        count: [u32; 3],
        /// Array with `RegularSubblock` type storing the sub-block parent indices and corners
        /// relative to the sub-block grid within the parent.
        subblocks: Array<array_type::RegularSubblock>,
        /// If present this further restricts the sub-block layout.
        mode: Option<SubblockMode>,
    },
    /// Divide each parent block into any number and arrangement of non-overlapping cuboid regions.
    ///
    /// Sub-blocks are described by the `parents` and `corners` arrays. Each row in `parents` is
    /// an IJK index on the block model grid. Each row of `corners` is
    /// $(i_{min}, j_{min}, k_{min}, i_{max}, j_{max}, k_{max})$ in floating-point and relative
    /// to the parent block, running from 0.0 to 1.0 across the parent. For example:
    ///
    /// - A sub-block covering the whole parent will be (0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    ///   no matter the size of the parent.
    ///
    /// - A sub-block covering the bottom third of the parent block would be
    ///   (0.0, 0.0, 0.0, 1.0, 1.0, 0.3333) and one covering the top two-thirds would be
    ///   (0.0, 0.0, 0.3333, 1.0, 1.0, 1.0), again no matter the size of the parent.
    ///
    /// Sub-blocks must stay within their parent, must have a non-zero size in all directions,
    /// and shouldn't overlap.
    Freeform {
        /// Array with `FreeformSubblock` type storing the sub-block parent indices and corners
        /// relative to the parent.
        subblocks: Array<array_type::FreeformSubblock>,
    },
}

impl Subblocks {
    /// The number of sub-blocks.
    pub fn len(&self) -> u64 {
        match self {
            Self::Regular { subblocks, .. } => subblocks.item_count(),
            Self::Freeform { subblocks, .. } => subblocks.item_count(),
        }
    }

    /// True if there are no sub-blocks.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the optional sub-block mode.
    ///
    /// Currently this will always be `None` for free-form sub-blocks.
    pub fn mode(&self) -> Option<SubblockMode> {
        match self {
            Subblocks::Regular { mode, .. } => *mode,
            _ => None,
        }
    }

    fn validate(&mut self, block_count: [u32; 3], val: &mut Validator) {
        match self {
            Subblocks::Regular {
                count,
                subblocks,
                mode,
            } => {
                val.enter("Subblocks::Regular")
                    .above_zero_seq(*count, "count")
                    .subblock_mode_and_count(*mode, *count)
                    .array(
                        subblocks,
                        Constraint::RegularSubblock {
                            block_count,
                            subblock_count: *count,
                            mode: *mode,
                        },
                        "subblocks",
                    );
            }
            Subblocks::Freeform { subblocks } => {
                val.enter("Subblocks::Freeform").array(
                    subblocks,
                    Constraint::FreeformSubblock { block_count },
                    "subblocks",
                );
            }
        }
    }
}

/// A optional mode for regular sub-blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum SubblockMode {
    /// Sub-blocks form a octree-like inside the parent block.
    ///
    /// To form this structure, cut the parent block in half in all directions to create
    /// eight child blocks. Repeat that cut for some or all of those children, and continue
    /// doing that until the limit on sub-block count is reached or until the sub-blocks
    /// accurately model the inputs.
    ///
    /// The sub-block count must be a power of two in each direction. This isn't strictly an
    /// octree because the sub-block count doesn't have to be the *same* in all directions.
    /// For example you can have count (16, 16, 2) and blocks will stop dividing the the W
    /// direction after the first split.
    Octree,
    /// Parent blocks are fully divided or not divided at all.
    ///
    /// Applications reading this mode may choose to merge sub-blocks with matching attributes
    /// to reduce the overall number of them.
    Full,
}

impl Validate for BlockModel {
    fn validate_inner(&mut self, val: &mut Validator) {
        let mut v = val
            .enter("BlockModel")
            .obj(&mut self.orient)
            .obj(&mut self.grid);
        if let Some(subblocks) = &mut self.subblocks {
            subblocks.validate(self.grid.count(), &mut v);
        }
    }
}
