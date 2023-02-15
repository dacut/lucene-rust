use std::{future::Future, io::Result as IoResult, num::NonZeroU32, pin::Pin};

pub const NO_MORE_DOCS: NonZeroU32 = i32::MAX as u32;

/// This abstract class defines methods to iterate over a set of non-decreasing doc ids. Note that
/// this class assumes it iterates on doc Ids, and therefore [NO_MORE_DOCS] is set to `i32::MAX`
/// (for compatibility with Java) in order to be used as a sentinel object. Implementations of this
/// class are expected to consider `i32::MAX` as an invalid value.
pub trait DocIdSetIterator {
    /// Returns the following:
    ///
    /// * `None` if [::next_doc] or [::advance] were not called yet.
    /// * [NoMoreDocs] if the iterator has exhausted.
    /// * Otherwise it should return the doc ID it is currently on.
    ///
    /// # Since
    /// 2.9
    fn doc_id(self: Pin<&Self>) -> Option<NonZeroU32>;

    /// Advances to the next document in the set and returns the doc it is currently on, or `None`
    /// if there are no more docs in the set.
    ///
    /// # Note
    /// After the iterator has exhausted you should not call this method, as it may result in
    /// unpredicted behavior.
    ///
    /// # Since
    /// 2.9
    fn next_doc(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<NonZeroU32>>>>>;

    /// Advances to the first beyond the current whose document number is greater than or equal to
    /// _target_, and returns the document number itself. Exhausts the iterator and returns
    /// `None` if _target_ is greater than the highest document number in the set.
    ///
    /// The behavior of this method is **undefined** when called with `target < current`,
    /// or after the iterator has exhausted. Both cases may result in unpredicted behavior.
    ///
    /// When `target > current` it behaves as if written:
    ///
    /// ```
    /// fn advance(self: Pin<Box<&mut Self>>) -> Pin<Box<dyn Future<Output = IoResult<Option<NonZeroU32>>>>> {
    ///     let this = self;
    ///     Box::pin(async move {
    ///         let mut doc = self.next_doc();
    ///         while doc < target {
    ///             doc = self.next_doc();
    ///         }
    ///
    ///         Ok(doc)
    ///     })
    /// }
    /// ```
    ///
    /// Some implementations are considerably more efficient than that.
    ///
    /// # Note
    /// This method may be called with `None` for efficiency by some
    /// Scorers. If your implementation cannot efficiently determine that it should exhaust, it is
    /// recommended that you check for that value in each call to this method.
    ///
    /// # Since
    /// 2.9
    fn advance(self: Pin<&mut Self>, target: Option<NonZeroU32>) -> Pin<Box<dyn Future<Output = IoResult<Option<NonZeroU32>>>>>;

    /// Slow (linear) implementation of [DocIdSetIterator::advance] relying on [DocIdSetIterator::next_doc()] to advance
    /// beyond the target position.
    fn slow_advance(self: Pin<&mut Self>, target: NonZeroU32) -> Pin<Box<dyn Future<Output = IoResult<Option<NonZeroU32>>>>> {
        let doc_id = self.as_ref().doc_id();
        assert!(doc_id.is_some());
        assert!(doc_id.unwrap() < target);

        let this = self;
        Box::pin(async move { doc_id_set_iterator_slow_advance(this, target).await })
    }

    /// Returns the estimated cost of this [DocIdSetIterator].
    ///
    /// This is generally an upper bound of the number of documents this iterator might match, but
    /// may be a rough heuristic, hardcoded value, or otherwise completely inaccurate.
    fn cost(self: Pin<&Self>) -> u64;
}

async fn doc_id_set_iterator_slow_advance<D>(mut this: Pin<&mut D>, target: NonZeroU32) -> IoResult<Option<NonZeroU32>>
where
    D: DocIdSetIterator + ?Sized,
{
    loop {
        match this.as_mut().next_doc().await? {
            None => return Ok(None),
            Some(doc) => {
                if doc >= target {
                    return Ok(Some(doc));
                }
            }
        }
    }
}
