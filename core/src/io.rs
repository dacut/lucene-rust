use tokio::io::{AsyncRead, AsyncWrite};

mod crc32_reader;
mod directory;
mod encoding;
pub use {crc32_reader::*, directory::*, encoding::*};

/// Type alias for [AsyncRead] types that can also be [Unpin]ned.
pub trait AsyncReadUnpin: AsyncRead + Unpin {}
impl<T: AsyncRead + Unpin + ?Sized> AsyncReadUnpin for T {}

/// Type alias for [AsyncWrite] types that can also be [Unpin]ned.
pub trait AsyncWriteUnpin: AsyncWrite + Unpin {}
impl<T: AsyncWrite + Unpin + ?Sized> AsyncWriteUnpin for T {}
