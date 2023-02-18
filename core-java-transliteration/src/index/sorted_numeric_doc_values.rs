use {
    crate::index::doc_values_iterator::DocValuesIterator,
    std::{
        future::Future,
        io::Result as IoResult,
        pin::Pin,
    }
};

pub trait SortedNumericDocValues: DocValuesIterator {
    /// Iterates to the next value in the current document. Do not call this more than
    /// [::doc_value_count] times for the document.
    fn next_value(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<i64>>>>;
     
    /// Retrieves the number of values for the current document. This must always be greater than zero.
    /// It is illegal to call this method after [DocValuesIterator::advance_exact] returned `false`.
    fn doc_value_count(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>>;
}
