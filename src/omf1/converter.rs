use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::Path,
};

use crate::{
    error::Error,
    file::{Compression, Limits},
    validate::Problems,
};

use super::reader::Omf1Reader;

/// Returns true if the file looks more like OMF1 than OMF2.
///
/// Does not guarantee that the file will load. Returns an error in file read fails.
pub fn detect(read: &mut impl Read) -> Result<bool, Error> {
    const PREFIX: [u8; 8] = [0x84, 0x83, 0x82, 0x81, b'O', b'M', b'F', b'-'];
    let mut prefix = [0; PREFIX.len()];
    read.read_exact(&mut prefix)?;
    Ok(prefix == PREFIX)
}

/// Returns true if the path looks more like OMF1 than OMF2.
///
/// Does not guarantee that the file will load. Returns an error in file open or read fails.
pub fn detect_open(path: &Path) -> Result<bool, Error> {
    detect(&mut File::open(path)?)
}

/// Converts a OMF1 files to OMF2.
///
/// This object allows you to set up the desired parameters then convert one or more files.
#[derive(Debug, Default)]
pub struct Converter {
    limits: Limits,
    compression: Compression,
}

impl Converter {
    /// Creates a new default converter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current limits.
    pub fn limits(&self) -> Limits {
        self.limits
    }

    /// Set the limits to use during conversion.
    pub fn set_limits(&mut self, limits: Limits) {
        self.limits = limits;
    }

    /// Returns the current compression level.
    pub fn compression(&self) -> Compression {
        self.compression
    }

    /// Set the compression level to use when writing.
    pub fn set_compression(&mut self, compression: Compression) {
        self.compression = compression;
    }

    /// Runs a conversion from one open file to another file.
    ///
    /// `input` must support read and seek, while `output` must support write.
    /// On success the validation warnings are returned.
    ///
    /// May be called more than once to convert multiple files with the same parameters.
    pub fn convert(&self, input: File, output: File) -> Result<Problems, Error> {
        let reader = Omf1Reader::new(input, self.limits.json_bytes)?;
        let mut writer = crate::file::Writer::new(output)?;
        writer.set_compression(self.compression);
        let project = reader.project()?.convert(&reader, &mut writer)?;
        writer.finish(project).map(|(_, p)| p)
    }

    /// Runs a conversion from one filename to another.
    ///
    /// The output file will be created if it does not exist, and truncated if it does.
    /// On success the validation warnings are returned.
    ///
    /// May be called more than once to convert multiple files with the same parameters.
    pub fn convert_open(
        &self,
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
    ) -> Result<Problems, Error> {
        let input = File::open(input_path)?;
        let output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_path)?;
        self.convert(input, output)
    }
}
