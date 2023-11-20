use std::{fmt::Debug, ptr::null};

use crate::ffi_tools::{into_ffi_free, FfiWrapper};

#[repr(C)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub mode: ImageMode,
    pub uint8: *const u8,
    pub uint16: *const u16,
}

impl FfiWrapper<image::DynamicImage> for ImageData {
    fn wrap(value: &image::DynamicImage) -> Self {
        let mut uint8 = null();
        let mut uint16 = null();
        match value {
            image::DynamicImage::ImageLuma8(img) => uint8 = img.as_ptr(),
            image::DynamicImage::ImageLumaA8(img) => uint8 = img.as_ptr(),
            image::DynamicImage::ImageRgb8(img) => uint8 = img.as_ptr(),
            image::DynamicImage::ImageRgba8(img) => uint8 = img.as_ptr(),
            image::DynamicImage::ImageLuma16(img) => uint16 = img.as_ptr(),
            image::DynamicImage::ImageLumaA16(img) => uint16 = img.as_ptr(),
            image::DynamicImage::ImageRgb16(img) => uint16 = img.as_ptr(),
            image::DynamicImage::ImageRgba16(img) => uint16 = img.as_ptr(),
            _ => panic!("unexpected image type"),
        }
        Self {
            width: value.width(),
            height: value.height(),
            mode: value.color().into(),
            uint8,
            uint16,
        }
    }
}

#[no_mangle]
pub extern "C" fn omf_image_data_free(data: *mut ImageData) -> bool {
    unsafe { into_ffi_free(data) }
}

#[derive(Debug)]
#[repr(i32)]
#[non_exhaustive]
pub enum ImageMode {
    Gray = 1,
    GrayAlpha = 2,
    Rgb = 3,
    Rgba = 4,
}

impl From<image::ColorType> for ImageMode {
    fn from(value: image::ColorType) -> Self {
        use image::ColorType::*;
        match value {
            L8 | L16 => Self::Gray,
            La8 | La16 => Self::GrayAlpha,
            Rgb8 | Rgb16 | Rgb32F => Self::Rgb,
            Rgba8 | Rgba16 | Rgba32F => Self::Rgba,
            _ => panic!("unexpected image type"),
        }
    }
}
