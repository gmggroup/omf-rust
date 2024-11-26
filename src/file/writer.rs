use std::{
    fmt::Debug,
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    path::Path,
};

use flate2::write::GzEncoder;

use crate::{
    array::DataType,
    array_type,
    error::Error,
    file::zip_container::FileType,
    validate::{Problems, Validate, Validator},
    Array, ArrayType, Project, FORMAT_VERSION_MAJOR, FORMAT_VERSION_MINOR,
    FORMAT_VERSION_PRERELEASE,
};

use super::zip_container::Builder;

/// Compression level to use. Applies to Parquet and JSON data in the OMF file.
#[derive(Debug, Clone, Copy)]
pub struct Compression(u32);

impl Compression {
    const MINIMUM: u32 = 0;
    const MAXIMUM: u32 = 9;

    /// Create a compression level, clamped to the range `0..=9`.
    pub fn new(level: u32) -> Self {
        Self(level.clamp(Self::MINIMUM, Self::MAXIMUM))
    }

    /// No compression.
    pub const fn none() -> Self {
        Self(0)
    }

    /// Compress as fast as possible at the cost of file size.
    pub const fn fast() -> Self {
        Self(1)
    }

    /// Take as long as necessary to compress as small as possible.
    pub const fn best() -> Self {
        Self(9)
    }

    /// Returns the compression level.
    pub const fn level(&self) -> u32 {
        self.0
    }
}

impl Default for Compression {
    /// The default compression level, a balance between speed and file size.
    fn default() -> Self {
        Self(6)
    }
}

impl From<Compression> for flate2::Compression {
    fn from(value: Compression) -> Self {
        Self::new(value.level())
    }
}

/// OMF writer object.
///
/// To use the writer:
///
/// 1. Create the writer object.
/// 1. Create an empty [`Project`] and fill in the details.
/// 1. For each element you want to store:
///     1. Write the arrays and image with the writer.
///     1. Fill in the required struct with the array pointers and other details then add it to the project.
///     1. Repeat for the attributes, adding them to the newly created element.
/// 1. Call `writer.finish(project)` to validate everything inside the the project and write it.
pub struct Writer<W: Write + Seek> {
    pub(crate) builder: Builder<W>,
    compression: Compression,
}

impl Writer<File> {
    /// Creates a writer by opening a file.
    ///
    /// The file will be created if it doesn't exist, and truncated and replaced if it does.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        Self::new(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)?,
        )
    }
}

impl<W: Write + Seek> Writer<W> {
    /// Creates a writer that writes into a file-like object.
    pub fn new(write: W) -> Result<Self, Error> {
        Ok(Self {
            builder: Builder::new(write)?,
            compression: Default::default(),
        })
    }

    /// Return the current compression.
    pub fn compression(&self) -> Compression {
        self.compression
    }

    /// Set the compression to use.
    ///
    /// This affects Parquet data and the JSON index, but not images.
    /// The default is `Compression::default()`.
    pub fn set_compression(&mut self, compression: Compression) {
        self.compression = compression;
    }

    /// Write an array from already-encoded bytes.
    ///
    /// Returns the new [`Array`](crate::Array) on success or an error if file IO fails.
    pub fn array_bytes<A: ArrayType>(
        &mut self,
        length: u64,
        bytes: &[u8],
    ) -> Result<Array<A>, Error> {
        let file_type = check_header::<A>(bytes)?;
        let mut f = self.builder.open(file_type)?;
        let name = f.name().to_owned();
        f.write_all(bytes)?;
        Ok(Array::new(name, length))
    }

    /// Consumes everything from `read` and writes it as a new array.
    ///
    /// The bytes must already be encoded in Parquet, PNG, or JPEG depending on the array type.
    /// Returns the new [`Array`](crate::Array) on success or an error if file IO fails on either
    /// side.
    pub fn array_bytes_from<A: ArrayType>(
        &mut self,
        length: u64,
        mut read: impl Read,
    ) -> Result<Array<A>, Error> {
        let mut header = [0_u8; 8];
        read.read_exact(&mut header)?;
        let file_type = check_header::<A>(&header)?;
        let mut f = self.builder.open(file_type)?;
        let name = f.name().to_owned();
        f.write_all(&header)?;
        let mut buffer = vec![0_u8; 4096];
        loop {
            let n = read.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            f.write_all(&buffer)?;
        }
        Ok(Array::new(name, length))
    }

    /// Write an existing PNG or JPEG image from a slice without re-encoding it.
    pub fn image_bytes(&mut self, bytes: &[u8]) -> Result<Array<array_type::Image>, Error> {
        self.array_bytes(0, bytes)
    }

    /// Write an existing PNG or JPEG image from a file without re-encoding it.
    pub fn image_bytes_from(&mut self, read: impl Read) -> Result<Array<array_type::Image>, Error> {
        self.array_bytes_from(0, read)
    }

    /// Validate and write the project and close the file.
    ///
    /// Returns validation warnings on success or an [`Error`] on failure, which can be a
    /// validation failure or a file IO error.
    pub fn finish(mut self, mut project: Project) -> Result<(W, Problems), Error> {
        let mut val = Validator::new().with_filenames(self.builder.filenames());
        project.validate_inner(&mut val);
        let warnings = val.finish().into_result()?;
        let gz = GzEncoder::new(self.builder.open(FileType::Index)?, self.compression.into());
        serde_json::to_writer(gz, &project).map_err(Error::SerializationFailed)?;
        // In the future we could base the format version on the data, writing backward
        // compatible files if new features weren't used.
        let write = self.builder.finish(
            FORMAT_VERSION_MAJOR,
            FORMAT_VERSION_MINOR,
            FORMAT_VERSION_PRERELEASE,
        )?;
        Ok((write, warnings))
    }
}

fn check_header<A: ArrayType>(bytes: &[u8]) -> Result<FileType, Error> {
    const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8, 0xFF];
    const PARQUET_MAGIC: &[u8] = b"PAR1";
    match A::DATA_TYPE {
        DataType::Image => {
            if bytes.starts_with(PNG_MAGIC) {
                Ok(FileType::Png)
            } else if bytes.starts_with(JPEG_MAGIC) {
                Ok(FileType::Jpeg)
            } else {
                Err(Error::NotImageData)
            }
        }
        _ => {
            if !bytes.starts_with(PARQUET_MAGIC) || !bytes.ends_with(PARQUET_MAGIC) {
                Err(Error::NotParquetData)
            } else {
                Ok(FileType::Parquet)
            }
        }
    }
}
