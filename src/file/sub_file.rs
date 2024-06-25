use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    sync::Arc,
};

/// A seek-able sub-file with a start and end point within a larger file.
pub struct SubFile {
    inner: Arc<File>,
    /// Start of the sub-file within `inner`.
    start: u64,
    /// The current file cursor position within the sub-file.
    position: u64,
    /// The length of the sub-file in bytes.
    len: u64,
}

impl SubFile {
    /// Creates a sub-file from seek-able object.
    ///
    /// This new file will its start and zero position at the current position of `inner` and
    /// extend up to `len` bytes.
    pub fn new(inner: Arc<File>, start: u64, len: u64) -> std::io::Result<Self> {
        start
            .checked_add(len)
            .expect("start + len should not overflow");
        Ok(Self {
            start,
            inner,
            position: 0,
            len,
        })
    }

    /// Returns the total length of the sub-file, ignoring the current position.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Returns true if the file is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of bytes remaining in the sub-file.
    fn remaining(&self) -> u64 {
        self.len.saturating_sub(self.position)
    }
}

impl Read for SubFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.position >= self.len {
            return Ok(0);
        }
        let limit = usize::try_from((buf.len() as u64).min(self.remaining())).expect("valid limit");
        let n = read_at(
            self.inner.as_ref(),
            &mut buf[..limit],
            self.start + self.position,
        )?;
        self.position += n as u64;
        Ok(n)
    }
}

impl Seek for SubFile {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_position = match pos {
            SeekFrom::Start(pos) => pos as i64,
            SeekFrom::End(delta) => (self.len as i64).saturating_add(delta),
            SeekFrom::Current(delta) => (self.position as i64).saturating_add(delta),
        };
        self.position =
            u64::try_from(new_position).map_err(|_| std::io::ErrorKind::InvalidInput)?;
        Ok(self.position)
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        Ok(self.position)
    }
}

#[cfg(feature = "parquet")]
impl parquet::file::reader::Length for SubFile {
    fn len(&self) -> u64 {
        self.len
    }
}

#[cfg(feature = "parquet")]
impl parquet::file::reader::ChunkReader for SubFile {
    type T = <std::fs::File as parquet::file::reader::ChunkReader>::T;

    fn get_read(&self, start: u64) -> parquet::errors::Result<Self::T> {
        self.inner.get_read(self.start.saturating_add(start))
    }

    fn get_bytes(&self, start: u64, length: usize) -> parquet::errors::Result<bytes::Bytes> {
        self.inner
            .get_bytes(self.start.saturating_add(start), length)
    }
}

/// Reads from a file at a specific offset
///
/// The Windows implementation moves the file cursor, which the Unix one doesn't,
/// so this should only be used from code that doesn't care about the file cursor.
#[cfg(windows)]
fn read_at(file: &std::fs::File, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
    use std::os::windows::fs::FileExt;
    file.seek_read(buf, offset)
}

/// Reads from a file at a specific offset
#[cfg(unix)]
fn read_at(file: &std::fs::File, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
    use std::os::unix::fs::FileExt;
    file.read_at(buf, offset)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn subfile() {
        let path = Path::new("./target/tmp/subfile.txt");
        std::fs::write(path, b"0123456789").unwrap();
        let base = Arc::new(File::open(path).unwrap());
        let mut t = SubFile::new(base.clone(), 2, 6).unwrap();
        let mut buf = [0; 5];
        t.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"23456");
        let mut buf = [0; 2];
        t.seek(SeekFrom::Current(-2)).unwrap();
        t.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"56");
        t.seek(SeekFrom::Current(-3)).unwrap();
        t.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"45");
        t.seek(SeekFrom::Start(0)).unwrap();
        t.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"23");
        let mut buf = [0; 10];
        let e = t.read_exact(&mut buf).unwrap_err();
        assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof);
        let e = t.seek(SeekFrom::End(-10)).unwrap_err();
        assert_eq!(e.kind(), std::io::ErrorKind::InvalidInput);
    }
}
