use {
    crc32fast::Hasher,
    pin_project::pin_project,
    std::{
        fmt::{Debug, Formatter, Result as FmtResult},
        io::Result as IoResult,
        pin::Pin,
        task::{Context, Poll},
    },
    tokio::io::{AsyncRead, ReadBuf},
};

/// A wrapper around an `AsyncRead` that computes the CRC32 of the data read.
#[pin_project]
pub struct Crc32Reader<T> {
    #[pin]
    wrapped: T,
    digest: Hasher,
}

impl<T> Crc32Reader<T> {
    /// Creates a new `Crc32Reader` that wraps the given [AsyncRead].
    pub fn new(wrapped: T) -> Self {
        Self {
            wrapped,
            digest: Hasher::new(),
        }
    }

    /// Returns the CRC32 of the data read so far.
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

impl<T: AsyncRead> AsyncRead for Crc32Reader<T> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<IoResult<()>> {
        let this = self.project();

        match this.wrapped.poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                this.digest.update(buf.filled());
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}
