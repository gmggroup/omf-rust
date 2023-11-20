use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
};

use flate2::bufread::ZlibDecoder;

use crate::{
    error::{Error, Limit},
    file::SubFile,
};

use super::{
    model::{FromModel, Model},
    objects::{Array, Image, Key, Project},
    Omf1Error,
};

/// The OMF1 file loader.
#[derive(Debug)]
pub struct Omf1Reader {
    file: File,
    project: Key<Project>,
    models: HashMap<String, Model>,
    version: String,
}

impl Omf1Reader {
    pub fn new(mut file: File, limit: Option<u64>) -> Result<Self, Error> {
        file.rewind()?;
        let (project, json_start, version) = read_header(&mut file)?;
        let stream_len = file.seek(SeekFrom::End(0))?;
        if let Some(lim) = limit {
            if stream_len.saturating_sub(json_start) > lim {
                return Err(Error::LimitExceeded(Limit::JsonBytes));
            }
        }
        file.seek(SeekFrom::Start(json_start))?;
        let models: HashMap<String, Model> =
            serde_json::from_reader(&mut file).map_err(Omf1Error::DeserializationFailed)?;
        Ok(Self {
            file,
            project,
            models,
            version,
        })
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn model<T>(&self, key: &Key<T>) -> Result<T::Output<'_>, Error>
    where
        T: FromModel,
    {
        match self.models.get(&key.value) {
            Some(model) => Ok(T::from_model(model)?),
            None => Err(Omf1Error::MissingItem {
                key: key.value.to_owned(),
            }
            .into()),
        }
    }

    pub fn project(&self) -> Result<&Project, Error> {
        self.model(&self.project)
    }

    pub fn image(&self, array: &Image) -> Result<impl Read, Error> {
        let mut f = self.file.try_clone()?;
        f.seek(SeekFrom::Start(array.start))?;
        Ok(ZlibDecoder::new(BufReader::new(SubFile::new(
            f,
            array.length,
        )?)))
    }

    pub fn array_decompressed_bytes(
        &self,
        array: &Array,
    ) -> Result<impl Iterator<Item = Result<u8, std::io::Error>>, Error> {
        let mut f = self.file.try_clone()?;
        f.seek(SeekFrom::Start(array.start))?;
        Ok(ZlibDecoder::new(BufReader::new(SubFile::new(f, array.length)?)).bytes())
    }
}

fn read_header(read: &mut impl Read) -> Result<(Key<Project>, u64, String), Error> {
    const MAGIC: [u8; 4] = [0x84, 0x83, 0x82, 0x81];
    const SUPPORTED_VERSION: &str = "OMF-v0.9.0";

    let mut header = [0_u8; 60];
    match read.read_exact(&mut header) {
        Ok(_) => (),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Err(Omf1Error::NotOmf1.into())
        }
        Err(e) => return Err(e.into()),
    };
    if header[..4] != MAGIC {
        return Err(Omf1Error::NotOmf1.into());
    }
    let version = String::from_utf8_lossy(&header[4..36])
        .trim_end_matches('\0')
        .to_owned();
    if !version.starts_with("OMF-") {
        return Err(Omf1Error::NotOmf1.into());
    }
    if version != SUPPORTED_VERSION {
        return Err(Omf1Error::UnsupportedVersion { version }.into());
    }
    let project = Key::from_bytes((&header[36..52]).try_into().expect("16 bytes"));
    let json_start = u64::from_le_bytes((&header[52..60]).try_into().expect("8 bytes"));
    Ok((project, json_start, version))
}
