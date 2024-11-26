/// Read from a file-like object at an offset.
pub trait ReadAt: Send + Sync + 'static {
    /// Seeks to `offset` and performs reads into `buf`.
    ///
    /// Returns the number of bytes read.
    ///
    /// Note that similar to File::read, it is not an error to return with a short read.
    /// May or may not move any underlying file pointer, even if the read fails or is short.
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;

    /// Returns the length of the data.
    fn size(&self) -> std::io::Result<u64>;
}

#[cfg(windows)]
impl ReadAt for std::fs::File {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        use std::os::windows::fs::FileExt;
        self.seek_read(buf, offset)
    }

    fn size(&self) -> std::io::Result<u64> {
        self.metadata().map(|m| m.len())
    }
}

#[cfg(unix)]
impl ReadAt for std::fs::File {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        use std::os::unix::fs::FileExt;
        file.read_at(buf, offset)
    }

    fn len(&self) -> std::io::Result<u64> {
        self.metadata().map(|m| m.len())
    }
}

impl ReadAt for Vec<u8> {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        let start = usize::try_from(offset).expect("offset must fit in usize");
        let end = start.saturating_add(buf.len()).min(self.len());
        let slice = &self[start..end];
        buf[..slice.len()].copy_from_slice(slice);
        Ok(slice.len())
    }

    fn size(&self) -> std::io::Result<u64> {
        Ok(self.len().try_into().expect("length must fit in u64"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_at_vec_middle() {
        let data = vec![1, 2, 3, 4, 5];
        let mut buf = [0; 3];
        assert_eq!(data.read_at(&mut buf, 1).unwrap(), 3);
        assert_eq!(&buf, &[2, 3, 4]);
    }

    #[test]
    fn read_at_vec_end() {
        let data = vec![1, 2, 3, 4, 5];
        let mut buf = [0; 3];
        assert_eq!(data.read_at(&mut buf, 3).unwrap(), 2);
        assert_eq!(&buf, &[4, 5, 0]);
    }
}
