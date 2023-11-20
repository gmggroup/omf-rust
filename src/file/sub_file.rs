use std::io::{Read, Seek, SeekFrom};

/// A seek-able sub-file with a start and end point within a larger file.
pub struct SubFile<T> {
    inner: T,
    offset: u64,
    position: u64,
    limit: u64,
}

impl<T: Seek> SubFile<T> {
    /// Creates a sub-file from seek-able object.
    ///
    /// This new file will its start and zero position at the current position of `inner` and
    /// extend up to `limit` bytes.
    pub fn new(mut inner: T, limit: u64) -> std::io::Result<Self> {
        Ok(Self {
            position: 0,
            offset: inner.stream_position()?,
            inner,
            limit,
        })
    }

    /// Returns the total length of the sub-file, ignoring the current position.
    pub fn len(&self) -> u64 {
        self.limit
    }

    /// Returns true if the file is empty.
    pub fn is_empty(&self) -> bool {
        self.limit == 0
    }
}

impl<T: Read> Read for SubFile<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.position == self.limit {
            return Ok(0);
        }
        let max = (buf.len() as u64).min(self.limit - self.position) as usize;
        let n = self.inner.read(&mut buf[..max])?;
        assert!(
            self.position + (n as u64) <= self.limit,
            "number of read bytes exceeds limit"
        );
        self.position += n as u64;
        Ok(n)
    }
}

impl<T: Seek> Seek for SubFile<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_position = match pos {
            SeekFrom::Start(pos) => pos as i64,
            SeekFrom::End(delta) => self.limit as i64 + delta,
            SeekFrom::Current(delta) => self.position as i64 + delta,
        };
        if new_position < 0 {
            return Err(std::io::ErrorKind::InvalidInput.into());
        }
        self.position = new_position as u64;
        self.inner
            .seek(SeekFrom::Start(self.offset + self.position))?;
        Ok(self.position)
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        Ok(self.position)
    }
}

#[cfg(feature = "parquet")]
impl parquet::file::reader::Length for SubFile<std::fs::File> {
    fn len(&self) -> u64 {
        self.limit
    }
}

#[cfg(feature = "parquet")]
impl parquet::file::reader::ChunkReader for SubFile<std::fs::File> {
    type T = <std::fs::File as parquet::file::reader::ChunkReader>::T;

    fn get_read(&self, start: u64) -> parquet::errors::Result<Self::T> {
        self.inner.get_read(self.offset.saturating_add(start))
    }

    fn get_bytes(&self, start: u64, length: usize) -> parquet::errors::Result<bytes::Bytes> {
        self.inner
            .get_bytes(self.offset.saturating_add(start), length)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn subfile() {
        let data = b"0123456789";
        let mut base = Cursor::new(data);
        base.seek(SeekFrom::Start(2)).unwrap();
        let mut t = SubFile::new(base, 6).unwrap();
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
