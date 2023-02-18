use {
    crc32fast::Hasher,
    std::{
        cmp::min,
        fmt::{Debug, Formatter, Result as FmtResult},
        io::{IoSliceMut, Read, Result as IoResult},
    },
};

pub struct Crc32Reader<T> {
    wrapped: T,
    digest: Hasher,
}

impl<T> Crc32Reader<T> {
    pub fn new(wrapped: T) -> Self {
        Self {
            wrapped,
            digest: Hasher::new(),
        }
    }

    pub fn digest(&self) -> u32 {
        self.digest.clone().finalize()
    }
}

impl<T> Clone for Crc32Reader<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            wrapped: self.wrapped.clone(),
            digest: self.digest.clone(),
        }
    }
}

impl<T> Debug for Crc32Reader<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Crc32Reader").field("wrapped", &self.wrapped).field("digest", &self.digest).finish()
    }
}

impl<T> Read for Crc32Reader<T>
where
    T: Read,
{
    #[cfg(feature = "can_vector")]
    fn is_read_vectored(&self) -> bool {
        self.wrapped.is_read_vectored()
    }

    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let n_read = self.wrapped.read(buf)?;
        self.digest.update(&buf[..n_read]);
        Ok(n_read)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> IoResult<()> {
        self.wrapped.read_exact(buf)?;
        self.digest.update(buf);
        Ok(())
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> IoResult<usize> {
        let start_len = buf.len();
        let n_read = self.wrapped.read_to_end(buf)?;
        self.digest.update(&buf[start_len..]);
        Ok(n_read)
    }

    fn read_to_string(&mut self, buf: &mut String) -> IoResult<usize> {
        let start_len = buf.len();
        let n_read = self.wrapped.read_to_string(buf)?;
        self.digest.update(buf[start_len..].as_bytes());
        Ok(n_read)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> IoResult<usize> {
        let n_read = self.wrapped.read_vectored(bufs)?;
        let mut to_update = n_read;
        let mut bufs_iter = bufs.iter();

        // Iterate through the buffers; the last one might be partially filled.
        while to_update > 0 {
            // This should never panic because we know that to_update is less than n_read.
            let buf: &[u8] = bufs_iter.next().unwrap();
            let buf_size = min(buf.len(), to_update);
            self.digest.update(&buf[..buf_size]);
            to_update -= buf_size;
        }

        Ok(n_read)
    }
}
