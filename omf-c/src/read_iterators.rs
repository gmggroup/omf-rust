use std::{ffi::c_char, fs::File, ptr::null};

use crate::{
    error::{set_error, Error},
    ffi_tools::{arg::not_null_mut, catch},
};

// Utility functions.

macro_rules! inner {
    ($arg:ident) => {
        &mut not_null_mut!($arg)?.0
    };
}

fn alloc<T>(t: T) -> *mut T {
    Box::into_raw(Box::new(t))
}

fn free<T>(obj: *mut T) {
    if !obj.is_null() {
        unsafe {
            drop(Box::from_raw(obj));
        }
    }
}

fn width_cast<T, const N: usize>(input: *mut T) -> *mut [T; N] {
    input.cast()
}

fn bytes_to_chars(input: *const u8) -> *const c_char {
    input.cast()
}

fn write_to_ptr<T: 'static>(ptr: *mut T, value: T) {
    if let Some(m) = unsafe { ptr.as_mut() } {
        *m = value;
    }
}

fn write_default_to_ptr<T: Default + 'static>(ptr: *mut T) {
    write_to_ptr(ptr, Default::default())
}

#[derive(Debug, Default)]
struct BytesCache(Vec<u8>);

impl BytesCache {
    fn set_string(&mut self, string: &str, value: *mut *const c_char, len: *mut usize) {
        self.0.clear();
        self.0.extend(string.as_bytes());
        self.0.push(0);
        write_to_ptr(value, bytes_to_chars(self.0.as_ptr()));
        write_to_ptr(len, self.0.len() - 1);
    }
}

pub(crate) fn next_simple<T: 'static>(
    iter: &mut impl Iterator<Item = Result<T, omf::error::Error>>,
    value: *mut T,
) -> Result<bool, Error> {
    match iter.next() {
        Some(Ok(v)) => {
            write_to_ptr(value, v);
            Ok(true)
        }
        Some(Err(err)) => Err(err.into()),
        None => Ok(false),
    }
}

pub(crate) fn next_option<T: Default + 'static>(
    iter: &mut impl Iterator<Item = Result<Option<T>, omf::error::Error>>,
    value: *mut T,
    is_null: *mut bool,
) -> Result<bool, Error> {
    match iter.next() {
        Some(Ok(Some(v))) => {
            write_to_ptr(value, v);
            write_to_ptr(is_null, false);
            Ok(true)
        }
        Some(Ok(None)) => {
            write_default_to_ptr(value);
            write_to_ptr(is_null, true);
            Ok(true)
        }
        Some(Err(err)) => Err(err.into()),
        None => Ok(false),
    }
}

pub(crate) fn next_option_convert<T: 'static, U: Default + 'static>(
    iter: &mut impl Iterator<Item = Result<Option<T>, omf::error::Error>>,
    value: *mut U,
    is_null: *mut bool,
    convert: impl FnOnce(T) -> U,
) -> Result<bool, Error> {
    match iter.next() {
        Some(Ok(Some(v))) => {
            write_to_ptr(value, convert(v));
            write_to_ptr(is_null, false);
            Ok(true)
        }
        Some(Ok(None)) => {
            write_default_to_ptr(value);
            write_to_ptr(is_null, true);
            Ok(true)
        }
        Some(Err(err)) => Err(err.into()),
        None => Ok(false),
    }
}

pub(crate) fn next_wide<T: 'static, const N: usize>(
    iter: &mut impl Iterator<Item = Result<[T; N], omf::error::Error>>,
    value: *mut T,
) -> Result<bool, Error> {
    next_simple(iter, width_cast(value))
}

pub(crate) fn next_wide_option<T: 'static, const N: usize>(
    iter: &mut impl Iterator<Item = Result<Option<[T; N]>, omf::error::Error>>,
    value: *mut T,
    is_null: *mut bool,
) -> Result<bool, Error>
where
    [T; N]: Default,
{
    next_option(iter, width_cast(value), is_null)
}

pub(crate) fn next_boundary<T: omf::data::NumberType>(
    iter: &mut impl Iterator<Item = Result<omf::data::Boundary<T>, omf::error::Error>>,
    value: *mut T,
    inclusive: *mut bool,
) -> Result<bool, Error> {
    match iter.next() {
        Some(Ok(boundary)) => {
            write_to_ptr(value, boundary.value());
            write_to_ptr(inclusive, boundary.is_inclusive());
            Ok(true)
        }
        Some(Err(err)) => Err(err.into()),
        None => Ok(false),
    }
}

// f32 scalars.

pub struct Scalars32(omf::data::GenericScalars<f32, File>);
pub(crate) fn scalars32_new(iter: omf::data::GenericScalars<f32, File>) -> *mut Scalars32 {
    alloc(Scalars32(iter))
}

#[no_mangle]
pub extern "C" fn omf_scalars32_free(iter: *mut Scalars32) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_scalars32_next(iter: *mut Scalars32, value: *mut f32) -> bool {
    catch::error_only(|| next_simple(inner!(iter), value)).unwrap_or(false)
}

// f64 scalars, can cast from f32.

pub struct Scalars64(omf::data::Scalars<File>);

pub(crate) fn scalars64_new(iter: omf::data::Scalars<File>) -> *mut Scalars64 {
    alloc(Scalars64(iter))
}

#[no_mangle]
pub extern "C" fn omf_scalars64_free(iter: *mut Scalars64) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_scalars64_next(iter: *mut Scalars64, value: *mut f64) -> bool {
    catch::error_only(|| next_simple(inner!(iter), value)).unwrap_or(false)
}

// f32 vertices.

pub struct Vertices32(omf::data::GenericArrays<f32, 3, File>);

pub(crate) fn vertices32_new(iter: omf::data::GenericArrays<f32, 3, File>) -> *mut Vertices32 {
    alloc(Vertices32(iter))
}

#[no_mangle]
pub extern "C" fn omf_vertices32_free(iter: *mut Vertices32) {
    free(iter);
}
#[no_mangle]
pub extern "C" fn omf_vertices32_next(iter: *mut Vertices32, value: *mut f32) -> bool {
    catch::error_only(|| next_wide(inner!(iter), value)).unwrap_or(false)
}

// f64 vertices, can cast from f32.

pub struct Vertices64(omf::data::Vertices<File>);

pub(crate) fn vertices64_new(iter: omf::data::Vertices<File>) -> *mut Vertices64 {
    alloc(Vertices64(iter))
}

#[no_mangle]
pub extern "C" fn omf_vertices64_free(iter: *mut Vertices64) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_vertices64_next(iter: *mut Vertices64, value: *mut f64) -> bool {
    catch::error_only(|| next_wide(inner!(iter), value)).unwrap_or(false)
}

// Segments.

pub struct Segments(omf::data::GenericPrimitives<2, File>);

pub(crate) fn segments_new(iter: omf::data::GenericPrimitives<2, File>) -> *mut Segments {
    alloc(Segments(iter))
}

#[no_mangle]
pub extern "C" fn omf_segments_free(iter: *mut Segments) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_segments_next(iter: *mut Segments, value: *mut u32) -> bool {
    catch::error_only(|| next_wide(inner!(iter), value)).unwrap_or(false)
}

// Triangles.

pub struct Triangles(omf::data::GenericPrimitives<3, File>);
pub(crate) fn triangles_new(iter: omf::data::GenericPrimitives<3, File>) -> *mut Triangles {
    alloc(Triangles(iter))
}

#[no_mangle]
pub extern "C" fn omf_triangles_free(iter: *mut Triangles) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_triangles_next(iter: *mut Triangles, value: *mut u32) -> bool {
    catch::error_only(|| next_wide(inner!(iter), value)).unwrap_or(false)
}

// Gradient.

pub struct Gradient(omf::data::Gradient<File>);
pub(crate) fn gradient_new(iter: omf::data::Gradient<File>) -> *mut Gradient {
    alloc(Gradient(iter))
}

#[no_mangle]
pub extern "C" fn omf_gradient_free(iter: *mut Gradient) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_gradient_next(iter: *mut Gradient, value: *mut u8) -> bool {
    catch::error_only(|| next_wide(inner!(iter), value)).unwrap_or(false)
}

// f32 texture coordinates.

pub struct Texcoords32(omf::data::GenericArrays<f32, 2, File>);

pub(crate) fn texcoords32_new(iter: omf::data::GenericArrays<f32, 2, File>) -> *mut Texcoords32 {
    alloc(Texcoords32(iter))
}

#[no_mangle]
pub extern "C" fn omf_texcoords32_free(iter: *mut Texcoords32) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_texcoords32_next(iter: *mut Texcoords32, value: *mut f32) -> bool {
    catch::error_only(|| next_wide(inner!(iter), value)).unwrap_or(false)
}

// f64 texcoords, can cast from f32.

pub struct Texcoords64(omf::data::Texcoords<File>);

pub(crate) fn texcoords64_new(iter: omf::data::Texcoords<File>) -> *mut Texcoords64 {
    alloc(Texcoords64(iter))
}

#[no_mangle]
pub extern "C" fn omf_texcoords64_free(iter: *mut Texcoords64) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_texcoords64_next(iter: *mut Texcoords64, value: *mut f64) -> bool {
    catch::error_only(|| next_wide(inner!(iter), value)).unwrap_or(false)
}

// f64 discrete colormap boundaries.

pub struct BoundariesFloat64(omf::data::BoundariesF64<File>);

pub(crate) fn boundaries_float64_new(
    iter: omf::data::BoundariesF64<File>,
) -> *mut BoundariesFloat64 {
    alloc(BoundariesFloat64(iter))
}

#[no_mangle]
pub extern "C" fn omf_boundaries_float64_free(iter: *mut BoundariesFloat64) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_boundaries_float64_next(
    iter: *mut BoundariesFloat64,
    value: *mut f64,
    inclusive: *mut bool,
) -> bool {
    catch::error_only(|| next_boundary(inner!(iter), value, inclusive)).unwrap_or(false)
}

// f32 discrete colormap boundaries.

pub struct BoundariesFloat32(omf::data::GenericBoundaries<f32, File>);

pub(crate) fn boundaries_float32_new(
    iter: omf::data::GenericBoundaries<f32, File>,
) -> *mut BoundariesFloat32 {
    alloc(BoundariesFloat32(iter))
}

#[no_mangle]
pub extern "C" fn omf_boundaries_float32_free(iter: *mut BoundariesFloat32) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_boundaries_float32_next(
    iter: *mut BoundariesFloat32,
    value: *mut f32,
    inclusive: *mut bool,
) -> bool {
    catch::error_only(|| next_boundary(inner!(iter), value, inclusive)).unwrap_or(false)
}

// i64 discrete colormap boundaries.

pub struct BoundariesInt64(omf::data::BoundariesI64<File>);

pub(crate) fn boundaries_int64_new(iter: omf::data::BoundariesI64<File>) -> *mut BoundariesInt64 {
    alloc(BoundariesInt64(iter))
}

#[no_mangle]
pub extern "C" fn omf_boundaries_int64_free(iter: *mut BoundariesInt64) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_boundaries_int64_next(
    iter: *mut BoundariesInt64,
    value: *mut i64,
    inclusive: *mut bool,
) -> bool {
    catch::error_only(|| next_boundary(inner!(iter), value, inclusive)).unwrap_or(false)
}

// f32 numbers.

pub struct NumbersFloat32(omf::data::GenericNumbers<f32, File>);

pub(crate) fn numbers_float32_new(
    iter: omf::data::GenericNumbers<f32, File>,
) -> *mut NumbersFloat32 {
    alloc(NumbersFloat32(iter))
}

#[no_mangle]
pub extern "C" fn omf_numbers_float32_free(iter: *mut NumbersFloat32) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_numbers_float32_next(
    iter: *mut NumbersFloat32,
    value: *mut f32,
    is_null: *mut bool,
) -> bool {
    catch::error_only(|| next_option(inner!(iter), value, is_null)).unwrap_or(false)
}

// f64 numbers, casting from date and date-time as well.

pub struct NumbersFloat64(omf::data::NumbersF64<File>);

pub(crate) fn numbers_float64_new(iter: omf::data::NumbersF64<File>) -> *mut NumbersFloat64 {
    alloc(NumbersFloat64(iter))
}

#[no_mangle]
pub extern "C" fn omf_numbers_float64_free(iter: *mut NumbersFloat64) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_numbers_float64_next(
    iter: *mut NumbersFloat64,
    value: *mut f64,
    is_null: *mut bool,
) -> bool {
    catch::error_only(|| next_option(inner!(iter), value, is_null)).unwrap_or(false)
}

// i64 numbers, casting from date and date-time as well.

pub struct NumbersInt64(omf::data::NumbersI64<File>);

pub(crate) fn numbers_int64_new(iter: omf::data::NumbersI64<File>) -> *mut NumbersInt64 {
    alloc(NumbersInt64(iter))
}

#[no_mangle]
pub extern "C" fn omf_numbers_int64_free(iter: *mut NumbersInt64) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_numbers_int64_next(
    iter: *mut NumbersInt64,
    value: *mut i64,
    is_null: *mut bool,
) -> bool {
    catch::error_only(|| next_option(inner!(iter), value, is_null)).unwrap_or(false)
}

// Nullable indices.

pub struct Indices(omf::data::Indices<File>);

pub(crate) fn indices_new(iter: omf::data::Indices<File>) -> *mut Indices {
    alloc(Indices(iter))
}

#[no_mangle]
pub extern "C" fn omf_indices_free(iter: *mut Indices) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_indices_next(
    iter: *mut Indices,
    value: *mut u32,
    is_null: *mut bool,
) -> bool {
    catch::error_only(|| next_option(inner!(iter), value, is_null)).unwrap_or(false)
}

// Nullable booleans.

pub struct Booleans(omf::data::Booleans<File>);

pub(crate) fn booleans_new(iter: omf::data::Booleans<File>) -> *mut Booleans {
    alloc(Booleans(iter))
}

#[no_mangle]
pub extern "C" fn omf_booleans_free(iter: *mut Booleans) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_booleans_next(
    iter: *mut Booleans,
    value: *mut bool,
    is_null: *mut bool,
) -> bool {
    catch::error_only(|| next_option(inner!(iter), value, is_null)).unwrap_or(false)
}

// Nullable colors.

pub struct Colors(omf::data::Colors<File>);
pub(crate) fn colors_new(iter: omf::data::Colors<File>) -> *mut Colors {
    alloc(Colors(iter))
}

#[no_mangle]
pub extern "C" fn omf_colors_free(iter: *mut Colors) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_omf_colors_next(
    iter: *mut Colors,
    value: *mut u8,
    is_null: *mut bool,
) -> bool {
    catch::error_only(|| next_wide_option(inner!(iter), value, is_null)).unwrap_or(false)
}

// Non-nullable name strings.

pub struct Names {
    iter: omf::data::Names<File>,
    bytes: BytesCache,
}

pub(crate) fn names_new(iter: omf::data::Names<File>) -> *mut Names {
    Box::into_raw(Box::new(Names {
        iter,
        bytes: Default::default(),
    }))
}

#[no_mangle]
pub extern "C" fn omf_names_free(iter: *mut Names) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_names_next(
    iter: *mut Names,
    value: *mut *const c_char,
    len: *mut usize,
) -> bool {
    catch::error_only(|| {
        let this = not_null_mut!(iter)?;
        match this.iter.next() {
            Some(Ok(s)) => {
                this.bytes.set_string(&s, value, len);
                Ok(true)
            }
            Some(Err(err)) => {
                set_error(err.into());
                Ok(false)
            }
            None => Ok(false),
        }
    })
    .unwrap_or(false)
}

// Nullable text strings.

pub struct Text {
    iter: omf::data::Text<File>,
    bytes: BytesCache,
}

pub(crate) fn text_new(iter: omf::data::Text<File>) -> *mut Text {
    Box::into_raw(Box::new(Text {
        iter,
        bytes: Default::default(),
    }))
}

#[no_mangle]
pub extern "C" fn omf_text_free(iter: *mut Text) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_text_next(
    iter: *mut Text,
    value: *mut *const c_char,
    len: *mut usize,
) -> bool {
    catch::error_only(|| {
        let this = not_null_mut!(iter)?;
        match this.iter.next() {
            Some(Ok(Some(s))) => {
                this.bytes.set_string(&s, value, len);
                Ok(true)
            }
            Some(Ok(None)) => {
                write_to_ptr(value, null());
                write_to_ptr(len, 0);
                Ok(true)
            }
            Some(Err(err)) => {
                set_error(err.into());
                Ok(false)
            }
            None => Ok(false),
        }
    })
    .unwrap_or(false)
}

// Regular sub-blocks

pub struct RegularSubblocks(omf::data::RegularSubblocks<File>);

pub(crate) fn regular_subblocks_new(
    iter: omf::data::RegularSubblocks<File>,
) -> *mut RegularSubblocks {
    alloc(RegularSubblocks(iter))
}

#[no_mangle]
pub extern "C" fn omf_regular_subblocks_free(iter: *mut RegularSubblocks) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_regular_subblocks_next(
    iter: *mut RegularSubblocks,
    parent_index: *mut u32,
    corners: *mut u32,
) -> bool {
    catch::error_only(|| {
        let this = not_null_mut!(iter)?;
        match this.0.next() {
            Some(Ok((p, c))) => {
                write_to_ptr(width_cast(parent_index), p);
                write_to_ptr(width_cast(corners), c);
                Ok(true)
            }
            Some(Err(err)) => {
                set_error(err.into());
                Ok(false)
            }
            None => Ok(false),
        }
    })
    .unwrap_or(false)
}

// Free-form sub-blocks with f64 corners, casts from f64

pub struct FreeformSubblocks32(omf::data::GenericFreeformSubblocks<f32, File>);

pub(crate) fn freeform_subblocks32_new(
    iter: omf::data::GenericFreeformSubblocks<f32, File>,
) -> *mut FreeformSubblocks32 {
    alloc(FreeformSubblocks32(iter))
}

#[no_mangle]
pub extern "C" fn omf_freeform_subblocks32_free(iter: *mut FreeformSubblocks32) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_freeform_subblocks32_next(
    iter: *mut FreeformSubblocks32,
    parent_index: *mut u32,
    corners: *mut f32,
) -> bool {
    catch::error_only(|| {
        let this = not_null_mut!(iter)?;
        match this.0.next() {
            Some(Ok((p, c))) => {
                write_to_ptr(width_cast(parent_index), p);
                write_to_ptr(width_cast(corners), c);
                Ok(true)
            }
            Some(Err(err)) => {
                set_error(err.into());
                Ok(false)
            }
            None => Ok(false),
        }
    })
    .unwrap_or(false)
}

// Free-form sub-blocks with f32 corners

pub struct FreeformSubblocks64(omf::data::FreeformSubblocks<File>);

pub(crate) fn freeform_subblocks64_new(
    iter: omf::data::FreeformSubblocks<File>,
) -> *mut FreeformSubblocks64 {
    alloc(FreeformSubblocks64(iter))
}

#[no_mangle]
pub extern "C" fn omf_freeform_subblocks64_free(iter: *mut FreeformSubblocks64) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_freeform_subblocks64_next(
    iter: *mut FreeformSubblocks64,
    parent_index: *mut u32,
    corners: *mut f64,
) -> bool {
    catch::error_only(|| {
        let this = not_null_mut!(iter)?;
        match this.0.next() {
            Some(Ok((p, c))) => {
                write_to_ptr(width_cast(parent_index), p);
                write_to_ptr(width_cast(corners), c);
                Ok(true)
            }
            Some(Err(err)) => {
                set_error(err.into());
                Ok(false)
            }
            None => Ok(false),
        }
    })
    .unwrap_or(false)
}

// 3D vectors with type f64, casts from anything

pub struct Vectors64x3(omf::data::Vectors<File>);

pub(crate) fn vectors64x3_new(iter: omf::data::Vectors<File>) -> *mut Vectors64x3 {
    alloc(Vectors64x3(iter))
}

#[no_mangle]
pub extern "C" fn omf_vectors64x3_free(iter: *mut Vectors64x3) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_vectors64x3_next(
    iter: *mut Vectors64x3,
    value: *mut f64,
    is_null: *mut bool,
) -> bool {
    catch::error_only(|| next_wide_option(inner!(iter), value, is_null)).unwrap_or(false)
}

// 3D vectors with type f32, casts from 2D f32

pub struct Vectors32x3(Vec32Iter);

enum Vec32Iter {
    X2(omf::data::GenericOptionalArrays<f32, 2, File>),
    X3(omf::data::GenericOptionalArrays<f32, 3, File>),
}

pub(crate) fn vectors32x3_new(iter: omf::data::Vectors<File>) -> *mut Vectors32x3 {
    let vec32_iter = match iter {
        omf::data::Vectors::F32x2(i) => Vec32Iter::X2(i),
        omf::data::Vectors::F32x3(i) => Vec32Iter::X3(i),
        _ => panic!("wrong vector type"),
    };
    alloc(Vectors32x3(vec32_iter))
}

#[no_mangle]
pub extern "C" fn omf_vectors32x3_free(iter: *mut Vectors32x3) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_vectors32x3_next(
    iter: *mut Vectors32x3,
    value: *mut f32,
    is_null: *mut bool,
) -> bool {
    catch::error_only(|| {
        let iter = inner!(iter);
        match iter {
            Vec32Iter::X2(i) => next_option_convert(i, width_cast(value), is_null, vec_2d_to_3d),
            Vec32Iter::X3(i) => next_wide_option(i, value, is_null),
        }
    })
    .unwrap_or(false)
}

fn vec_2d_to_3d<T: omf::data::FloatType>([x, y]: [T; 2]) -> [T; 3] {
    [x, y, T::ZERO]
}

// 2D vectors with type f64, casts from 2D f32

pub struct Vectors64x2(Vec2DIter);

enum Vec2DIter {
    F32(omf::data::GenericOptionalArrays<f32, 2, File>),
    F64(omf::data::GenericOptionalArrays<f64, 2, File>),
}

pub(crate) fn vectors64x2_new(iter: omf::data::Vectors<File>) -> *mut Vectors64x2 {
    let vec32_iter = match iter {
        omf::data::Vectors::F32x2(i) => Vec2DIter::F32(i),
        omf::data::Vectors::F64x2(i) => Vec2DIter::F64(i),
        _ => panic!("wrong vector type"),
    };
    alloc(Vectors64x2(vec32_iter))
}

#[no_mangle]
pub extern "C" fn omf_vectors64x2_free(iter: *mut Vectors64x2) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_vectors64x2_next(
    iter: *mut Vectors64x2,
    value: *mut f64,
    is_null: *mut bool,
) -> bool {
    catch::error_only(|| {
        let iter = inner!(iter);
        match iter {
            Vec2DIter::F32(i) => {
                next_option_convert(i, width_cast(value), is_null, |[x, y]| [x.into(), y.into()])
            }
            Vec2DIter::F64(i) => next_wide_option(i, value, is_null),
        }
    })
    .unwrap_or(false)
}

// 2D vectors with type f32, no casting

pub struct Vectors32x2(omf::data::GenericOptionalArrays<f32, 2, File>);

pub(crate) fn vectors32x2_new(
    iter: omf::data::GenericOptionalArrays<f32, 2, File>,
) -> *mut Vectors32x2 {
    alloc(Vectors32x2(iter))
}

#[no_mangle]
pub extern "C" fn omf_vectors32x2_free(iter: *mut Vectors32x2) {
    free(iter);
}

#[no_mangle]
pub extern "C" fn omf_vectors32x2_next(
    iter: *mut Vectors32x2,
    value: *mut f32,
    is_null: *mut bool,
) -> bool {
    catch::error_only(|| next_wide_option(inner!(iter), value, is_null)).unwrap_or(false)
}
