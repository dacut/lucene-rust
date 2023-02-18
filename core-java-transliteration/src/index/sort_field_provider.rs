use {
    crate::search::sort_field::SortField,
    std::{future::Future, io::Result as IoResult, pin::Pin},
    tokio::io::{AsyncRead, AsyncWrite},
};

pub trait SortFieldProvider {
    /// The name this SortFieldProvider is registered under.
    fn get_name(&self) -> &str;

    /// Reads a SortField from serialized bytes.
    fn read_sort_field<R>(
        self: Pin<&Self>,
        input: Pin<&mut R>,
    ) -> Pin<Box<dyn Future<Output = IoResult<Box<dyn SortField>>>>>
    where
        R: AsyncRead;

    /// Writes a SortField to serialized bytes.
    fn write_sort_field<W>(
        self: Pin<&Self>,
        field: Pin<Box<dyn SortField>>,
        output: Pin<&mut W>,
    ) -> Pin<Box<dyn Future<Output = IoResult<()>>>>
    where
        W: AsyncWrite;
}
