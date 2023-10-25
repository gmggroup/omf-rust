use std::{collections::HashSet, fs::File};

use crate::{
    array::Constraint,
    error::{Error, InvalidData},
    file::SubFile,
    pqarray::read::{MultiIter, NullableGroupIter, NullableIter, SimpleIter},
    SubblockMode,
};

use super::{
    write_checks::{subblock_is_octree_compat, valid_subblock_sizes},
    FloatType, NumberType,
};

/// Iterator for reading scalar data, supporting `f32` and `f64` types generically.
///
/// When used for tensor block model sizes this checks that all sizes are greater than zero.
#[derive(Debug)]
pub struct GenericScalars<T: FloatType> {
    inner: SimpleIter<T, SubFile<File>>,
    is_size: bool,
}

impl<T: FloatType> GenericScalars<T> {
    pub(crate) fn new(inner: SimpleIter<T, SubFile<File>>, constraint: &Constraint) -> Self {
        Self {
            inner,
            is_size: matches!(constraint, Constraint::Size),
        }
    }
}

impl<T: FloatType> Iterator for GenericScalars<T> {
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;
        if self.is_size {
            if let Ok(value) = item {
                if value <= T::default() {
                    return Some(Err(InvalidData::SizeZeroOrLess {
                        value: value.into(),
                    }
                    .into()));
                }
            }
        }
        Some(item)
    }
}

/// Iterator for reading nullable indices.
///
/// Checks that all indices are within range.
#[derive(Debug)]
pub struct Indices {
    inner: NullableIter<u32, SubFile<File>>,
    category_count: u64,
}

impl Indices {
    pub(crate) fn new(inner: NullableIter<u32, SubFile<File>>, constraint: &Constraint) -> Self {
        let &Constraint::Index(category_count) = constraint else {
            panic!("invalid constraint");
        };
        Self {
            inner,
            category_count,
        }
    }
}

impl Iterator for Indices {
    type Item = Result<Option<u32>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;
        if let Ok(Some(i)) = &item {
            let index: u64 = (*i).into();
            if index >= self.category_count {
                return Some(Err(InvalidData::IndexOutOfRange {
                    value: index,
                    maximum: self.category_count.saturating_add(1),
                }
                .into()));
            }
        }
        Some(item)
    }
}

/// Iterator for reading segments or triangles.
///
/// Checks that all vertex indices are within range.
#[derive(Debug)]
pub struct GenericPrimitives<const N: usize> {
    inner: MultiIter<u32, SubFile<File>, N>,
    vertex_count: u64,
}

impl<const N: usize> GenericPrimitives<N> {
    pub(crate) fn new(inner: MultiIter<u32, SubFile<File>, N>, constraint: &Constraint) -> Self {
        let vertex_count = match constraint {
            Constraint::Segment(n) | Constraint::Triangle(n) => *n,
            _ => panic!("invalid constraint"),
        };
        Self {
            inner,
            vertex_count,
        }
    }
}

impl<const N: usize> Iterator for GenericPrimitives<N> {
    type Item = Result<[u32; N], Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;
        if let Ok(value) = &item {
            for i in value {
                let index: u64 = (*i).into();
                if index >= self.vertex_count {
                    return Some(Err(InvalidData::IndexOutOfRange {
                        value: index,
                        maximum: self.vertex_count.saturating_add(1),
                    }
                    .into()));
                }
            }
        }
        Some(item)
    }
}

/// Iterator for reading boundary values.
///
/// Checks that the values are increasing.
#[derive(Debug)]
pub(super) struct BoundaryValues<T: NumberType> {
    inner: SimpleIter<T, SubFile<File>>,
    previous: Option<T>,
}

impl<T: NumberType> BoundaryValues<T> {
    pub(crate) fn new(inner: SimpleIter<T, SubFile<File>>) -> Self {
        Self {
            inner,
            previous: None,
        }
    }
}

impl<T: NumberType> Iterator for BoundaryValues<T> {
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;
        if let (Ok(v), Some(p)) = (&item, &self.previous) {
            if v < p {
                return Some(Err(InvalidData::BoundaryDecreases.into()));
            }
        }
        Some(item)
    }
}

/// Iterator for reading free-form sub-blocks, supporting `f32` or `f64` generically.
///
/// Checks that the parent index and corners are all valid.
#[derive(Debug)]
pub struct GenericFreeformSubblocks<T: FloatType> {
    parents: MultiIter<u32, SubFile<File>, 3>,
    corners: MultiIter<T, SubFile<File>, 6>,
    block_count: [u32; 3],
}

impl<T: FloatType> GenericFreeformSubblocks<T> {
    pub(crate) fn new(
        parents: MultiIter<u32, SubFile<File>, 3>,
        corners: MultiIter<T, SubFile<File>, 6>,
        constraint: &Constraint,
    ) -> Self {
        let block_count = match constraint {
            &Constraint::RegularSubblock { block_count, .. }
            | &Constraint::FreeformSubblock { block_count } => block_count,
            _ => panic!("invalid constraint"),
        };
        Self {
            parents,
            corners,
            block_count,
        }
    }
}

impl<T: FloatType> Iterator for GenericFreeformSubblocks<T> {
    type Item = Result<([u32; 3], [T; 6]), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.parents.next(), self.corners.next()) {
            (None, _) | (_, None) => None,
            (Some(Err(e)), _) | (_, Some(Err(e))) => Some(Err(e)),
            (Some(Ok(parent)), Some(Ok(corners))) => {
                if let Err(e) = check_freeform_corners(corners) {
                    Some(Err(e))
                } else if let Err(e) = check_parent(self.block_count, parent) {
                    Some(Err(e))
                } else {
                    Some(Ok((parent, corners)))
                }
            }
        }
    }
}

/// Iterator for reading regular sub-blocks.
///
/// Checks that the parent index and corners are all valid.
#[derive(Debug)]
pub struct RegularSubblocks {
    parents: MultiIter<u32, SubFile<File>, 3>,
    corners: MultiIter<u32, SubFile<File>, 6>,
    block_count: [u32; 3],
    subblock_count: [u32; 3],
    mode: Option<(SubblockMode, HashSet<[u32; 3]>)>,
}

impl RegularSubblocks {
    pub(crate) fn new(
        parents: MultiIter<u32, SubFile<File>, 3>,
        corners: MultiIter<u32, SubFile<File>, 6>,
        constraint: &Constraint,
    ) -> Self {
        let &Constraint::RegularSubblock {
            block_count,
            subblock_count,
            mode,
        } = constraint
        else {
            panic!("invalid constraint");
        };
        Self {
            parents,
            corners,
            block_count,
            subblock_count,
            mode: mode.map(|m| (m, valid_subblock_sizes(m, subblock_count))),
        }
    }
}

impl Iterator for RegularSubblocks {
    type Item = Result<([u32; 3], [u32; 6]), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.parents.next(), self.corners.next()) {
            (None, _) | (_, None) => None,
            (Some(Err(e)), _) | (_, Some(Err(e))) => Some(Err(e)),
            (Some(Ok(parent)), Some(Ok(corners))) => {
                if let Err(e) = check_regular_corners(self.subblock_count, &self.mode, corners) {
                    Some(Err(e))
                } else if let Err(e) = check_parent(self.block_count, parent) {
                    Some(Err(e))
                } else {
                    Some(Ok((parent, corners)))
                }
            }
        }
    }
}

#[inline]
fn check_parent(block_count: [u32; 3], parent: [u32; 3]) -> Result<(), Error> {
    for (count, index) in block_count.into_iter().zip(parent) {
        if index >= count {
            return Err(InvalidData::BlockIndexOutOfRange {
                value: parent.map(Into::into),
                maximum: block_count,
            }
            .into());
        }
    }
    Ok(())
}

#[inline]
fn check_regular_corners(
    subblock_count: [u32; 3],
    mode_and_sizes: &Option<(SubblockMode, HashSet<[u32; 3]>)>,
    corners: [u32; 6],
) -> Result<(), Error> {
    let corners: [u32; 6] = corners.map(Into::into);
    for i in 0..3 {
        if corners[i] >= corners[i + 3] {
            return Err(InvalidData::RegularSubblockZeroSize { corners }.into());
        }
        if corners[i + 3] > subblock_count[i] {
            return Err(InvalidData::RegularSubblockOutsideParent {
                corners,
                maximum: subblock_count,
            }
            .into());
        }
    }
    if let Some((mode, valid_sizes)) = &mode_and_sizes {
        let size = std::array::from_fn(|i| corners[i + 3] - corners[i]);
        if !valid_sizes.contains(&size) {
            return Err(InvalidData::RegularSubblockNotInMode {
                corners,
                mode: *mode,
            }
            .into());
        }
        if *mode == SubblockMode::Octree && !subblock_is_octree_compat(&corners) {
            return Err(InvalidData::RegularSubblockNotInMode {
                corners,
                mode: *mode,
            }
            .into());
        }
    }
    Ok(())
}

#[inline]
fn check_freeform_corners<T: FloatType>(corners: [T; 6]) -> Result<(), Error> {
    for i in 0..3 {
        if corners[i] < T::ZERO {
            return Err(InvalidData::FreeformSubblockOutsideParent {
                corners: corners.map(Into::into),
            }
            .into());
        }
        if corners[i + 3] > T::ONE {
            return Err(InvalidData::FreeformSubblockOutsideParent {
                corners: corners.map(Into::into),
            }
            .into());
        }
        if corners[i] >= corners[i + 3] {
            return Err(InvalidData::FreeformSubblockZeroSize {
                corners: corners.map(Into::into),
            }
            .into());
        }
    }
    Ok(())
}

/// Iterator for reading numbers, supporting `f32`, `f64`, `i64`, date, and date-time generically.
#[derive(Debug)]
pub struct GenericNumbers<T: NumberType>(pub(crate) NullableIter<T, SubFile<File>>);

impl<T: NumberType> Iterator for GenericNumbers<T> {
    type Item = Result<Option<T>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Iterator for reading small fixed-size arrays, like vertices and texture coordinates.
#[derive(Debug)]
pub struct GenericArrays<T: NumberType, const N: usize>(pub(crate) MultiIter<T, SubFile<File>, N>);

impl<T: NumberType, const N: usize> Iterator for GenericArrays<T, N> {
    type Item = Result<[T; N], Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Iterator for reading non-nullable colors, such as category legends.
#[derive(Debug)]
pub struct Gradient(pub(crate) MultiIter<u8, SubFile<File>, 4>);

impl Iterator for Gradient {
    type Item = Result<[u8; 4], Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Iterator for reading small optional fixed-size arrays, like 2D and 3D vectors.
#[derive(Debug)]
pub struct GenericOptionalArrays<T: NumberType, const N: usize>(
    pub(crate) NullableGroupIter<T, SubFile<File>, N>,
);

impl<T: NumberType, const N: usize> Iterator for GenericOptionalArrays<T, N> {
    type Item = Result<Option<[T; N]>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Iterator for reading nullable colors.
#[derive(Debug)]
pub struct Colors(pub(crate) NullableGroupIter<u8, SubFile<File>, 4>);

impl Iterator for Colors {
    type Item = Result<Option<[u8; 4]>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Iterator for reading nullable booleans.
#[derive(Debug)]
pub struct Booleans(pub(crate) NullableIter<bool, SubFile<File>>);

impl Iterator for Booleans {
    type Item = Result<Option<bool>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Iterator for reading non-nullable strings, such as category names.
#[derive(Debug)]
pub struct Names(pub(crate) SimpleIter<String, SubFile<File>>);

impl Iterator for Names {
    type Item = Result<String, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Iterator for reading nullable strings.
#[derive(Debug)]
pub struct Text(pub(crate) NullableIter<String, SubFile<File>>);

impl Iterator for Text {
    type Item = Result<Option<String>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
