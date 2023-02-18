use {
    std::{
        future::Future,
        pin::Pin,
        io::Result as IoResult,
    },
};

/// An async equivalent of the Lucene Java BytesRefIterator, returning a future of bytes.
pub trait BytesIterator {
    /// Increments the iteration to the next vector of bytes in the iterator. Returns the resulting
    /// [Vec<u8>] or `None` if the end of the iterator is reached.
    /// 
    /// After this method returns null, do not call it again: the results are undefined.
    ///
    /// # Returns
    /// The next [Vec<u8>] in the iterator or `None` if the end of the iterator is reached.
    /// 
    /// # Errors
    /// [IoError] If there is a low-level I/O error.
    fn next(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<Vec<u8>>>>>>;
}