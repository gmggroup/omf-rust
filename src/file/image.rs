use std::io::{BufReader, Cursor};

use crate::{array_type, error::Error, Array};

use super::{Limits, Reader, Writer};

impl From<Limits> for image::Limits {
    fn from(value: Limits) -> Self {
        let mut out = Self::no_limits();
        out.max_alloc = value.image_bytes;
        out.max_image_width = value.image_dim;
        out.max_image_height = value.image_dim;
        out
    }
}

impl Reader {
    /// Read and decode an image.
    pub fn image(&self, image: &Array<array_type::Image>) -> Result<image::DynamicImage, Error> {
        let f = BufReader::new(self.array_bytes_reader(image)?);
        let mut reader = image::ImageReader::new(f).with_guessed_format()?;
        reader.limits(self.limits().into());
        Ok(reader.decode()?)
    }
}

impl Writer {
    /// Write an image in PNG encoding.
    ///
    /// This supports grayscale, grayscale + alpha, RGB, and RGBA, in 8 or 16 bits per channel.
    pub fn image_png(
        &mut self,
        image: &image::DynamicImage,
    ) -> Result<Array<array_type::Image>, Error> {
        let mut bytes = Vec::new();
        image.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;
        self.image_bytes(&bytes)
    }

    /// Write an image in JPEG encoding.
    ///
    /// Unlike PNG this is limited to 8-bit RGB and compression is lossy, but it will give
    /// much better compression ratios. The JPEG compression level is set by the `quality`
    /// argument, from 1 to 100. 90 is a reasonable level for preserving fine detail in the image,
    /// while lower values will give a smaller file.
    ///
    /// If you have an existing image in JPEG encoding you shouldn't be using this method,
    /// instead add the raw bytes of the file with `writer.image_bytes(&bytes)` to avoid recompressing
    /// the image and losing more detail.
    pub fn image_jpeg(
        &mut self,
        image: &image::RgbImage,
        quality: u8,
    ) -> Result<Array<array_type::Image>, Error> {
        let mut bytes = Vec::new();
        image.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(
            &mut Cursor::new(&mut bytes),
            quality.clamp(1, 100),
        ))?;
        self.image_bytes(&bytes)
    }
}
