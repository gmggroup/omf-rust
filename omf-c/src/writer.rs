use std::{
    ffi::{c_char, CStr},
    fs::File,
    panic::RefUnwindSafe,
    path::PathBuf,
    ptr::{null, null_mut},
    sync::Mutex,
};

use omf::date_time::{i64_to_date, i64_to_date_time};

use crate::{
    arrays::{Array, ArrayType},
    error::{Error, InvalidArg},
    ffi_tools::{
        arg::{not_null, not_null_consume, slice, string_not_null},
        catch, FfiStorage,
    },
    image_data::ImageMode,
    validation::handle_validation,
    writer_handle::{Handle, HandleComponent},
};

use crate::{
    attributes::Attribute,
    elements::{Element, Project},
    validation::Validation,
};

pub(crate) struct WriterWrapper {
    pub path: PathBuf,
    pub inner: omf::file::Writer,
    pub project: Option<omf::Project>,
    pub storage: FfiStorage,
}

impl WriterWrapper {
    fn project_mut(&mut self) -> Result<&mut omf::Project, Error> {
        self.project
            .as_mut()
            .ok_or_else(|| Error::InvalidCall("you must call omf_writer_project first".to_owned()))
    }
}

pub struct Writer(pub(crate) Mutex<WriterWrapper>);

macro_rules! wrapper {
    ($writer:ident) => {
        not_null!($writer)?.0.lock().expect("intact lock")
    };
}

#[no_mangle]
pub extern "C" fn omf_writer_open(path: *const c_char) -> *mut Writer {
    catch::error(|| {
        let path = PathBuf::from(string_not_null!(path)?);
        let wrapper = WriterWrapper {
            inner: omf::file::Writer::open(&path)?,
            path,
            project: None,
            storage: Default::default(),
        };
        Ok(Box::into_raw(Box::new(Writer(Mutex::new(wrapper)))))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_writer_compression(writer: *mut Writer) -> i32 {
    catch::error(|| Ok(wrapper!(writer).inner.compression().level() as i32)).unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn omf_writer_set_compression(writer: *mut Writer, compression: i32) -> bool {
    catch::error(|| {
        wrapper!(writer)
            .inner
            .set_compression(omf::file::Compression::new(compression.clamp(0, 9) as u32));
        Ok(true)
    })
    .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn omf_writer_finish(writer: *mut Writer, validation: *mut *mut Validation) -> bool {
    catch::error(|| {
        let writer = not_null_consume!(writer)?;
        // Clear validation if not null.
        if !validation.is_null() {
            unsafe { validation.write(null_mut()) };
        }
        // Get the omf::Project and omf::file::Writer.
        let wrapper = writer.0.into_inner().expect("intact lock");
        let project = wrapper.project.unwrap_or_default();
        // Finish writing file.
        let result = wrapper.inner.finish(project);
        match &result {
            Ok((_file, warnings)) => handle_validation(warnings, validation),
            Err(omf::error::Error::ValidationFailed(errors)) => {
                handle_validation(errors, validation)
            }
            _ => {}
        }
        result?;
        Ok(true)
    })
    .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn omf_writer_cancel(writer: *mut Writer) -> bool {
    catch::error(|| {
        let writer = not_null_consume!(writer)?;
        let state = writer.0.into_inner().expect("intact lock");
        std::fs::remove_file(state.path).map_err(omf::error::Error::IoError)?;
        Ok(true)
    })
    .unwrap_or(false)
}

fn omf_writer_metadata(
    writer: *mut Writer,
    handle: *mut Handle,
    key: *const c_char,
    value: impl Into<serde_json::Value>,
) -> Result<*mut Handle, Error> {
    let mut wrapper = not_null!(writer)?.0.lock().expect("intact lock");
    let handle = Handle::from_ptr(handle)?;
    let json_value = value.into();
    let is_recursive = matches!(
        json_value,
        serde_json::Value::Array(_) | serde_json::Value::Object(_)
    );
    let new_comp = match handle.metadata(wrapper.project_mut()?)? {
        crate::writer_handle::HandleMetadata::Map(m) => {
            let key = string_not_null!(key)?;
            m.insert(key.clone(), json_value);
            HandleComponent::Nested { key }
        }
        crate::writer_handle::HandleMetadata::Vec(v) => {
            let index = v.len();
            v.push(json_value);
            HandleComponent::Array { index }
        }
    };
    if is_recursive {
        Ok(wrapper.storage.keep_mut(handle.join(new_comp)))
    } else {
        Ok(null_mut())
    }
}

#[no_mangle]
pub extern "C" fn omf_writer_metadata_null(
    writer: *mut Writer,
    handle: *mut Handle,
    name: *const c_char,
) -> bool {
    catch::error(|| omf_writer_metadata(writer, handle, name, ())).is_some()
}

#[no_mangle]
pub extern "C" fn omf_writer_metadata_boolean(
    writer: *mut Writer,
    handle: *mut Handle,
    name: *const c_char,
    value: bool,
) -> bool {
    catch::error(|| omf_writer_metadata(writer, handle, name, value)).is_some()
}

#[no_mangle]
pub extern "C" fn omf_writer_metadata_number(
    writer: *mut Writer,
    handle: *mut Handle,
    name: *const c_char,
    value: f64,
) -> bool {
    catch::error(|| omf_writer_metadata(writer, handle, name, value)).is_some()
}

#[no_mangle]
pub extern "C" fn omf_writer_metadata_string(
    writer: *mut Writer,
    handle: *mut Handle,
    name: *const c_char,
    value: *const c_char,
) -> bool {
    catch::error(|| omf_writer_metadata(writer, handle, name, string_not_null!(value)?)).is_some()
}

#[no_mangle]
pub extern "C" fn omf_writer_metadata_list(
    writer: *mut Writer,
    handle: *mut Handle,
    name: *const c_char,
) -> *mut Handle {
    let value = serde_json::Value::Array(Default::default());
    catch::error(|| omf_writer_metadata(writer, handle, name, value)).unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_writer_metadata_object(
    writer: *mut Writer,
    handle: *mut Handle,
    name: *const c_char,
) -> *mut Handle {
    let value = serde_json::Value::Object(Default::default());
    catch::error(|| omf_writer_metadata(writer, handle, name, value)).unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_writer_project(writer: *mut Writer, project: *const Project) -> *mut Handle {
    catch::error(|| {
        let mut wrapper = not_null!(writer)?.0.lock().expect("intact lock");
        if wrapper.project.is_some() {
            return Err(Error::InvalidCall(
                "second call to 'omf_writer_project' on this writer".to_owned(),
            ));
        }
        wrapper.project = Some(not_null!(project)?.to_omf()?);
        Ok(wrapper.storage.keep_mut(Handle::default()))
    })
    .unwrap_or_else(null_mut)
}

fn push_with_index<T>(vec: &mut Vec<T>, item: T) -> usize {
    let index = vec.len();
    vec.push(item);
    index
}

#[no_mangle]
pub extern "C" fn omf_writer_element(
    writer: *mut Writer,
    handle: *mut Handle,
    element: *const Element,
) -> *mut Handle {
    catch::error(|| {
        let mut wrapper = not_null!(writer)?.0.lock().expect("intact lock");
        let handle = Handle::from_ptr(handle)?;
        let index = push_with_index(
            handle.elements(wrapper.project_mut()?)?,
            not_null!(element)?.to_omf()?,
        );
        Ok(wrapper
            .storage
            .keep_mut(handle.join(HandleComponent::Element { index })))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_writer_attribute(
    writer: *mut Writer,
    handle: *mut Handle,
    attribute: *const Attribute,
) -> *mut Handle {
    catch::error(|| {
        let mut wrapper = not_null!(writer)?.0.lock().expect("intact lock");
        let handle = Handle::from_ptr(handle)?;
        let index = push_with_index(
            handle.attributes(wrapper.project_mut()?)?,
            not_null!(attribute)?.to_omf()?,
        );
        Ok(wrapper
            .storage
            .keep_mut(handle.join(HandleComponent::Attribute { index })))
    })
    .unwrap_or_else(null_mut)
}

#[no_mangle]
pub extern "C" fn omf_writer_image_bytes(
    writer: *mut Writer,
    bytes: *const c_char,
    n_bytes: usize,
) -> *const Array {
    catch::error(|| {
        let bytes_slice: &[u8] = slice!(bytes.cast(), n_bytes)?;
        let mut wrapper = not_null!(writer)?.0.lock().expect("intact lock");
        let image = wrapper.inner.image_bytes(bytes_slice)?;
        Ok(wrapper.storage.convert_ptr(image))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_bytes(
    writer: *mut Writer,
    array_type: ArrayType,
    item_count: u64,
    bytes: *const c_char,
    n_bytes: usize,
) -> *const Array {
    catch::error(|| {
        let mut wrapper = not_null!(writer)?.0.lock().expect("intact lock");
        let bytes_slice: &[u8] = slice!(bytes.cast(), n_bytes)?;
        macro_rules! arr {
            ($name:ident) => {
                Array::$name(wrapper.inner.array_bytes(item_count, bytes_slice)?)
            };
        }
        let array = match array_type {
            ArrayType::Image => arr!(Image),
            ArrayType::Scalars32 => arr!(Scalar),
            ArrayType::Scalars64 => arr!(Scalar),
            ArrayType::Vertices32 => arr!(Vertex),
            ArrayType::Vertices64 => arr!(Vertex),
            ArrayType::Segments => arr!(Segment),
            ArrayType::Triangles => arr!(Triangle),
            ArrayType::Names => arr!(Name),
            ArrayType::Gradient => arr!(Gradient),
            ArrayType::Texcoords32 => arr!(Texcoord),
            ArrayType::Texcoords64 => arr!(Texcoord),
            ArrayType::BoundariesFloat32 => arr!(Boundary),
            ArrayType::BoundariesFloat64 => arr!(Boundary),
            ArrayType::BoundariesInt64 => arr!(Boundary),
            ArrayType::BoundariesDate => arr!(Boundary),
            ArrayType::BoundariesDateTime => arr!(Boundary),
            ArrayType::RegularSubblocks => arr!(RegularSubblock),
            ArrayType::FreeformSubblocks32 => arr!(FreeformSubblock),
            ArrayType::FreeformSubblocks64 => arr!(FreeformSubblock),
            ArrayType::NumbersFloat32 => arr!(Number),
            ArrayType::NumbersFloat64 => arr!(Number),
            ArrayType::NumbersInt64 => arr!(Number),
            ArrayType::NumbersDate => arr!(Number),
            ArrayType::NumbersDateTime => arr!(Number),
            ArrayType::Indices => arr!(Index),
            ArrayType::Vectors32x2 => arr!(Vector),
            ArrayType::Vectors64x2 => arr!(Vector),
            ArrayType::Vectors32x3 => arr!(Vector),
            ArrayType::Vectors64x3 => arr!(Vector),
            ArrayType::Text => arr!(Text),
            ArrayType::Booleans => arr!(Boolean),
            ArrayType::Colors => arr!(Color),
            _ => return Err(InvalidArg::Enum.into()),
        };
        Ok(wrapper.storage.keep(array))
    })
    .unwrap_or_else(null)
}

fn copy_pixels<T: Copy + 'static>(
    width: u32,
    height: u32,
    n_channels: usize,
    pixels: *const T,
) -> Result<Vec<T>, Error> {
    let n_bytes = usize::try_from(width)
        .expect("u32 fits in usize")
        .saturating_mul(usize::try_from(height).expect("u32 fits in usize"))
        .saturating_mul(n_channels);
    let slice = slice!(pixels, n_bytes)?;
    let mut vec = Vec::new();
    vec.try_reserve_exact(n_bytes)
        .map_err(|_| omf::error::Error::OutOfMemory)?;
    vec.extend_from_slice(slice);
    Ok(vec)
}

#[no_mangle]
pub extern "C" fn omf_writer_image_file(writer: *mut Writer, path: *const c_char) -> *const Array {
    catch::error(|| {
        let mut wrapper = not_null!(writer)?.0.lock().expect("intact lock");
        let f = File::open(string_not_null!(path)?).map_err(omf::error::Error::from)?;
        let image = wrapper.inner.image_bytes_from(f)?;
        Ok(wrapper.storage.convert_ptr(image))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_image_jpeg(
    writer: *mut Writer,
    width: u32,
    height: u32,
    pixels: *const u8,
    quality: u32,
) -> *const Array {
    catch::error(|| {
        let mut wrapper = not_null!(writer)?.0.lock().expect("intact lock");
        let buffer = copy_pixels(width, height, 3, pixels)?;
        let rgb = image::RgbImage::from_vec(width, height, buffer).expect("correct buffer size");
        let image = wrapper
            .inner
            .image_jpeg(&rgb, quality.clamp(1, 100) as u8)?;
        Ok(wrapper.storage.convert_ptr(image))
    })
    .unwrap_or_else(null)
}

fn write_png<P: image::Pixel>(
    writer: *mut Writer,
    width: u32,
    height: u32,
    pixels: *const P::Subpixel,
) -> Result<*const Array, Error>
where
    P::Subpixel: RefUnwindSafe + 'static,
    image::DynamicImage: From<image::ImageBuffer<P, Vec<P::Subpixel>>>,
{
    let mut wrapper = not_null!(writer)?.0.lock().expect("intact lock");
    let buffer = copy_pixels(width, height, P::CHANNEL_COUNT.into(), pixels)?;
    let image_obj = image::ImageBuffer::<P, Vec<P::Subpixel>>::from_vec(width, height, buffer)
        .expect("correct buffer size")
        .into();
    let image = wrapper.inner.image_png(&image_obj)?;
    Ok(wrapper.storage.convert_ptr(image))
}

#[no_mangle]
pub extern "C" fn omf_writer_image_png8(
    writer: *mut Writer,
    width: u32,
    height: u32,
    mode: ImageMode,
    pixels: *const u8,
) -> *const Array {
    #[allow(unreachable_patterns)]
    catch::error(|| match mode {
        ImageMode::Gray => write_png::<image::Luma<u8>>(writer, width, height, pixels),
        ImageMode::GrayAlpha => write_png::<image::LumaA<u8>>(writer, width, height, pixels),
        ImageMode::Rgb => write_png::<image::Rgb<u8>>(writer, width, height, pixels),
        ImageMode::Rgba => write_png::<image::Rgba<u8>>(writer, width, height, pixels),
        _ => Err(Error::InvalidArgument(InvalidArg::Enum)),
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_image_png16(
    writer: *mut Writer,
    width: u32,
    height: u32,
    mode: ImageMode,
    pixels: *const u16,
) -> *const Array {
    #[allow(unreachable_patterns)]
    catch::error(|| match mode {
        ImageMode::Gray => write_png::<image::Luma<u16>>(writer, width, height, pixels),
        ImageMode::GrayAlpha => write_png::<image::LumaA<u16>>(writer, width, height, pixels),
        ImageMode::Rgb => write_png::<image::Rgb<u16>>(writer, width, height, pixels),
        ImageMode::Rgba => write_png::<image::Rgba<u16>>(writer, width, height, pixels),
        _ => Err(Error::InvalidArgument(InvalidArg::Enum)),
    })
    .unwrap_or_else(null)
}

macro_rules! write_array_from {
    ($writer:ident, $values:ident, $length:ident, $method:ident) => {
        catch::error(|| {
            let mut wrapper = wrapper!($writer);
            let slice = slice!($values, $length)?;
            let array = wrapper.inner.$method(slice.iter().copied())?;
            Ok(wrapper.storage.convert_ptr(array))
        })
        .unwrap_or_else(null)
    };
}

macro_rules! write_nullable_array_from {
    ($writer:ident, $values:ident, $mask:ident, $length:ident, $method:ident) => {
        catch::error(|| {
            let mut wrapper = wrapper!($writer);
            let array = if $mask.is_null() {
                wrapper
                    .inner
                    .$method(slice!($values, $length)?.iter().copied().map(Some))
            } else {
                wrapper.inner.$method(
                    slice!($values, $length)?
                        .iter()
                        .zip(slice!($mask, $length)?)
                        .map(|(v, m)| if *m { None } else { Some(*v) }),
                )
            }?;
            Ok(wrapper.storage.convert_ptr(array))
        })
        .unwrap_or_else(null)
    };
}

macro_rules! write_nullable_array_from_convert {
    ($writer:ident, $values:ident, $mask:ident, $length:ident, $method:ident, $convert:expr) => {
        catch::error(|| {
            let mut wrapper = wrapper!($writer);
            let array = if $mask.is_null() {
                wrapper
                    .inner
                    .$method(slice!($values, $length)?.iter().map(|v| Some($convert(*v))))
            } else {
                wrapper.inner.$method(
                    slice!($values, $length)?
                        .iter()
                        .zip(slice!($mask, $length)?)
                        .map(|(v, m)| if *m { None } else { Some($convert(*v)) }),
                )
            }?;
            Ok(wrapper.storage.convert_ptr(array))
        })
        .unwrap_or_else(null)
    };
}

macro_rules! write_array {
    ($writer:ident . $method:ident ( $iter:expr )) => {
        catch::error(|| {
            let mut wrapper = wrapper!($writer);
            let array = wrapper.inner.$method($iter)?;
            Ok(wrapper.storage.convert_ptr(array))
        })
        .unwrap_or_else(null)
    };
}

fn func_iter<T: Copy + Default + 'static>(
    source: extern "C" fn(*mut (), *mut T) -> bool,
    object: *mut (),
) -> impl Iterator<Item = T> {
    std::iter::from_fn(move || {
        let mut value = T::default();
        if source(object, &mut value) {
            Some(value)
        } else {
            None
        }
    })
}

fn nullable_func_iter<T: Copy + Default + 'static>(
    source: extern "C" fn(*mut (), *mut T, *mut bool) -> bool,
    object: *mut (),
) -> impl Iterator<Item = Option<T>> {
    std::iter::from_fn(move || {
        let mut value = T::default();
        let mut is_null = false;
        if source(object, &mut value, &mut is_null) {
            Some(if is_null { None } else { Some(value) })
        } else {
            None
        }
    })
}

fn nullable_func_iter_convert<T: Copy + Default + 'static, U>(
    source: extern "C" fn(*mut (), *mut T, *mut bool) -> bool,
    object: *mut (),
    convert: impl Fn(T) -> U,
) -> impl Iterator<Item = Option<U>> {
    std::iter::from_fn(move || {
        let mut value = T::default();
        let mut is_null = false;
        if source(object, &mut value, &mut is_null) {
            Some(if is_null { None } else { Some(convert(value)) })
        } else {
            None
        }
    })
}

fn wide_func_iter<T, const N: usize>(
    source: extern "C" fn(*mut (), *mut T) -> bool,
    object: *mut (),
) -> impl Iterator<Item = [T; N]>
where
    T: 'static,
    [T; N]: Copy + Default,
{
    std::iter::from_fn(move || {
        let mut value = <[T; N]>::default();
        if source(object, value.as_mut_ptr()) {
            Some(value)
        } else {
            None
        }
    })
}

fn nullable_wide_func_iter<T, const N: usize>(
    source: extern "C" fn(*mut (), *mut T, *mut bool) -> bool,
    object: *mut (),
) -> impl Iterator<Item = Option<[T; N]>>
where
    T: 'static,
    [T; N]: Copy + Default,
{
    std::iter::from_fn(move || {
        let mut value = <[T; N]>::default();
        let mut is_null = false;
        if source(object, value.as_mut_ptr(), &mut is_null) {
            Some(if is_null { None } else { Some(value) })
        } else {
            None
        }
    })
}

pub type Scalar32Source = extern "C" fn(*mut (), *mut f32) -> bool;
pub type Scalar64Source = extern "C" fn(*mut (), *mut f64) -> bool;
pub type Vertex32Source = extern "C" fn(*mut (), *mut f32) -> bool;
pub type Vertex64Source = extern "C" fn(*mut (), *mut f64) -> bool;
pub type SegmentSource = extern "C" fn(*mut (), *mut u32) -> bool;
pub type TriangleSource = extern "C" fn(*mut (), *mut u32) -> bool;
pub type NameSource = extern "C" fn(*mut (), *mut *const c_char, *mut usize) -> bool;
pub type GradientSource = extern "C" fn(*mut (), *mut u8) -> bool;
pub type Texcoord32Source = extern "C" fn(*mut (), *mut f32) -> bool;
pub type Texcoord64Source = extern "C" fn(*mut (), *mut f64) -> bool;
pub type BoundaryFloat32Source = extern "C" fn(*mut (), *mut f32, *mut bool) -> bool;
pub type BoundaryFloat64Source = extern "C" fn(*mut (), *mut f64, *mut bool) -> bool;
pub type BoundaryInt64Source = extern "C" fn(*mut (), *mut i64, *mut bool) -> bool;
pub type RegularSubblockSource = extern "C" fn(*mut (), *mut u32, *mut u32) -> bool;
pub type FreeformSubblock32Source = extern "C" fn(*mut (), *mut u32, *mut f32) -> bool;
pub type FreeformSubblock64Source = extern "C" fn(*mut (), *mut u32, *mut f64) -> bool;
pub type NumberFloat32Source = extern "C" fn(*mut (), *mut f32, *mut bool) -> bool;
pub type NumberFloat64Source = extern "C" fn(*mut (), *mut f64, *mut bool) -> bool;
pub type NumberInt64Source = extern "C" fn(*mut (), *mut i64, *mut bool) -> bool;
pub type IndexSource = extern "C" fn(*mut (), *mut u32, *mut bool) -> bool;
pub type Vector32x2Source = extern "C" fn(*mut (), *mut f32, *mut bool) -> bool;
pub type Vector64x2Source = extern "C" fn(*mut (), *mut f64, *mut bool) -> bool;
pub type Vector32x3Source = extern "C" fn(*mut (), *mut f32, *mut bool) -> bool;
pub type Vector64x3Source = extern "C" fn(*mut (), *mut f64, *mut bool) -> bool;
pub type TextSource = extern "C" fn(*mut (), *mut *const c_char, *mut usize) -> bool;
pub type BooleanSource = extern "C" fn(*mut (), *mut bool, *mut bool) -> bool;
pub type ColorSource = extern "C" fn(*mut (), *mut u8, *mut bool) -> bool;

// Scalars

#[no_mangle]
pub extern "C" fn omf_writer_array_scalars32_iter(
    writer: *mut Writer,
    source: Scalar32Source,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_scalars(func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_scalars64_iter(
    writer: *mut Writer,
    source: Scalar64Source,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_scalars(func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_scalars64(
    writer: *mut Writer,
    values: *const f64,
    length: usize,
) -> *const Array {
    write_array_from!(writer, values, length, array_scalars)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_scalars32(
    writer: *mut Writer,
    values: *const f32,
    length: usize,
) -> *const Array {
    write_array_from!(writer, values, length, array_scalars)
}

// Vertices

#[no_mangle]
pub extern "C" fn omf_writer_array_vertices32_iter(
    writer: *mut Writer,
    source: Vertex32Source,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_vertices(wide_func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_vertices64_iter(
    writer: *mut Writer,
    source: Vertex64Source,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_vertices(wide_func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_vertices64(
    writer: *mut Writer,
    values: *const [f64; 3],
    length: usize,
) -> *const Array {
    write_array_from!(writer, values, length, array_vertices)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_vertices32(
    writer: *mut Writer,
    values: *const [f32; 3],
    length: usize,
) -> *const Array {
    write_array_from!(writer, values, length, array_vertices)
}

// Segments

#[no_mangle]
pub extern "C" fn omf_writer_array_segments_iter(
    writer: *mut Writer,
    source: SegmentSource,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_segments(wide_func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_segments(
    writer: *mut Writer,
    values: *const [u32; 2],
    length: usize,
) -> *const Array {
    write_array_from!(writer, values, length, array_segments)
}

// Triangles

#[no_mangle]
pub extern "C" fn omf_writer_array_triangles_iter(
    writer: *mut Writer,
    source: TriangleSource,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_triangles(wide_func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_triangles(
    writer: *mut Writer,
    values: *const [u32; 3],
    length: usize,
) -> *const Array {
    write_array_from!(writer, values, length, array_triangles)
}

// Names

#[no_mangle]
pub extern "C" fn omf_writer_array_names_iter(
    writer: *mut Writer,
    source: NameSource,
    object: *mut (),
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let mut non_utf8 = false;
        let array = wrapper.inner.array_names(std::iter::from_fn(|| {
            let mut ptr = null();
            let mut len = usize::MAX;
            if source(object, &mut ptr, &mut len) {
                name_from_ptr(ptr, len, &mut non_utf8)
            } else {
                None
            }
        }))?;
        if non_utf8 {
            Err(Error::InvalidArgument(InvalidArg::NotUtf8("names")))
        } else {
            Ok(wrapper.storage.convert_ptr(array))
        }
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_names(
    writer: *mut Writer,
    values: *const *const c_char,
    length: usize,
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let slice = slice!(values, length)?;
        let mut non_utf8 = false;
        let array = wrapper.inner.array_names(
            slice
                .iter()
                .map_while(|ptr| name_from_ptr(*ptr, usize::MAX, &mut non_utf8)),
        )?;
        if non_utf8 {
            Err(Error::InvalidArgument(InvalidArg::NotUtf8("names")))
        } else {
            Ok(wrapper.storage.convert_ptr(array))
        }
    })
    .unwrap_or_else(null)
}

fn name_from_ptr(ptr: *const c_char, len: usize, non_utf8: &mut bool) -> Option<String> {
    if ptr.is_null() {
        Some(String::new())
    } else if len == usize::MAX {
        // Read string as nul-terminated.
        if let Ok(s) = unsafe { CStr::from_ptr(ptr) }.to_str() {
            Some(s.to_string())
        } else {
            *non_utf8 = true;
            None
        }
    } else {
        // Use size.
        let slice = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len) };
        if let Ok(s) = std::str::from_utf8(slice) {
            Some(s.to_owned())
        } else {
            *non_utf8 = true;
            None
        }
    }
}

// Gradient

#[no_mangle]
pub extern "C" fn omf_writer_array_gradient_iter(
    writer: *mut Writer,
    source: GradientSource,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_gradient(wide_func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_gradient(
    writer: *mut Writer,
    values: *const [u8; 4],
    length: usize,
) -> *const Array {
    write_array_from!(writer, values, length, array_gradient)
}

// Texcoords

#[no_mangle]
pub extern "C" fn omf_writer_array_texcoords32_iter(
    writer: *mut Writer,
    source: Texcoord32Source,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_texcoords(wide_func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_texcoords64_iter(
    writer: *mut Writer,
    source: Texcoord64Source,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_texcoords(wide_func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_texcoords64(
    writer: *mut Writer,
    values: *const [f64; 2],
    length: usize,
) -> *const Array {
    write_array_from!(writer, values, length, array_texcoords)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_texcoords32(
    writer: *mut Writer,
    values: *const [f32; 2],
    length: usize,
) -> *const Array {
    write_array_from!(writer, values, length, array_texcoords)
}

// Boundaries

fn boundary_iter<T: Default, U: omf::data::NumberType + 'static>(
    source: extern "C" fn(*mut (), *mut T, *mut bool) -> bool,
    object: *mut (),
    convert: impl Fn(T) -> U,
) -> impl Iterator<Item = omf::data::Boundary<U>> {
    std::iter::from_fn(move || {
        let mut value = T::default();
        let mut inclusive = false;
        if source(object, &mut value, &mut inclusive) {
            if inclusive {
                Some(omf::data::Boundary::LessEqual(convert(value)))
            } else {
                Some(omf::data::Boundary::Less(convert(value)))
            }
        } else {
            None
        }
    })
}

fn boundary_iter_from<T: Copy + 'static, U: omf::data::NumberType + 'static>(
    values: &'static [T],
    convert: impl Fn(T) -> U + 'static,
) -> impl Iterator<Item = omf::data::Boundary<U>> {
    values
        .iter()
        .map(move |v| omf::data::Boundary::from_value(convert(*v), false))
}

fn boundary_iter_from_inc<T: Copy + 'static, U: omf::data::NumberType + 'static>(
    values: &'static [T],
    inclusive: &'static [bool],
    convert: impl Fn(T) -> U + 'static,
) -> impl Iterator<Item = omf::data::Boundary<U>> {
    values
        .iter()
        .zip(inclusive.iter())
        .map(move |(v, i)| omf::data::Boundary::from_value(convert(*v), *i))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_boundaries_float32_iter(
    writer: *mut Writer,
    source: BoundaryFloat32Source,
    object: *mut (),
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let array = wrapper
            .inner
            .array_boundaries(boundary_iter(source, object, |x| x))?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_boundaries_float64_iter(
    writer: *mut Writer,
    source: BoundaryFloat64Source,
    object: *mut (),
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let array = wrapper
            .inner
            .array_boundaries(boundary_iter(source, object, |x| x))?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_boundaries_int64_iter(
    writer: *mut Writer,
    source: BoundaryInt64Source,
    object: *mut (),
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let array = wrapper
            .inner
            .array_boundaries(boundary_iter(source, object, |x| x))?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_boundaries_date_iter(
    writer: *mut Writer,
    source: BoundaryInt64Source,
    object: *mut (),
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let array = wrapper
            .inner
            .array_boundaries(boundary_iter(source, object, i64_to_date))?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_boundaries_date_time_iter(
    writer: *mut Writer,
    source: BoundaryInt64Source,
    object: *mut (),
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let array =
            wrapper
                .inner
                .array_boundaries(boundary_iter(source, object, i64_to_date_time))?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_boundaries_float32(
    writer: *mut Writer,
    values: *const f32,
    inclusive: *const bool,
    length: usize,
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let values = slice!(values, length)?;
        let array = if inclusive.is_null() {
            wrapper
                .inner
                .array_boundaries(boundary_iter_from(values, |x| x))
        } else {
            wrapper.inner.array_boundaries(boundary_iter_from_inc(
                values,
                slice!(inclusive, length)?,
                |x| x,
            ))
        }?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_boundaries_float64(
    writer: *mut Writer,
    values: *const f64,
    inclusive: *const bool,
    length: usize,
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let values = slice!(values, length)?;
        let array = if inclusive.is_null() {
            wrapper
                .inner
                .array_boundaries(boundary_iter_from(values, |x| x))
        } else {
            wrapper.inner.array_boundaries(boundary_iter_from_inc(
                values,
                slice!(inclusive, length)?,
                |x| x,
            ))
        }?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_boundaries_int64(
    writer: *mut Writer,
    values: *const i64,
    inclusive: *const bool,
    length: usize,
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let values = slice!(values, length)?;
        let array = if inclusive.is_null() {
            wrapper
                .inner
                .array_boundaries(boundary_iter_from(values, |x| x))
        } else {
            wrapper.inner.array_boundaries(boundary_iter_from_inc(
                values,
                slice!(inclusive, length)?,
                |x| x,
            ))
        }?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_boundaries_date(
    writer: *mut Writer,
    values: *const i64,
    inclusive: *const bool,
    length: usize,
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let values = slice!(values, length)?;
        let array = if inclusive.is_null() {
            wrapper
                .inner
                .array_boundaries(boundary_iter_from(values, i64_to_date))
        } else {
            let inc = slice!(inclusive, length)?;
            wrapper
                .inner
                .array_boundaries(boundary_iter_from_inc(values, inc, i64_to_date))
        }?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_boundaries_date_time(
    writer: *mut Writer,
    values: *const i64,
    inclusive: *const bool,
    length: usize,
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let values = slice!(values, length)?;
        let array = if inclusive.is_null() {
            wrapper
                .inner
                .array_boundaries(boundary_iter_from(values, i64_to_date_time))
        } else {
            let inc = slice!(inclusive, length)?;
            wrapper
                .inner
                .array_boundaries(boundary_iter_from_inc(values, inc, i64_to_date_time))
        }?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

// Regular sub-blocks

#[no_mangle]
pub extern "C" fn omf_writer_array_regular_subblocks_iter(
    writer: *mut Writer,
    source: RegularSubblockSource,
    object: *mut (),
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let array = wrapper
            .inner
            .array_regular_subblocks(std::iter::from_fn(|| {
                let mut parent = [0; 3];
                let mut corners = [0; 6];
                if source(object, parent.as_mut_ptr(), corners.as_mut_ptr()) {
                    Some((parent, corners))
                } else {
                    None
                }
            }))?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_regular_subblocks(
    writer: *mut Writer,
    parents: *const [u32; 3],
    corners: *const [u32; 6],
    length: usize,
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let parents = slice!(parents, length)?;
        let corners = slice!(corners, length)?;
        let array = wrapper
            .inner
            .array_regular_subblocks(parents.iter().copied().zip(corners.iter().copied()))?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

// Free-form sub-blocks

#[no_mangle]
pub extern "C" fn omf_writer_array_freeform_subblocks32_iter(
    writer: *mut Writer,
    source: FreeformSubblock32Source,
    object: *mut (),
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let array = wrapper
            .inner
            .array_freeform_subblocks(std::iter::from_fn(|| {
                let mut parent = [0; 3];
                let mut corners = [0.0; 6];
                if source(object, parent.as_mut_ptr(), corners.as_mut_ptr()) {
                    Some((parent, corners))
                } else {
                    None
                }
            }))?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_freeform_subblocks64_iter(
    writer: *mut Writer,
    source: FreeformSubblock64Source,
    object: *mut (),
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let array = wrapper
            .inner
            .array_freeform_subblocks(std::iter::from_fn(|| {
                let mut parent = [0; 3];
                let mut corners = [0.0; 6];
                if source(object, parent.as_mut_ptr(), corners.as_mut_ptr()) {
                    Some((parent, corners))
                } else {
                    None
                }
            }))?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_freeform_subblocks32(
    writer: *mut Writer,
    parents: *const [u32; 3],
    corners: *const [f32; 6],
    length: usize,
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let parents = slice!(parents, length)?;
        let corners = slice!(corners, length)?;
        let array = wrapper
            .inner
            .array_freeform_subblocks(parents.iter().copied().zip(corners.iter().copied()))?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_freeform_subblocks64(
    writer: *mut Writer,
    parents: *const [u32; 3],
    corners: *const [f64; 6],
    length: usize,
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let parents = slice!(parents, length)?;
        let corners = slice!(corners, length)?;
        let array = wrapper
            .inner
            .array_freeform_subblocks(parents.iter().copied().zip(corners.iter().copied()))?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

// Numbers

#[no_mangle]
pub extern "C" fn omf_writer_array_numbers_float32_iter(
    writer: *mut Writer,
    source: NumberFloat32Source,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_numbers(nullable_func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_numbers_float64_iter(
    writer: *mut Writer,
    source: NumberFloat64Source,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_numbers(nullable_func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_numbers_int64_iter(
    writer: *mut Writer,
    source: NumberInt64Source,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_numbers(nullable_func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_numbers_date_iter(
    writer: *mut Writer,
    source: NumberInt64Source,
    object: *mut (),
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let array =
            wrapper
                .inner
                .array_numbers(nullable_func_iter_convert(source, object, i64_to_date))?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_numbers_date_time_iter(
    writer: *mut Writer,
    source: NumberInt64Source,
    object: *mut (),
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let array = wrapper.inner.array_numbers(nullable_func_iter_convert(
            source,
            object,
            i64_to_date_time,
        ))?;
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_numbers_float32(
    writer: *mut Writer,
    values: *const f32,
    mask: *const bool,
    length: usize,
) -> *const Array {
    write_nullable_array_from!(writer, values, mask, length, array_numbers)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_numbers_float64(
    writer: *mut Writer,
    values: *const f64,
    mask: *const bool,
    length: usize,
) -> *const Array {
    write_nullable_array_from!(writer, values, mask, length, array_numbers)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_numbers_int64(
    writer: *mut Writer,
    values: *const i64,
    mask: *const bool,
    length: usize,
) -> *const Array {
    write_nullable_array_from!(writer, values, mask, length, array_numbers)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_numbers_date(
    writer: *mut Writer,
    values: *const i32,
    mask: *const bool,
    length: usize,
) -> *const Array {
    let convert = |n: i32| i64_to_date(n.into());
    write_nullable_array_from_convert!(writer, values, mask, length, array_numbers, convert)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_numbers_date_time(
    writer: *mut Writer,
    values: *const i64,
    mask: *const bool,
    length: usize,
) -> *const Array {
    write_nullable_array_from_convert!(
        writer,
        values,
        mask,
        length,
        array_numbers,
        i64_to_date_time
    )
}

// Indices

#[no_mangle]
pub extern "C" fn omf_writer_array_indices_iter(
    writer: *mut Writer,
    source: IndexSource,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_indices(nullable_func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_indices(
    writer: *mut Writer,
    values: *const u32,
    mask: *const bool,
    length: usize,
) -> *const Array {
    write_nullable_array_from!(writer, values, mask, length, array_indices)
}

// Vectors

#[no_mangle]
pub extern "C" fn omf_writer_array_vectors32x2_iter(
    writer: *mut Writer,
    source: Vector32x2Source,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_vectors(nullable_wide_func_iter::<_, 2>(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_vectors32x3_iter(
    writer: *mut Writer,
    source: Vector32x3Source,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_vectors(nullable_wide_func_iter::<_, 3>(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_vectors64x2_iter(
    writer: *mut Writer,
    source: Vector64x2Source,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_vectors(nullable_wide_func_iter::<_, 2>(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_vectors64x3_iter(
    writer: *mut Writer,
    source: Vector64x3Source,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_vectors(nullable_wide_func_iter::<_, 3>(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_vectors32x2(
    writer: *mut Writer,
    values: *const [f32; 2],
    mask: *const bool,
    length: usize,
) -> *const Array {
    write_nullable_array_from!(writer, values, mask, length, array_vectors)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_vectors64x2(
    writer: *mut Writer,
    values: *const [f64; 2],
    mask: *const bool,
    length: usize,
) -> *const Array {
    write_nullable_array_from!(writer, values, mask, length, array_vectors)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_vectors32x3(
    writer: *mut Writer,
    values: *const [f32; 3],
    mask: *const bool,
    length: usize,
) -> *const Array {
    write_nullable_array_from!(writer, values, mask, length, array_vectors)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_vectors64x3(
    writer: *mut Writer,
    values: *const [f64; 3],
    mask: *const bool,
    length: usize,
) -> *const Array {
    write_nullable_array_from!(writer, values, mask, length, array_vectors)
}

// Text

#[no_mangle]
pub extern "C" fn omf_writer_array_text_iter(
    writer: *mut Writer,
    source: TextSource,
    object: *mut (),
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let mut non_utf8 = false;
        let array = wrapper.inner.array_text(std::iter::from_fn(|| {
            let mut ptr = null();
            let mut len = usize::MAX;
            if source(object, &mut ptr, &mut len) {
                text_from_ptr(ptr, len, &mut non_utf8)
            } else {
                None
            }
        }))?;
        if non_utf8 {
            Err(Error::InvalidArgument(InvalidArg::NotUtf8("names")))
        } else {
            Ok(wrapper.storage.convert_ptr(array))
        }
    })
    .unwrap_or_else(null)
}

#[no_mangle]
pub extern "C" fn omf_writer_array_text(
    writer: *mut Writer,
    values: *const *const c_char,
    length: usize,
) -> *const Array {
    catch::error(|| {
        let mut wrapper = wrapper!(writer);
        let slice = slice!(values, length)?;
        let mut non_utf8 = false;
        let array = wrapper.inner.array_text(
            slice
                .iter()
                .map_while(|ptr| text_from_ptr(*ptr, usize::MAX, &mut non_utf8)),
        )?;
        if non_utf8 {
            return Err(Error::InvalidArgument(InvalidArg::NotUtf8(
                "array of names",
            )));
        }
        Ok(wrapper.storage.convert_ptr(array))
    })
    .unwrap_or_else(null)
}

fn text_from_ptr(ptr: *const c_char, len: usize, non_utf8: &mut bool) -> Option<Option<String>> {
    if ptr.is_null() {
        Some(None)
    } else if len == usize::MAX {
        // Read string as nul-terminated.
        if let Ok(s) = unsafe { CStr::from_ptr(ptr) }.to_str() {
            Some(Some(s.to_string()))
        } else {
            *non_utf8 = true;
            None
        }
    } else {
        // Use size.
        let slice = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len) };
        if let Ok(s) = std::str::from_utf8(slice) {
            Some(Some(s.to_owned()))
        } else {
            *non_utf8 = true;
            None
        }
    }
}

// Booleans

#[no_mangle]
pub extern "C" fn omf_writer_array_booleans_iter(
    writer: *mut Writer,
    source: BooleanSource,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_booleans(nullable_func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_booleans(
    writer: *mut Writer,
    values: *const bool,
    mask: *const bool,
    length: usize,
) -> *const Array {
    write_nullable_array_from!(writer, values, mask, length, array_booleans)
}

// Colors

#[no_mangle]
pub extern "C" fn omf_writer_array_colors_iter(
    writer: *mut Writer,
    source: ColorSource,
    object: *mut (),
) -> *const Array {
    write_array!(writer.array_colors(nullable_wide_func_iter(source, object)))
}

#[no_mangle]
pub extern "C" fn omf_writer_array_colors(
    writer: *mut Writer,
    values: *const [u8; 4],
    mask: *const bool,
    length: usize,
) -> *const Array {
    write_nullable_array_from!(writer, values, mask, length, array_colors)
}
