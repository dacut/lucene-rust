use {
    crate::index::terms::Terms,
    std::{future::Future, io::Result as IoResult, pin::Pin},
};

/// Provides a [Terms] index for fields that have it, and lists which fields do. This is
/// primarily an internal/experimental API (see [FieldsProducer]), although it is also used to
/// expose the set of term vectors per document.
pub trait Fields {
    /// Returns an iterator that will step through all fields names.
    fn iter(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Box<dyn Iterator<Item = String>>>>>>;

    /// Get the [Terms] for this field. This will return `None` if the field does not exist.
    fn terms(self: Pin<&Self>, field: &str) -> Pin<Box<dyn Future<Output = IoResult<Option<Pin<Box<dyn Terms>>>>>>>;

    /// Returns the number of fields or `None` if the number of distinct field names is unknown. If <=
    /// 0, [::into_iter] will return as many field names.
    fn size(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<usize>>>>>;
}
