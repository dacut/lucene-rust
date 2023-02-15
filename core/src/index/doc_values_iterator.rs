use {
    crate::search::doc_id_set_iterator::DocIdSetIterator,
    std::{
        future::Future,
        io::Result as IoResult,
        num::NonZeroU32,
        pin::Pin,
    },
};

pub trait DocValuesIterator: DocIdSetIterator {
    /// Advance the iterator to exactly `target` and return whether `target` has a value.
    /// `target` must be greater than or equal to the current doc id and must be
    /// a valid doc id, ie. `> 0` and `< maxDoc`. After this method returns, `.doc_id()`
    /// returns `Some(target)`.
    fn advance_exact(self: Pin<&mut Self>, target: NonZeroU32) -> Pin<Box<dyn Future<Output = IoResult<bool>>>>;
}
