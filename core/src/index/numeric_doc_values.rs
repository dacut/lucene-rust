use {
    crate::index::doc_values_iterator::DocValuesIterator,
    std::{
        future::Future,
        io::Result as IoResult,
        pin::Pin,
    }
};

pub trait NumericDocValues: DocValuesIterator {
    /// Returns the numeric value for the current document ID. It is illegal to call this method after
    /// [DocValuesIterator::advance_exact] returned `false`.
    fn long_value(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<i64>>>>;
}
