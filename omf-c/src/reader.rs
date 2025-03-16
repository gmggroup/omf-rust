use std::{ffi::c_char, fs::File, io::Read, path::PathBuf, ptr::null_mut, sync::Mutex};

use crate::{
    arrays::{array, array_action, Array, ArrayType},
    elements::{FileVersion, Limits, Project},
    error::Error,
    ffi_tools::{
        arg::{not_null, not_null_consume, slice_mut, slice_mut_len, string_not_null},
        catch, FfiStorage, IntoFfi,
    },
    image_data::ImageData,
    read_iterators::*,
    validation::{handle_validation, Validation},
};

struct ReaderWrapper {
    pub inner: omf::file::Reader<File>,
    pub storage: FfiStorage,
    pub project_loaded: bool,
}

macro_rules! unsafe_cast {
    ($from:literal -> $to:literal) => {
        Err(omf::error::Error::UnsafeCast($from, $to).into())
    };
}

pub struct Reader(Mutex<ReaderWrapper>);

impl ReaderWrapper {
    fn array_type(&self, array: &Array) -> Result<ArrayType, Error> {
        Ok(match array {
            Array::Image(_) => ArrayType::Image,
            Array::Name(_) => ArrayType::Names,
            Array::Text(_) => ArrayType::Text,
            Array::Boolean(_) => ArrayType::Booleans,
            Array::Segment(_) => ArrayType::Segments,
            Array::Triangle(_) => ArrayType::Triangles,
            Array::Gradient(_) => ArrayType::Gradient,
            Array::RegularSubblock(_) => ArrayType::RegularSubblocks,
            Array::Index(_) => ArrayType::Indices,
            Array::Color(_) => ArrayType::Colors,
            Array::Scalar(a) => match self.inner.array_scalars(a)? {
                omf::data::Scalars::F32(_) => ArrayType::Scalars32,
                omf::data::Scalars::F64(_) => ArrayType::Scalars64,
            },
            Array::Vertex(a) => match self.inner.array_vertices(a)? {
                omf::data::Vertices::F32(_) => ArrayType::Vertices32,
                omf::data::Vertices::F64(_) => ArrayType::Vertices64,
            },
            Array::Texcoord(a) => match self.inner.array_texcoords(a)? {
                omf::data::Texcoords::F32(_) => ArrayType::Texcoords32,
                omf::data::Texcoords::F64(_) => ArrayType::Texcoords64,
            },
            Array::Boundary(a) => match self.inner.array_boundaries(a)? {
                omf::data::Boundaries::F32(_) => ArrayType::BoundariesFloat32,
                omf::data::Boundaries::F64(_) => ArrayType::BoundariesFloat64,
                omf::data::Boundaries::I64(_) => ArrayType::BoundariesInt64,
                omf::data::Boundaries::Date(_) => ArrayType::BoundariesDate,
                omf::data::Boundaries::DateTime(_) => ArrayType::BoundariesDateTime,
            },
            Array::FreeformSubblock(a) => match self.inner.array_freeform_subblocks(a)? {
                omf::data::FreeformSubblocks::F32(_) => ArrayType::FreeformSubblocks32,
                omf::data::FreeformSubblocks::F64(_) => ArrayType::FreeformSubblocks64,
            },
            Array::Number(a) => match self.inner.array_numbers(a)? {
                omf::data::Numbers::F32(_) => ArrayType::NumbersFloat32,
                omf::data::Numbers::F64(_) => ArrayType::NumbersFloat64,
                omf::data::Numbers::I64(_) => ArrayType::NumbersInt64,
                omf::data::Numbers::Date(_) => ArrayType::NumbersDate,
                omf::data::Numbers::DateTime(_) => ArrayType::NumbersDateTime,
            },
            Array::Vector(a) => match self.inner.array_vectors(a)? {
                omf::data::Vectors::F32x2(_) => ArrayType::Vectors32x2,
                omf::data::Vectors::F64x2(_) => ArrayType::Vectors64x2,
                omf::data::Vectors::F32x3(_) => ArrayType::Vectors32x3,
                omf::data::Vectors::F64x3(_) => ArrayType::Vectors64x3,
            },
        })
    }

    fn array_info(&self, array: &Array) -> Result<ArrayInfo, Error> {
        let array_type = self.array_type(array)?;
        array_action!(array, |a| {
            Ok(ArrayInfo {
                array_type,
                item_count: a.item_count(),
                compressed_size: self.inner.array_compressed_size(a)?,
            })
        })
    }
}

#[no_mangle]
pub extern "C" fn omf_reader_open(path: *const c_char) -> *mut Reader {
    catch::error(|| {
        let path = PathBuf::from(string_not_null!(path)?);
        let wrapper = ReaderWrapper {
            inner: omf::file::Reader::open(path)?,
            storage: FfiStorage::new(),
            project_loaded: false,
        };
        Ok(Box::into_raw(Box::new(Reader(Mutex::new(wrapper)))))
    })
    .unwrap_or(null_mut())
}

#[no_mangle]
pub extern "C" fn omf_reader_close(reader: *mut Reader) -> bool {
    catch::panic_bool(|| {
        _ = not_null_consume!(reader);
    })
}

macro_rules! wrapper {
    ($reader:ident) => {
        not_null!($reader)?.0.lock().expect("intact lock")
    };
}

#[no_mangle]
pub extern "C" fn omf_reader_project(
    reader: *mut Reader,
    validation: *mut *mut Validation,
) -> *const Project {
    catch::error(|| {
        let mut wrapper = wrapper!(reader);
        if wrapper.project_loaded {
            return Err(Error::InvalidCall(
                "second call to 'omf_reader_project' on this reader".to_owned(),
            ));
        }
        let result = wrapper.inner.project();
        match &result {
            Ok((_, warnings)) => handle_validation(warnings, validation),
            Err(omf::error::Error::ValidationFailed(errors)) => {
                handle_validation(errors, validation)
            }
            _ => {}
        }
        let (project, _) = result?;
        wrapper.project_loaded = true;
        Ok(wrapper.storage.convert_ptr_mut(project))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_version(reader: *mut Reader) -> FileVersion {
    catch::error(|| {
        let wrapper = wrapper!(reader);
        Ok(wrapper.inner.version().into())
    })
    .unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn omf_reader_limits(reader: *mut Reader) -> Limits {
    catch::error(|| {
        let limits = if reader.is_null() {
            Default::default()
        } else {
            wrapper!(reader).inner.limits()
        };
        Ok(limits.into())
    })
    .unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn omf_reader_set_limits(reader: *mut Reader, limits: *const Limits) -> bool {
    catch::error(|| {
        let limits = not_null!(limits)?;
        wrapper!(reader).inner.set_limits((*limits).into());
        Ok(true)
    })
    .unwrap_or(false)
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct ArrayInfo {
    pub array_type: ArrayType,
    pub item_count: u64,
    pub compressed_size: u64,
}

#[no_mangle]
pub extern "C" fn omf_reader_array_info(reader: *mut Reader, array: *const Array) -> ArrayInfo {
    catch::error(|| {
        let array = not_null!(array)?;
        wrapper!(reader).array_info(array)
    })
    .unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn omf_reader_array_bytes(
    reader: *mut Reader,
    array: *const Array,
    output: *mut c_char,
    n_output: usize,
) -> bool {
    catch::error(|| {
        let wrapper = wrapper!(reader);
        let mut read = array_action!(not_null!(array)?, |a| wrapper.inner.array_bytes_reader(a))?;
        let found = u64::try_from(n_output).expect("usize fits in u64");
        if read.len() != found {
            return Err(Error::BufferLengthWrong {
                found,
                expected: read.len(),
            });
        }
        read.read_exact(slice_mut!(output.cast(), n_output)?)
            .map_err(omf::error::Error::from)?;
        Ok(true)
    })
    .unwrap_or(false)
}

// Image

#[no_mangle]
pub extern "C" fn omf_reader_image(reader: *mut Reader, array: *const Array) -> *mut ImageData {
    catch::error(|| Ok(wrapper!(reader).inner.image(&array!(array)?)?.into_ffi()))
        .unwrap_or_else(null_mut)
}

// Scalars

#[no_mangle]
pub extern "C" fn omf_reader_array_scalars32_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Scalars32 {
    catch::error(
        || match wrapper!(reader).inner.array_scalars(&array!(array)?)? {
            omf::data::Scalars::F32(i) => Ok(scalars32_new(i)),
            omf::data::Scalars::F64(_) => unsafe_cast!("64-bit float" -> "32-bit float"),
        },
    )
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_scalars64_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Scalars64 {
    catch::error(|| {
        Ok(scalars64_new(
            wrapper!(reader).inner.array_scalars(&array!(array)?)?,
        ))
    })
    .unwrap_or_else(null_mut)
}

fn check_slice(n_values: usize, expected: u64) -> Result<(), Error> {
    let found = u64::try_from(n_values).expect("usize fits in u64");
    if found != expected {
        return Err(Error::BufferLengthWrong { found, expected });
    }
    Ok(())
}

fn into_slice<T: 'static>(
    values: *mut T,
    n_values: usize,
    expected: u64,
    iter: impl IntoIterator<Item = Result<T, omf::error::Error>>,
) -> Result<(), Error> {
    check_slice(n_values, expected)?;
    for (out, val) in slice_mut!(values, n_values)?.iter_mut().zip(iter) {
        *out = val?;
    }
    Ok(())
}

fn into_slice_nullable<T: Default + 'static>(
    values: *mut T,
    mask: *mut bool,
    n_values: usize,
    expected: u64,
    iter: impl IntoIterator<Item = Result<Option<T>, omf::error::Error>>,
) -> Result<(), Error> {
    check_slice(n_values, expected)?;
    for ((out, is_null), val) in slice_mut!(values, n_values)?
        .iter_mut()
        .zip(slice_mut!(mask, n_values)?)
        .zip(iter)
    {
        let value = val?;
        *is_null = value.is_none();
        *out = value.unwrap_or_default();
    }
    Ok(())
}

fn into_slice_nullable_convert<T, U: Default + 'static>(
    values: *mut U,
    mask: *mut bool,
    n_values: usize,
    expected: u64,
    iter: impl IntoIterator<Item = Result<Option<T>, omf::error::Error>>,
    f: impl Fn(T) -> U + Copy,
) -> Result<(), Error> {
    check_slice(n_values, expected)?;
    for ((out, is_null), val) in slice_mut!(values, n_values)?
        .iter_mut()
        .zip(slice_mut!(mask, n_values)?)
        .zip(iter)
    {
        let value = val?;
        *is_null = value.is_none();
        *out = value.map(f).unwrap_or_default();
    }
    Ok(())
}

#[no_mangle]
pub extern "C" fn omf_reader_array_scalars64(
    reader: *mut Reader,
    array: *const Array,
    values: *mut f64,
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = wrapper!(reader).inner.array_scalars(&array)?;
        into_slice(values, n_values, array.item_count(), iter)
    })
    .is_some()
}

#[no_mangle]
pub extern "C" fn omf_reader_array_scalars32(
    reader: *mut Reader,
    array: *const Array,
    values: *mut f32,
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = match wrapper!(reader).inner.array_scalars(&array)? {
            omf::data::Scalars::F32(i) => i,
            omf::data::Scalars::F64(_) => return unsafe_cast!("64-bit float" -> "32-bit float"),
        };
        into_slice(values, n_values, array.item_count(), iter)
    })
    .is_some()
}

// Vertices

#[no_mangle]
pub extern "C" fn omf_reader_array_vertices32_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Vertices32 {
    catch::error(
        || match wrapper!(reader).inner.array_vertices(&array!(array)?)? {
            omf::data::Vertices::F32(i) => Ok(vertices32_new(i)),
            omf::data::Vertices::F64(_) => unsafe_cast!("64-bit float" -> "32-bit float"),
        },
    )
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_vertices64_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Vertices64 {
    catch::error(|| {
        Ok(vertices64_new(
            wrapper!(reader).inner.array_vertices(&array!(array)?)?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_vertices64(
    reader: *mut Reader,
    array: *const Array,
    values: *mut [f64; 3],
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = wrapper!(reader).inner.array_vertices(&array)?;
        into_slice(values, n_values, array.item_count(), iter)
    })
    .is_some()
}

#[no_mangle]
pub extern "C" fn omf_reader_array_vertices32(
    reader: *mut Reader,
    array: *const Array,
    values: *mut [f32; 3],
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = match wrapper!(reader).inner.array_vertices(&array)? {
            omf::data::Vertices::F32(i) => i,
            omf::data::Vertices::F64(_) => return unsafe_cast!("64-bit float" -> "32-bit float"),
        };
        into_slice(values, n_values, array.item_count(), iter)
    })
    .is_some()
}

// Segments

#[no_mangle]
pub extern "C" fn omf_reader_array_segments_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Segments {
    catch::error(|| {
        Ok(segments_new(
            wrapper!(reader).inner.array_segments(&array!(array)?)?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_segments(
    reader: *mut Reader,
    array: *const Array,
    values: *mut [u32; 2],
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = wrapper!(reader).inner.array_segments(&array)?;
        into_slice(values, n_values, array.item_count(), iter)
    })
    .is_some()
}

// Triangles

#[no_mangle]
pub extern "C" fn omf_reader_array_triangles_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Triangles {
    catch::error(|| {
        Ok(triangles_new(
            wrapper!(reader).inner.array_triangles(&array!(array)?)?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_triangles(
    reader: *mut Reader,
    array: *const Array,
    values: *mut [u32; 3],
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = wrapper!(reader).inner.array_triangles(&array)?;
        into_slice(values, n_values, array.item_count(), iter)
    })
    .is_some()
}

// Names

#[no_mangle]
pub extern "C" fn omf_reader_array_names_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Names {
    catch::error(|| {
        Ok(names_new(
            wrapper!(reader).inner.array_names(&array!(array)?)?,
        ))
    })
    .unwrap_or_else(null_mut)
}

// Gradient

#[no_mangle]
pub extern "C" fn omf_reader_array_gradient_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Gradient {
    catch::error(|| {
        Ok(gradient_new(
            wrapper!(reader).inner.array_gradient(&array!(array)?)?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_gradient(
    reader: *mut Reader,
    array: *const Array,
    values: *mut [u8; 4],
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = wrapper!(reader).inner.array_gradient(&array)?;
        into_slice(values, n_values, array.item_count(), iter)
    })
    .is_some()
}

// Texture coordinates

#[no_mangle]
pub extern "C" fn omf_reader_array_texcoords32_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Texcoords32 {
    catch::error(
        || match wrapper!(reader).inner.array_texcoords(&array!(array)?)? {
            omf::data::Texcoords::F32(i) => Ok(texcoords32_new(i)),
            omf::data::Texcoords::F64(_) => unsafe_cast!("64-bit float" -> "32-bit float"),
        },
    )
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_texcoords64_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Texcoords64 {
    catch::error(|| {
        Ok(texcoords64_new(
            wrapper!(reader).inner.array_texcoords(&array!(array)?)?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_texcoords64(
    reader: *mut Reader,
    array: *const Array,
    values: *mut [f64; 2],
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = wrapper!(reader).inner.array_texcoords(&array)?;
        into_slice(values, n_values, array.item_count(), iter)
    })
    .is_some()
}

#[no_mangle]
pub extern "C" fn omf_reader_array_texcoords32(
    reader: *mut Reader,
    array: *const Array,
    values: *mut [f32; 2],
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = match wrapper!(reader).inner.array_texcoords(&array)? {
            omf::data::Texcoords::F32(i) => i,
            omf::data::Texcoords::F64(_) => return unsafe_cast!("64-bit float" -> "32-bit float"),
        };
        into_slice(values, n_values, array.item_count(), iter)
    })
    .is_some()
}

// Boundaries

#[no_mangle]
pub extern "C" fn omf_reader_array_boundaries_float32_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut BoundariesFloat32 {
    catch::error(
        || match wrapper!(reader).inner.array_boundaries(&array!(array)?)? {
            omf::data::Boundaries::F32(i) => Ok(boundaries_float32_new(i)),
            omf::data::Boundaries::F64(_) => unsafe_cast!("64-bit float" -> "32-bit float"),
            omf::data::Boundaries::I64(_) => unsafe_cast!("64-bit int" -> "32-bit float"),
            omf::data::Boundaries::Date(_) => unsafe_cast!("date" -> "32-bit float"),
            omf::data::Boundaries::DateTime(_) => unsafe_cast!("date-time" -> "32-bit float"),
        },
    )
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_boundaries_float64_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut BoundariesFloat64 {
    catch::error(|| {
        Ok(boundaries_float64_new(
            wrapper!(reader)
                .inner
                .array_boundaries(&array!(array)?)?
                .try_into_f64()?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_boundaries_int64_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut BoundariesInt64 {
    catch::error(|| {
        Ok(boundaries_int64_new(
            wrapper!(reader)
                .inner
                .array_boundaries(&array!(array)?)?
                .try_into_i64()?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_boundaries_float64(
    reader: *mut Reader,
    array: *const Array,
    values: *mut f64,
    inclusive: *mut bool,
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = wrapper!(reader)
            .inner
            .array_boundaries(&array)?
            .try_into_f64()?;
        let values = slice_mut_len!(values, n_values, array.item_count())?;
        let inclusive = slice_mut_len!(inclusive, n_values, array.item_count())?;
        for ((out, inc), val) in values.iter_mut().zip(inclusive.iter_mut()).zip(iter) {
            let bound = val?;
            *out = bound.value();
            *inc = bound.is_inclusive();
        }
        Ok(())
    })
    .is_some()
}

#[no_mangle]
pub extern "C" fn omf_reader_array_boundaries_int64(
    reader: *mut Reader,
    array: *const Array,
    values: *mut i64,
    inclusive: *mut bool,
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = wrapper!(reader)
            .inner
            .array_boundaries(&array)?
            .try_into_i64()?;
        let values = slice_mut_len!(values, n_values, array.item_count())?;
        let inclusive = slice_mut_len!(inclusive, n_values, array.item_count())?;
        for ((out, inc), val) in values.iter_mut().zip(inclusive.iter_mut()).zip(iter) {
            let bound = val?;
            *out = bound.value();
            *inc = bound.is_inclusive();
        }
        Ok(())
    })
    .is_some()
}

// Regular sub-blocks

#[no_mangle]
pub extern "C" fn omf_reader_array_regular_subblocks_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut RegularSubblocks {
    catch::error(|| {
        Ok(regular_subblocks_new(
            wrapper!(reader)
                .inner
                .array_regular_subblocks(&array!(array)?)?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_regular_subblocks(
    reader: *mut Reader,
    array: *const Array,
    parents: *mut [u32; 3],
    corners: *mut [u32; 6],
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = wrapper!(reader).inner.array_regular_subblocks(&array)?;
        let parents = slice_mut_len!(parents, n_values, array.item_count())?;
        let corners = slice_mut_len!(corners, n_values, array.item_count())?;
        for ((p, c), block) in parents.iter_mut().zip(corners.iter_mut()).zip(iter) {
            (*p, *c) = block?;
        }
        Ok(())
    })
    .is_some()
}

// Freeform sub-blocks

#[no_mangle]
pub extern "C" fn omf_reader_array_freeform_subblocks32_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut FreeformSubblocks32 {
    catch::error(|| {
        let wrapper = wrapper!(reader);
        match wrapper.inner.array_freeform_subblocks(&array!(array)?)? {
            omf::data::FreeformSubblocks::F32(i) => Ok(freeform_subblocks32_new(i)),
            omf::data::FreeformSubblocks::F64(_) => unsafe_cast!("64-bit float" -> "32-bit float"),
        }
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_freeform_subblocks64_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut FreeformSubblocks64 {
    catch::error(|| {
        Ok(freeform_subblocks64_new(
            wrapper!(reader)
                .inner
                .array_freeform_subblocks(&array!(array)?)?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_freeform_subblocks64(
    reader: *mut Reader,
    array: *const Array,
    parents: *mut [u32; 3],
    corners: *mut [f64; 6],
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = wrapper!(reader).inner.array_freeform_subblocks(&array)?;
        let parents = slice_mut_len!(parents, n_values, array.item_count())?;
        let corners = slice_mut_len!(corners, n_values, array.item_count())?;
        for ((p, c), block) in parents.iter_mut().zip(corners.iter_mut()).zip(iter) {
            (*p, *c) = block?;
        }
        Ok(())
    })
    .is_some()
}

#[no_mangle]
pub extern "C" fn omf_reader_array_freeform_subblocks32(
    reader: *mut Reader,
    array: *const Array,
    parents: *mut [u32; 3],
    corners: *mut [f32; 6],
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = match wrapper!(reader).inner.array_freeform_subblocks(&array)? {
            omf::data::FreeformSubblocks::F32(i) => i,
            omf::data::FreeformSubblocks::F64(_) => {
                return unsafe_cast!("64-bit float" -> "32-bit float")
            }
        };
        let parents = slice_mut_len!(parents, n_values, array.item_count())?;
        let corners = slice_mut_len!(corners, n_values, array.item_count())?;
        for ((p, c), block) in parents.iter_mut().zip(corners.iter_mut()).zip(iter) {
            (*p, *c) = block?;
        }
        Ok(())
    })
    .is_some()
}

// Numbers

#[no_mangle]
pub extern "C" fn omf_reader_array_numbers_float32_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut NumbersFloat32 {
    catch::error(
        || match wrapper!(reader).inner.array_numbers(&array!(array)?)? {
            omf::data::Numbers::F32(i) => Ok(numbers_float32_new(i)),
            omf::data::Numbers::F64(_) => unsafe_cast!("64-bit float" -> "32-bit float"),
            omf::data::Numbers::I64(_) => unsafe_cast!("64-bit int" -> "32-bit float"),
            omf::data::Numbers::Date(_) => unsafe_cast!("date" -> "32-bit float"),
            omf::data::Numbers::DateTime(_) => unsafe_cast!("date-time" -> "32-bit float"),
        },
    )
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_numbers_float64_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut NumbersFloat64 {
    catch::error(|| {
        Ok(numbers_float64_new(
            wrapper!(reader)
                .inner
                .array_numbers(&array!(array)?)?
                .try_into_f64()?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_numbers_int64_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut NumbersInt64 {
    catch::error(|| {
        Ok(numbers_int64_new(
            wrapper!(reader)
                .inner
                .array_numbers(&array!(array)?)?
                .try_into_i64()?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_numbers_float64(
    reader: *mut Reader,
    array: *const Array,
    values: *mut f64,
    mask: *mut bool,
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = wrapper!(reader)
            .inner
            .array_numbers(&array)?
            .try_into_f64()?;
        into_slice_nullable(values, mask, n_values, array.item_count(), iter)
    })
    .is_some()
}

#[no_mangle]
pub extern "C" fn omf_reader_array_numbers_float32(
    reader: *mut Reader,
    array: *const Array,
    values: *mut f32,
    mask: *mut bool,
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        match wrapper!(reader).inner.array_numbers(&array)? {
            omf::data::Numbers::F32(i) => {
                into_slice_nullable(values, mask, n_values, array.item_count(), i)
            }
            omf::data::Numbers::F64(_) => unsafe_cast!("64-bit float" -> "32-bit float"),
            omf::data::Numbers::I64(_) => unsafe_cast!("64-bit int" -> "32-bit float"),
            omf::data::Numbers::Date(_) => unsafe_cast!("date" -> "32-bit float"),
            omf::data::Numbers::DateTime(_) => unsafe_cast!("date-time" -> "32-bit float"),
        }
    })
    .is_some()
}

// Indices

#[no_mangle]
pub extern "C" fn omf_reader_array_indices_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Indices {
    catch::error(|| {
        Ok(indices_new(
            wrapper!(reader).inner.array_indices(&array!(array)?)?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_indices(
    reader: *mut Reader,
    array: *const Array,
    values: *mut u32,
    mask: *mut bool,
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = wrapper!(reader).inner.array_indices(&array)?;
        into_slice_nullable(values, mask, n_values, array.item_count(), iter)
    })
    .is_some()
}

// Vectors

#[no_mangle]
pub extern "C" fn omf_reader_array_vectors32x2_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Vectors32x2 {
    catch::error(
        || match wrapper!(reader).inner.array_vectors(&array!(array)?)? {
            omf::data::Vectors::F32x2(i) => Ok(vectors32x2_new(i)),
            omf::data::Vectors::F64x2(_) => unsafe_cast!("64-bit float" -> "32-bit float"),
            omf::data::Vectors::F32x3(_) | omf::data::Vectors::F64x3(_) => {
                unsafe_cast!("3D" -> "2D")
            }
        },
    )
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_vectors64x2_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Vectors64x2 {
    catch::error(|| {
        let iter = wrapper!(reader).inner.array_vectors(&array!(array)?)?;
        match &iter {
            omf::data::Vectors::F32x2(_) | omf::data::Vectors::F64x2(_) => {
                Ok(vectors64x2_new(iter))
            }
            omf::data::Vectors::F32x3(_) | omf::data::Vectors::F64x3(_) => {
                unsafe_cast!("3D" -> "2D")
            }
        }
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_vectors32x3_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Vectors32x3 {
    catch::error(|| {
        let iter = wrapper!(reader).inner.array_vectors(&array!(array)?)?;
        match &iter {
            omf::data::Vectors::F32x3(_) | omf::data::Vectors::F32x2(_) => {
                Ok(vectors32x3_new(iter))
            }
            omf::data::Vectors::F64x2(_) | omf::data::Vectors::F64x3(_) => {
                unsafe_cast!("64-bit float" -> "32-bit float")
            }
        }
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_vectors64x3_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Vectors64x3 {
    catch::error(|| {
        Ok(vectors64x3_new(
            wrapper!(reader).inner.array_vectors(&array!(array)?)?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_vectors32x2(
    reader: *mut Reader,
    array: *const Array,
    values: *mut [f32; 2],
    mask: *mut bool,
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        match wrapper!(reader).inner.array_vectors(&array)? {
            omf::data::Vectors::F32x2(i) => {
                into_slice_nullable(values, mask, n_values, array.item_count(), i)
            }
            omf::data::Vectors::F64x2(_) => unsafe_cast!("64-bit float" -> "32-bit float"),
            omf::data::Vectors::F32x3(_) | omf::data::Vectors::F64x3(_) => {
                unsafe_cast!("3D" -> "2D")
            }
        }
    })
    .is_some()
}

#[no_mangle]
pub extern "C" fn omf_reader_array_vectors64x2(
    reader: *mut Reader,
    array: *const Array,
    values: *mut [f64; 2],
    mask: *mut bool,
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let c = array.item_count();
        match wrapper!(reader).inner.array_vectors(&array)? {
            omf::data::Vectors::F32x2(i) => {
                into_slice_nullable_convert(values, mask, n_values, c, i, |[x, y]| {
                    [x as f64, y as f64]
                })
            }
            omf::data::Vectors::F64x2(i) => {
                into_slice_nullable(values, mask, n_values, array.item_count(), i)
            }
            omf::data::Vectors::F32x3(_) | omf::data::Vectors::F64x3(_) => {
                unsafe_cast!("3D" -> "2D")
            }
        }
    })
    .is_some()
}

#[no_mangle]
pub extern "C" fn omf_reader_array_vectors32x3(
    reader: *mut Reader,
    array: *const Array,
    values: *mut [f32; 3],
    mask: *mut bool,
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let c = array.item_count();
        match wrapper!(reader).inner.array_vectors(&array)? {
            omf::data::Vectors::F32x2(i) => {
                into_slice_nullable_convert(values, mask, n_values, c, i, |[x, y]| [x, y, 0.0])
            }
            omf::data::Vectors::F32x3(i) => into_slice_nullable(values, mask, n_values, c, i),
            omf::data::Vectors::F64x2(_) | omf::data::Vectors::F64x3(_) => {
                unsafe_cast!("64-bit float" -> "32-bit float")
            }
        }
    })
    .is_some()
}

#[no_mangle]
pub extern "C" fn omf_reader_array_vectors64x3(
    reader: *mut Reader,
    array: *const Array,
    values: *mut [f64; 3],
    mask: *mut bool,
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let c = array.item_count();
        match wrapper!(reader).inner.array_vectors(&array)? {
            omf::data::Vectors::F32x2(i) => {
                into_slice_nullable_convert(values, mask, n_values, c, i, |[x, y]| {
                    [x as f64, y as f64, 0.0]
                })
            }
            omf::data::Vectors::F64x2(i) => {
                into_slice_nullable_convert(values, mask, n_values, c, i, |[x, y]| [x, y, 0.0])
            }
            omf::data::Vectors::F32x3(i) => {
                into_slice_nullable_convert(values, mask, n_values, c, i, |[x, y, z]| {
                    [x as f64, y as f64, z as f64]
                })
            }
            omf::data::Vectors::F64x3(i) => into_slice_nullable(values, mask, n_values, c, i),
        }
    })
    .is_some()
}

// Text

#[no_mangle]
pub extern "C" fn omf_reader_array_text_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Text {
    catch::error(|| {
        Ok(text_new(
            wrapper!(reader).inner.array_text(&array!(array)?)?,
        ))
    })
    .unwrap_or_else(null_mut)
}

// Booleans

#[no_mangle]
pub extern "C" fn omf_reader_array_booleans_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Booleans {
    catch::error(|| {
        Ok(booleans_new(
            wrapper!(reader).inner.array_booleans(&array!(array)?)?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_booleans(
    reader: *mut Reader,
    array: *const Array,
    values: *mut bool,
    mask: *mut bool,
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = wrapper!(reader).inner.array_booleans(&array)?;
        into_slice_nullable(values, mask, n_values, array.item_count(), iter)
    })
    .is_some()
}

// Colors

#[no_mangle]
pub extern "C" fn omf_reader_array_colors_iter(
    reader: *mut Reader,
    array: *const Array,
) -> *mut Colors {
    catch::error(|| {
        Ok(colors_new(
            wrapper!(reader).inner.array_colors(&array!(array)?)?,
        ))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_reader_array_colors(
    reader: *mut Reader,
    array: *const Array,
    values: *mut [u8; 4],
    mask: *mut bool,
    n_values: usize,
) -> bool {
    catch::error(|| {
        let array = array!(array)?;
        let iter = wrapper!(reader).inner.array_colors(&array)?;
        into_slice_nullable(values, mask, n_values, array.item_count(), iter)
    })
    .is_some()
}
