use std::io::{Seek, Write};

use crate::{
    error::Error,
    file::{ReadAt, Writer},
};

use super::{
    objects::{
        Array, DataType, Int2Array, Int3Array, Key, ScalarArray, Vector2Array, Vector3Array,
    },
    reader::Omf1Reader,
    Omf1Error,
};

pub fn vertices_array<W: Write + Seek + Send, R: ReadAt>(
    r: &Omf1Reader<R>,
    w: &mut Writer<W>,
    array: &Key<Vector3Array>,
) -> Result<crate::Array<crate::array_type::Vertex>, Error> {
    let mut iter = HoldError::new(
        ByteChunks(r.array_decompressed_bytes(&r.model(array)?.array)?)
            .map(|r| r.map(vector3_from_le_bytes)),
    );
    let array = w.array_vertices(iter.by_ref())?;
    iter.finish()?;
    Ok(array)
}

pub fn scalars_array<W: Write + Seek + Send, R: ReadAt>(
    r: &Omf1Reader<R>,
    w: &mut Writer<W>,
    array_key: &Key<ScalarArray>,
) -> Result<crate::Array<crate::array_type::Scalar>, Error> {
    let array = &r.model(array_key)?.array;
    // The OMF1 code doesn't require scalar arrays to be float, so allow int too.
    let is_float = array.dtype == DataType::Float;
    let mut iter = items(r, array, move |b| {
        if is_float {
            Ok(f64::from_le_bytes(b))
        } else {
            Ok(i64::from_le_bytes(b) as f64)
        }
    })?;
    let array = w.array_scalars(iter.by_ref())?;
    iter.finish()?;
    Ok(array)
}

pub enum ScalarValues {
    Float(Vec<f64>),
    Int(Vec<i64>),
}

pub fn load_scalars<R: ReadAt>(
    r: &Omf1Reader<R>,
    array: &ScalarArray,
) -> Result<ScalarValues, Error> {
    if array.array.dtype == DataType::Float {
        let mut iter = items(r, &array.array, |b| Ok(f64::from_le_bytes(b)))?;
        let out = iter.by_ref().collect();
        iter.finish()?;
        Ok(ScalarValues::Float(out))
    } else {
        let mut iter = items(r, &array.array, |b| Ok(i64::from_le_bytes(b)))?;
        let out = iter.by_ref().collect();
        iter.finish()?;
        Ok(ScalarValues::Int(out))
    }
}

pub fn numbers_array<W: Write + Seek + Send, R: ReadAt>(
    r: &Omf1Reader<R>,
    w: &mut Writer<W>,
    array_key: &Key<ScalarArray>,
) -> Result<crate::Array<crate::array_type::Number>, Error> {
    let array = &r.model(array_key)?.array;
    if array.dtype == DataType::Float {
        let mut iter = items(r, array, |b| Ok(none_if_nan(f64::from_le_bytes(b))))?;
        let array = w.array_numbers(iter.by_ref())?;
        iter.finish()?;
        Ok(array)
    } else {
        let mut iter = items(r, array, |b| Ok(Some(i64::from_le_bytes(b))))?;
        let array = w.array_numbers(iter.by_ref())?;
        iter.finish()?;
        Ok(array)
    }
}

pub fn segments_array<W: Write + Seek + Send, R: ReadAt>(
    r: &Omf1Reader<R>,
    w: &mut Writer<W>,
    array: &Key<Int2Array>,
) -> Result<crate::Array<crate::array_type::Segment>, Error> {
    let mut iter = items(r, &r.model(array)?.array, segment_from_le_bytes)?;
    let array = w.array_segments(iter.by_ref())?;
    iter.finish()?;
    Ok(array)
}

pub fn triangles_array<W: Write + Seek + Send, R: ReadAt>(
    r: &Omf1Reader<R>,
    w: &mut Writer<W>,
    array: &Key<Int3Array>,
) -> Result<crate::Array<crate::array_type::Triangle>, Error> {
    let mut iter = items(r, &r.model(array)?.array, triangle_from_le_bytes)?;
    let array = w.array_triangles(iter.by_ref())?;
    iter.finish()?;
    Ok(array)
}

pub fn color_array<W: Write + Seek + Send, R: ReadAt>(
    r: &Omf1Reader<R>,
    w: &mut Writer<W>,
    array: &Int3Array,
) -> Result<crate::Array<crate::array_type::Color>, Error> {
    let mut iter = items(r, &array.array, color_from_le_bytes)?;
    let array = w.array_colors(iter.by_ref())?;
    iter.finish()?;
    Ok(array)
}

pub fn vectors2_array<W: Write + Seek + Send, R: ReadAt>(
    r: &Omf1Reader<R>,
    w: &mut Writer<W>,
    array: &Key<Vector2Array>,
) -> Result<crate::Array<crate::array_type::Vector>, Error> {
    let mut iter = items(r, &r.model(array)?.array, |b| {
        Ok(none_if_any_nan(vector2_from_le_bytes(b)))
    })?;
    let array = w.array_vectors(iter.by_ref())?;
    iter.finish()?;
    Ok(array)
}

pub fn vectors3_array<W: Write + Seek + Send, R: ReadAt>(
    r: &Omf1Reader<R>,
    w: &mut Writer<W>,
    array: &Key<Vector3Array>,
) -> Result<crate::Array<crate::array_type::Vector>, Error> {
    let mut iter = items(r, &r.model(array)?.array, |b| {
        Ok(none_if_any_nan(vector3_from_le_bytes(b)))
    })?;
    let array = w.array_vectors(iter.by_ref())?;
    iter.finish()?;
    Ok(array)
}

pub fn index_array<W: Write + Seek + Send, R: ReadAt>(
    r: &Omf1Reader<R>,
    w: &mut Writer<W>,
    array_key: &Key<ScalarArray>,
) -> Result<(u32, crate::Array<crate::array_type::Index>), Error> {
    let array = &r.model(array_key)?.array;
    let is_float = array.dtype == DataType::Float;
    let mut maximum = 0_u32;
    let mut iter = items(r, array, |b| {
        let n = if is_float {
            let x = f64::from_le_bytes(b);
            if x.floor() != x {
                return Err(Omf1Error::NonIntegerArray.into());
            }
            x as i64
        } else {
            i64::from_le_bytes(b)
        };
        if n == -1 {
            Ok(None)
        } else if n < 0 || n > (u32::MAX as i64) {
            Err(Omf1Error::IndexOutOfRange { index: n }.into())
        } else {
            maximum = maximum.max(n as u32);
            Ok(Some(n as u32))
        }
    })?;
    let array = w.array_indices(iter.by_ref())?;
    iter.finish()?;
    Ok((maximum, array))
}

fn items<T, const N: usize, R: ReadAt>(
    r: &Omf1Reader<R>,
    array: &Array,
    mut func: impl FnMut([u8; N]) -> Result<T, Error>,
) -> Result<HoldError<impl Iterator<Item = Result<T, Error>>, Error>, Error>
where
    [u8; N]: Default,
{
    Ok(HoldError::new(
        ByteChunks(r.array_decompressed_bytes(array)?)
            .map(move |r| r.map_err(Error::from).and_then(&mut func)),
    ))
}

fn from_bytes<T, const B: usize, const N: usize, const M: usize>(
    bytes: [u8; B],
    func: impl Fn([u8; N]) -> T,
) -> [T; M] {
    debug_assert_eq!(M, B / N);
    std::array::from_fn(|i| func((&bytes[(i * N)..((i + 1) * N)]).try_into().unwrap()))
}

fn segment_from_le_bytes(bytes: [u8; 16]) -> Result<[u32; 2], Error> {
    let [a, b] = from_bytes(bytes, |b| {
        let n = i64::from_le_bytes(b);
        n.try_into()
            .map_err(|_| Omf1Error::IndexOutOfRange { index: n })
    });
    Ok([a?, b?])
}

fn triangle_from_le_bytes(bytes: [u8; 24]) -> Result<[u32; 3], Error> {
    let [a, b, c] = from_bytes(bytes, |b| {
        let n = i64::from_le_bytes(b);
        n.try_into()
            .map_err(|_| Omf1Error::IndexOutOfRange { index: n })
    });
    Ok([a?, b?, c?])
}

fn color_from_le_bytes(bytes: [u8; 24]) -> Result<Option<[u8; 4]>, Error> {
    let [r, g, b] = from_bytes(bytes, |b| {
        i64::from_le_bytes(b).clamp(0, u8::MAX as i64) as u8
    });
    Ok(Some([r, g, b, u8::MAX]))
}

fn vector2_from_le_bytes(bytes: [u8; 16]) -> [f64; 2] {
    from_bytes(bytes, f64::from_le_bytes)
}

fn vector3_from_le_bytes(bytes: [u8; 24]) -> [f64; 3] {
    from_bytes(bytes, f64::from_le_bytes)
}

fn none_if_nan(input: f64) -> Option<f64> {
    if input.is_nan() {
        None
    } else {
        Some(input)
    }
}

fn none_if_any_nan<const N: usize>(input: [f64; N]) -> Option<[f64; N]> {
    if input.into_iter().any(f64::is_nan) {
        None
    } else {
        Some(input)
    }
}

struct ByteChunks<I, const N: usize>(I);

impl<I, const N: usize> Iterator for ByteChunks<I, N>
where
    [u8; N]: Default + Copy,
    I: Iterator<Item = Result<u8, std::io::Error>>,
{
    type Item = Result<[u8; N], std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut item = [0_u8; N];
        for byte in item.iter_mut() {
            match self.0.next() {
                Some(Ok(b)) => *byte = b,
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        }
        Some(Ok(item))
    }
}

pub struct HoldError<I, E> {
    inner: I,
    result: Result<(), E>,
}

impl<I, E> HoldError<I, E> {
    fn new(inner: I) -> Self {
        Self {
            inner,
            result: Ok(()),
        }
    }

    pub fn finish(self) -> Result<(), E> {
        self.result
    }
}

impl<T, I, E> Iterator for HoldError<I, E>
where
    I: Iterator<Item = Result<T, E>>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            None => None,
            Some(Err(e)) => {
                self.result = Err(e);
                None
            }
            Some(Ok(byte)) => Some(byte),
        }
    }
}
