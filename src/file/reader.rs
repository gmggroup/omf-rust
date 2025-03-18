use std::{
    io::{BufReader, Read},
    sync::Arc,
};

use flate2::read::GzDecoder;

use crate::{
    FORMAT_VERSION_MAJOR, FORMAT_VERSION_MINOR, FORMAT_VERSION_PRERELEASE, Project, array,
    error::{Error, Limit},
    validate::{Problems, Validate, Validator},
};

use super::{
    ReadAt, SubFile,
    zip_container::{Archive, INDEX_NAME},
};

pub const DEFAULT_VALIDATION_LIMIT: u32 = 100;
pub const DEFAULT_JSON_LIMIT: u64 = 1024 * 1024;
#[cfg(target_pointer_width = "32")]
pub const DEFAULT_MEMORY_LIMIT: u64 = 1024 * 1024 * 1024;
#[cfg(not(target_pointer_width = "32"))]
pub const DEFAULT_MEMORY_LIMIT: u64 = 16 * 1024 * 1024 * 1024;

/// Memory limits for reading OMF files.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Maximum uncompressed size for the JSON index.
    ///
    /// Default is 1 MB.
    pub json_bytes: Option<u64>,
    /// Maximum uncompressed image size.
    ///
    /// Default is 1 GB on 32-bit systems or 16 GB on 64-bit systems.
    pub image_bytes: Option<u64>,
    /// Maximum image width or height, default unlimited.
    pub image_dim: Option<u32>,
    /// Maximum number of validation errors.
    ///
    /// Errors beyond this limit will be discarded. Default is 100.
    pub validation: Option<u32>,
}

impl Limits {
    /// Creates an object with no limits set.
    ///
    /// Running without limits is not recommended.
    pub fn no_limits() -> Self {
        Self {
            json_bytes: None,
            image_bytes: None,
            image_dim: None,
            validation: None,
        }
    }
}

impl Default for Limits {
    /// The default limits.
    fn default() -> Self {
        Self {
            json_bytes: Some(DEFAULT_JSON_LIMIT),
            image_bytes: Some(DEFAULT_MEMORY_LIMIT),
            image_dim: None,
            validation: Some(DEFAULT_VALIDATION_LIMIT),
        }
    }
}

/// OMF reader object.
///
/// Typical usage pattern is:
///
/// 1. Create the reader object.
/// 1. Optional: retrieve the file version with `reader.version()`.
/// 1. Optional: adjust the limits with `reader.set_limits(...)`.
/// 1. Read the project from the file with `reader.project()`.
/// 1. Iterate through the project's contents to find the elements and attributes you want to load.
/// 1. For each of those items load the array or image data.
///
/// > **Warning:**
/// > When loading arrays and images from OMF files, beware of "zip bombs"
/// > where data is maliciously crafted to expand to an excessive size when decompressed,
/// > leading to a potential denial of service attack.
/// > Use the limits provided and check arrays sizes before allocating memory.
pub struct Reader<R> {
    archive: Archive<R>,
    version: [u32; 2],
    limits: Limits,
}

#[cfg(not(target_family = "wasm"))]
impl Reader<std::fs::File> {
    /// Creates a reader by opening the given path.
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        Self::new(std::fs::File::open(path)?)
    }
}

impl<R: ReadAt> Reader<R> {
    /// Creates the reader from a [`ReadAt`] object.
    ///
    /// This object can be a `std::fs::File`, a `Vec<u8>`, or anything you implement the trait for.
    /// Currently this is restricted to `'static` objects, and they must be `Sync` and `Send`.
    pub fn new(data: R) -> Result<Self, Error> {
        let size = data.size()?;
        let archive = Archive::new(SubFile::new(Arc::new(data), 0, size)?)?;
        let (version, pre_release) = archive.version();
        if let Some(pre) = pre_release {
            if Some(pre) != FORMAT_VERSION_PRERELEASE {
                return Err(Error::PreReleaseVersion(version[0], version[1], pre.into()));
            }
        }
        if version > [FORMAT_VERSION_MAJOR, FORMAT_VERSION_MINOR] {
            return Err(Error::NewerVersion(version[0], version[1]));
        }
        Ok(Self {
            archive,
            version,
            limits: Default::default(),
        })
    }

    /// Returns the current limits.
    pub fn limits(&self) -> Limits {
        self.limits
    }

    /// Sets the memory limits.
    ///
    /// These limits prevent the reader from consuming excessive system resources, which might
    /// allow denial of service attacks with maliciously crafted files. Running without limits
    /// is not recommended.
    pub fn set_limits(&mut self, limits: Limits) {
        self.limits = limits;
    }

    /// Return the version number of the file, which can only be `[2, 0]` right now.
    pub fn version(&self) -> [u32; 2] {
        self.version
    }

    /// Reads, validates, and returns the root `Project` object from the file.
    ///
    /// Fails with an error if an IO error occurs, the `json_bytes` limit is exceeded, or validation
    /// fails. Validation warnings are returned alongside the project if successful or included
    /// with the errors if not.
    pub fn project(&self) -> Result<(Project, Problems), Error> {
        let mut project: Project = serde_json::from_reader(BufReader::new(LimitedRead::new(
            GzDecoder::new(self.archive.open(INDEX_NAME)?),
            self.limits().json_bytes.unwrap_or(u64::MAX),
        )))
        .map_err(Error::DeserializationFailed)?;
        let mut val = Validator::new()
            .with_filenames(self.archive.filenames())
            .with_limit(self.limits().validation);
        project.validate_inner(&mut val);
        let warnings = val.finish().into_result()?;
        Ok((project, warnings))
    }

    /// Returns the size in bytes of the compressed array.
    pub fn array_compressed_size(
        &self,
        array: &array::Array<impl array::ArrayType>,
    ) -> Result<u64, Error> {
        Ok(self.archive.span(array.filename())?.size)
    }

    /// Returns a sub-file for reading raw bytes from the file.
    ///
    /// Fails with an error if the range is invalid. The contents are not checked or validated by
    /// this method. The caller must ensure they are valid and safe to use. This function doesn't
    /// check against any limit.
    pub fn array_bytes_reader(
        &self,
        array: &array::Array<impl array::ArrayType>,
    ) -> Result<SubFile<R>, Error> {
        array.constraint(); // Check that validation has been done.
        self.archive.open(array.filename())
    }

    /// Return the compressed bytes of an array.
    ///
    /// The will allocate memory to store the result. Call `array_compressed_size` to find out how
    /// much will be allocated.
    pub fn array_bytes(
        &self,
        array: &array::Array<impl array::ArrayType>,
    ) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::new();
        self.array_bytes_reader(array)?.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

struct LimitedRead<R> {
    inner: R,
    limit: u64,
}

impl<R> LimitedRead<R> {
    fn new(inner: R, limit: u64) -> Self {
        Self { inner, limit }
    }
}

impl<R: Read> Read for LimitedRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.limit = self.limit.saturating_sub(n as u64);
        if self.limit == 0 {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                Error::LimitExceeded(Limit::JsonBytes),
            ))
        } else {
            Ok(n)
        }
    }
}
