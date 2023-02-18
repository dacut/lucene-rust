use {
    crate::index::impacts::Impacts,
    std::{future::Future, pin::Pin, io::Result as IoResult},
};

/// Source of [Impacts].
pub trait ImpactsSource {
    /// Shallow-advance to `target`. This is cheaper than calling [DocIdSetIterator::advance]
    /// and allows further calls to [::get_impacts] to ignore doc IDs that are less than `target`
    /// in order to get more precise information about impacts. This method may not be called on
    /// targets that are less than the current [DocIdSetIterator::doc_id]. After this method has
    /// been called, [DocIdSetIterator::next_doc] may not be called if the current doc ID is less
    /// than `target - 1` and [DocIdSetIterator::advance] may not be called on targets that are
    /// less than `target`.
    fn advance_shallow(self: Pin<Box<Self>>, target: u32) -> Pin<Box<dyn Future<Output = IoResult<()>>>>;

    /// Get information about upcoming impacts for doc ids that are greater than or equal to the
    /// maximum of [DocIdSetIterator::doc_id] and the last target that was passed to [::advance_shallow].
    /// This method may not be called on an unpositioned iterator on which
    /// [::advance_shallow] has never been called.
    /// 
    /// # Note
    /// Advancing this iterator may invalidate the returned impacts, so they should not be used
    /// after the iterator has been advanced.
    fn get_impacts(self: Pin<Box<Self>>) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn Impacts>>>>>>;
}
