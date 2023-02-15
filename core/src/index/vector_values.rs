use {
    crate::index::doc_values_iterator::DocValuesIterator,
    std::{
        future::{ready, Future},
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
        pin::Pin,
    },
};

/// The maximum length of a vector.
pub const MAX_DIMENSIONS: usize = 1024;

pub trait VectorValues: DocValuesIterator {
    /// Returns the dimension of the vectors.
    fn dimension(self: Pin<&Self>) -> usize;

    /// Returns the number of vectors for this field.
    fn size(self: Pin<&Self>) -> usize;

    fn cost(self: Pin<&Self>) -> u64 {
        self.size() as u64
    }

    /// Return the vector value for the current document ID. It is illegal to call this method when the
    /// iterator is not positioned: before advancing, or after failing to advance.
    fn vector_value(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Vec<f32>>>>>;

    /// Return the binary encoded vector value for the current document ID. These are the bytes
    /// corresponding to the float array return by [::vector_value]. It is illegal to call this
    /// method when the iterator is not positioned: before advancing, or after failing to advance.
    fn binary_value(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Vec<u8>>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "binary_value is not supported"))))
    }
}
