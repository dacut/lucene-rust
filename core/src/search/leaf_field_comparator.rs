use {
    crate::search::{doc_id_set_iterator::DocIdSetIterator, scorable::Scorable},
    std::{cmp::Ordering, future::Future, io::Result as IoResult, pin::Pin},
};

/// Comparator that gets instantiated on each leaf from a top-level [FieldComparator]
/// instance.
///
/// A leaf comparator must define these functions:
///
/// * [LeafFieldComparator::set_bottom]: This method is called by [FieldValueHitQueue] to notify the
///   FieldComparator of the current weakest ("bottom") slot. Note that this slot may not hold
///   the weakest value according to your comparator, in cases where your comparator is not the
///   primary one (ie, is only used to break ties from the comparators before it).
/// * [LeafFieldComparator::compare_bottom]: Compare a new hit (docID) against the "weakest" (bottom)
///   entry in the queue.
/// * [LeafFieldComparator::compare_top]: Compare a new hit (docID) against the top value previously set
///   by a call to [FieldComparator::set_top_value].
/// * [LeafFieldComparator::copy]: Installs a new hit into the priority queue. The [FieldValueHitQueue]
///    calls this method when a new hit is competitive.
///
/// # See also
/// [FieldComparator]
pub trait LeafFieldComparator {
    /// Set the bottom slot, ie the "weakest" (sorted last) entry in the queue. When
    /// [LeafFieldComparator::compare_bottom] is called, you should compare against this slot. This
    /// will always be called before [LeafFieldComparator::compareBottom].
    ///
    /// # Parameters
    /// * `slot`: the currently weakest (sorted last) slot in the queue
    fn set_bottom(&mut self, slot: usize) -> IoResult<()>;

    /// Compare the bottom of the queue with this doc. This will only invoked after `set_bottom` has been
    /// called. This should return the same result as [FieldComparator::compare] as if
    /// bottom were slot1 and the new document were slot 2.
    ///
    /// For a search that hits many results, this method will be the hotspot (invoked by far the
    /// most frequently).
    ///
    /// # Parameters
    /// * `doc`: that was hit
    fn compare_bottom(&self, doc: i32) -> Pin<Box<dyn Future<Output = IoResult<Ordering>>>>;

    /// Compare the top value with this doc. This will only invoked after set_top_value has been called.
    /// This should return the same result as [FieldComparator::compare] as if top_value
    /// were slot1 and the new document were slot 2. This is only called for searches that use
    ///  search_after (deep paging).
    ///
    /// # Parameters
    /// * `doc`: that was hit
    fn compare_top(&self, doc: i32) -> Pin<Box<dyn Future<Output = IoResult<Ordering>>>>;

    /// This method is called when a new hit is competitive. You should copy any state associated with
    /// this document that will be required for future comparisons, into the specified slot.
    ///
    /// # Parameters
    /// * `slot`: which slot to copy the hit to
    /// * `doc`: docID relative to current reader
    fn copy(&mut self, slot: usize, doc: i32) -> IoResult<()>;

    /// Sets the Scorer to use in case a document's score is needed.
    ///
    /// # Parameters
    /// * `scorer`: Scorer instance that you should use to obtain the current hit's score, if
    ///    necessary.
    fn set_scorer(&mut self, scorer: Box<dyn Scorable>) -> IoResult<()>;

    /// Returns a competitive iterator
    ///
    /// # Returns
    /// An iterator over competitive docs that are stronger than already collected docs or
    /// `None` if such an iterator is not available for the current comparator or segment.
    fn competitve_iterator(&self) -> IoResult<Option<Box<dyn DocIdSetIterator>>> {
        Ok(None)
    }

    /// Informs this leaf comparator that hits threshold is reached. This method is called from a
    /// collector when hits threshold is reached.
    fn set_hits_threshold_reached(&mut self) -> IoResult<()> {
        Ok(())
    }
}
