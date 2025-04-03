use std::{
    collections::HashMap,
    io::{Read, Seek, Write},
};

use zip::{
    read::{ZipArchive, ZipFile},
    write::{FullFileOptions, ZipWriter},
};

use crate::{FORMAT_NAME, error::Error};

use super::{ReadAt, SubFile};

pub(crate) const INDEX_NAME: &str = "index.json.gz";
pub(crate) const PARQUET_EXT: &str = ".parquet";
pub(crate) const PNG_EXT: &str = ".png";
pub(crate) const JPEG_EXT: &str = ".jpg";

pub(crate) enum FileType {
    Index,
    Parquet,
    Png,
    Jpeg,
}

pub(crate) struct Builder<W: Write + Seek> {
    zip_writer: ZipWriter<W>,
    next_id: u64,
    filenames: Vec<String>,
}

impl<W: Write + Seek> Builder<W> {
    pub fn new(write: W) -> Result<Self, Error> {
        Ok(Self {
            zip_writer: ZipWriter::new(write),
            next_id: 1,
            filenames: Vec::new(),
        })
    }

    fn id(&mut self) -> u64 {
        let i = self.next_id;
        self.next_id += 1;
        i
    }

    pub fn open(&mut self, file_type: FileType) -> Result<SubFileWrite<'_, W>, Error> {
        let name = match file_type {
            FileType::Index => INDEX_NAME.to_owned(),
            FileType::Parquet => format!("{}{PARQUET_EXT}", self.id()),
            FileType::Png => format!("{}{PNG_EXT}", self.id()),
            FileType::Jpeg => format!("{}{JPEG_EXT}", self.id()),
        };
        self.zip_writer.start_file(
            name.clone(),
            FullFileOptions::default()
                .large_file(true)
                .compression_method(zip::CompressionMethod::Stored),
        )?;
        self.filenames.push(name.clone());
        Ok(SubFileWrite {
            name,
            inner: &mut self.zip_writer,
        })
    }

    pub fn filenames(&self) -> impl Iterator<Item = &str> {
        self.filenames.iter().map(|s| &**s)
    }

    pub fn finish(mut self, major: u32, minor: u32, pre_release: Option<&str>) -> Result<W, Error> {
        use std::fmt::Write;
        let mut comment = format!("{FORMAT_NAME} {major}.{minor}");
        if let Some(pre) = pre_release {
            _ = write!(&mut comment, "-{pre}");
        }
        self.zip_writer.set_comment(comment);
        Ok(self.zip_writer.finish()?)
    }
}

pub(crate) struct SubFileWrite<'a, W: Write + Seek> {
    name: String,
    inner: &'a mut ZipWriter<W>,
}

impl<W: Write + Seek> SubFileWrite<'_, W> {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<W: Write + Seek> std::io::Write for SubFileWrite<'_, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct FileSpan {
    pub offset: u64,
    pub size: u64,
}

impl<'a, R: Read> From<ZipFile<'a, R>> for FileSpan {
    fn from(f: ZipFile<'a, R>) -> Self {
        Self {
            offset: f.data_start(),
            size: f.compressed_size(),
        }
    }
}

pub(crate) struct Archive<R> {
    file: SubFile<R>,
    members: HashMap<String, FileSpan>,
    version: [u32; 2],
    pre_release: Option<String>,
}

impl<R: ReadAt> Archive<R> {
    pub fn new(file: SubFile<R>) -> Result<Self, Error> {
        let mut zip_archive = ZipArchive::new(file)?;
        let mut members = HashMap::new();
        let mut index_found = false;
        for i in 0..zip_archive.len() {
            let f = zip_archive.by_index_raw(i)?;
            if f.compression() != zip::CompressionMethod::Stored {
                return Err(Error::ZipError("members may not be compressed".into()));
            }
            index_found = index_found || f.name() == INDEX_NAME;
            members.insert(f.name().into(), f.into());
        }
        if !index_found {
            return Err(Error::ZipMemberMissing(INDEX_NAME.to_owned()));
        }
        let Some((version, pre_release)) = get_version(zip_archive.comment()) else {
            return Err(Error::NotOmf(
                String::from_utf8_lossy(zip_archive.comment()).into_owned(),
            ));
        };
        Ok(Self {
            file: zip_archive.into_inner(),
            members,
            version,
            pre_release,
        })
    }

    pub fn version(&self) -> ([u32; 2], Option<&str>) {
        (self.version, self.pre_release.as_deref())
    }

    pub fn filenames(&self) -> impl Iterator<Item = &str> {
        self.members.keys().map(|s| &**s)
    }

    pub fn span(&self, name: &str) -> Result<FileSpan, Error> {
        self.members
            .get(name)
            .ok_or_else(|| Error::ZipMemberMissing(name.to_owned()))
            .copied()
    }

    pub fn open(&self, name: &str) -> Result<SubFile<R>, Error> {
        let span = self
            .members
            .get(name)
            .ok_or_else(|| Error::ZipMemberMissing(name.to_owned()))?;
        Ok(self.file.sub_file(span.offset, span.size)?)
    }
}

fn get_version(comment_bytes: &[u8]) -> Option<([u32; 2], Option<String>)> {
    let comment = std::str::from_utf8(comment_bytes).ok()?;
    let mut dash_parts = comment
        .strip_prefix(FORMAT_NAME)?
        .strip_prefix(' ')?
        .split('-');
    let main = dash_parts.next()?;
    let pre_release = dash_parts.next().map(ToOwned::to_owned);
    let mut version_parts = main.split('.');
    let major = version_parts.next()?.parse().ok()?;
    let minor = version_parts.next()?.parse().ok()?;
    if version_parts.next().is_some() {
        return None;
    }
    Some(([major, minor], pre_release))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions() {
        assert_eq!(
            get_version("Open Mining Format 2.0".as_bytes()),
            Some(([2, 0], None))
        );
        assert_eq!(
            get_version("Open Mining Format 2.0-alpha.1".as_bytes()),
            Some(([2, 0], Some("alpha.1".to_string())))
        );
        assert_eq!(get_version("Something else 1.0".as_bytes()), None);
        assert_eq!(get_version(b"Something not UTF-8 \xff"), None);
    }
}
