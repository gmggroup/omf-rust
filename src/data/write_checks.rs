use std::collections::HashSet;

use crate::{
    array::{Array, ArrayType, Constraint},
    error::InvalidData,
    validate::Reason,
    SubblockMode,
};

use super::NumberType;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ArrayWriteCheck {
    MinimumScalar(f64),
    MaximumIndex(u32),
    Subblocks([[u32; 3]; 3]),
    RegularSubblocksMaximum([[u32; 6]; 3]),
    RegularSubblockCorners(HashSet<[u32; 6]>),
    Invalid(InvalidData),
}

impl ArrayWriteCheck {
    pub fn check(&self, array: &Array<impl ArrayType>) -> Result<(), Reason> {
        match (self, array.constraint()) {
            (Self::MinimumScalar(min), Constraint::Size) => {
                // Check that minimum scalar value is not zero or less.
                if *min <= 0.0 {
                    return Err(Reason::InvalidData(InvalidData::SizeZeroOrLess {
                        value: *min,
                    }));
                }
            }
            (
                Self::MaximumIndex(max_index),
                Constraint::Segment(count) | Constraint::Triangle(count) | Constraint::Index(count),
            ) => {
                // Check the maximum index against the vertex count.
                if u64::from(*max_index) >= *count {
                    return Err(Reason::InvalidData(InvalidData::IndexOutOfRange {
                        value: u64::from(*max_index),
                        maximum: (*count).saturating_sub(1),
                    }));
                }
            }
            (
                Self::Subblocks(maximum_parents),
                Constraint::RegularSubblock { block_count, .. }
                | Constraint::FreeformSubblock { block_count, .. },
            ) => {
                // Check parent block indices against the block count.
                for (i, parent) in maximum_parents.iter().enumerate() {
                    if parent[i] >= block_count[i] {
                        return Err(Reason::InvalidData(InvalidData::BlockIndexOutOfRange {
                            value: *parent,
                            maximum: block_count.map(|n| n.saturating_sub(1)),
                        }));
                    }
                }
            }
            (
                Self::RegularSubblocksMaximum(maximums),
                Constraint::RegularSubblock { subblock_count, .. },
            ) => {
                // Check regular sub-block indices against the sub-block count.
                for (i, corners) in maximums.iter().enumerate() {
                    if corners[i + 3] > subblock_count[i] {
                        return Err(Reason::InvalidData(
                            InvalidData::RegularSubblockOutsideParent {
                                corners: *corners,
                                maximum: *subblock_count,
                            },
                        ));
                    }
                }
            }
            (
                Self::RegularSubblockCorners(all_corners),
                Constraint::RegularSubblock {
                    subblock_count,
                    mode: Some(mode),
                    ..
                },
            ) => {
                // Check regular sub-block indices against the sub-block count.
                let valid_sizes = valid_subblock_sizes(*mode, *subblock_count);
                for corners in all_corners {
                    let size = std::array::from_fn(|i| corners[i + 3] - corners[i]);
                    if !valid_sizes.contains(&size)
                        || (*mode == SubblockMode::Octree && !subblock_is_octree_compat(corners))
                    {
                        return Err(Reason::InvalidData(InvalidData::RegularSubblockNotInMode {
                            corners: *corners,
                            mode: *mode,
                        }));
                    }
                }
            }
            (Self::Invalid(inv), _) => {
                // Pass error on.
                return Err(Reason::InvalidData(inv.clone()));
            }
            _ => (),
        }
        Ok(())
    }
}

pub(crate) fn subblock_is_octree_compat(corners: &[u32; 6]) -> bool {
    (0..3).all(|i| corners[i] % (corners[i + 3] - corners[i]) == 0)
}

pub(crate) fn valid_subblock_sizes(
    mode: SubblockMode,
    subblock_count: [u32; 3],
) -> HashSet<[u32; 3]> {
    let mut valid_sizes = HashSet::new();
    match mode {
        SubblockMode::Octree => {
            let mut size = subblock_count;
            loop {
                valid_sizes.insert(size);
                if size == [1, 1, 1] {
                    break;
                }
                size = size.map(|n| (n / 2).max(1));
            }
        }
        SubblockMode::Full => {
            valid_sizes.insert([1, 1, 1]);
            valid_sizes.insert(subblock_count);
        }
    }
    valid_sizes
}

pub(crate) struct MaximumIndex(u32);

impl MaximumIndex {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn visit<T: Copy + Into<u32>>(&mut self, value: T) -> T {
        self.0 = self.0.max(value.into());
        value
    }

    pub fn visit_opt<T: Copy + Into<u32>>(&mut self, value: Option<T>) -> Option<T> {
        value.map(|v| self.visit(v))
    }

    pub fn visit_array<T: Copy + Into<u32>, const N: usize>(&mut self, value: [T; N]) -> [T; N] {
        for v in value {
            self.visit(v);
        }
        value
    }

    pub fn get(self) -> Vec<ArrayWriteCheck> {
        vec![ArrayWriteCheck::MaximumIndex(self.0)]
    }
}

pub(crate) struct MinimumScalar(f64);

impl MinimumScalar {
    pub fn new() -> Self {
        Self(f64::INFINITY)
    }

    pub fn visit<T: Copy + Into<f64>>(&mut self, value: T) -> T {
        self.0 = self.0.min(value.into());
        value
    }

    pub fn get(self) -> Vec<ArrayWriteCheck> {
        vec![ArrayWriteCheck::MinimumScalar(self.0)]
    }
}

pub(crate) struct IncreasingBoundary<T> {
    previous: Option<T>,
    ok: bool,
}

impl<T: NumberType> IncreasingBoundary<T> {
    pub fn new() -> Self {
        Self {
            previous: None,
            ok: true,
        }
    }

    pub fn visit(&mut self, value: T) -> T {
        if let Some(previous) = self.previous {
            if value < previous {
                self.ok = false;
            }
        }
        self.previous = Some(value);
        value
    }

    pub fn get(self) -> Vec<ArrayWriteCheck> {
        if self.ok {
            vec![]
        } else {
            vec![ArrayWriteCheck::Invalid(InvalidData::BoundaryDecreases)]
        }
    }
}

pub(crate) struct ParentIndices {
    maximums: [[u32; 3]; 3],
}

impl ParentIndices {
    pub fn new() -> Self {
        Self {
            maximums: [[0; 3]; 3],
        }
    }

    pub fn visit<T: Copy + Into<u32>>(&mut self, value: [T; 3]) {
        let parent = value.map(Into::into);
        for i in 0..3 {
            if parent[i] > self.maximums[i][i] {
                self.maximums[i] = parent;
            }
        }
    }

    pub fn get(self) -> Vec<ArrayWriteCheck> {
        vec![ArrayWriteCheck::Subblocks(self.maximums)]
    }
}

pub(crate) struct FreeformCorners(Option<InvalidData>);

impl FreeformCorners {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn visit<T: Copy + Into<f64>>(&mut self, corners: [T; 6]) {
        if self.0.is_none() {
            let corners: [f64; 6] = corners.map(Into::into);
            for i in 0..3 {
                if corners[i] >= corners[i + 3] {
                    self.0 = Some(InvalidData::FreeformSubblockZeroSize { corners });
                    return;
                }
                if corners[i] < 0.0 || corners[i + 3] > 1.0 {
                    self.0 = Some(InvalidData::FreeformSubblockOutsideParent { corners });
                    return;
                }
            }
        }
    }

    pub fn get(self) -> Vec<ArrayWriteCheck> {
        self.0.map(ArrayWriteCheck::Invalid).into_iter().collect()
    }
}

pub(crate) struct RegularCorners {
    maximum_corners: [[u32; 6]; 3],
    corners: HashSet<[u32; 6]>,
    invalid_size: Option<[u32; 6]>,
}

impl RegularCorners {
    pub fn new() -> Self {
        Self {
            maximum_corners: [[0; 6]; 3],
            corners: HashSet::new(),
            invalid_size: None,
        }
    }

    pub fn visit<T: Copy + Into<u32>>(&mut self, corners: [T; 6]) {
        let corners: [u32; 6] = corners.map(Into::into);
        self.corners.insert(corners);
        for i in 0..3 {
            if corners[i + 3] > self.maximum_corners[i][i + 3] {
                self.maximum_corners[i] = corners;
            }
        }
        if self.invalid_size.is_none() {
            for i in 0..3 {
                if corners[i] >= corners[i + 3] {
                    self.invalid_size = Some(corners);
                    break;
                }
            }
        }
    }

    pub fn get(self) -> Vec<ArrayWriteCheck> {
        let mut out = vec![
            ArrayWriteCheck::RegularSubblocksMaximum(self.maximum_corners),
            ArrayWriteCheck::RegularSubblockCorners(self.corners),
        ];
        if let Some(corners) = self.invalid_size {
            out.push(ArrayWriteCheck::Invalid(
                InvalidData::RegularSubblockZeroSize { corners },
            ));
        }
        out
    }
}
