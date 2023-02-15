use {
    crate::{
        index::leaf_reader_context::LeafReaderContext,
        search::{leaf_field_comparator::LeafFieldComparator, sort_field::CustomField},
    },
    std::{cmp::Ordering, future::Future, io::Result as IoResult, pin::Pin},
};

/// A FieldComparator compares hits so as to determine their sort order when collecting the
/// top results with [TopFieldCollector]. The concrete public FieldComparator classes here
/// correspond to the SortField types.
///
/// The document IDs passed to these methods must only move forwards, since they are using doc
/// values iterators to retrieve sort values.
///
/// This API is designed to achieve high performance sorting, by exposing a tight interaction with
/// [FieldValueHitQueue] as it visits hits. Whenever a hit is competitive, it's enrolled into a
/// virtual slot, which is an int ranging from 0 to _num_hits_ - 1. Segment transitions are handled by
/// creating a dedicated per-segment [LeafFieldComparator] which also needs to interact with
/// the [FieldValueHitQueue] but can optimize based on the segment to collect.
///
/// The following functions need to be implemented
///
/// * [FieldComparator::compare]: Compare a hit at 'slot a' with hit 'slot b'.
/// * [FieldComparator::set_top_value]: This method is called by [TopFieldCollector] to notify the
///   FieldComparator of the top most value, which is used by future calls to
///   [LeafFieldComparator::compare_top].
/// * [FieldComparator::get_leaf_comparator]: Invoked when the search is switching to the next
///    segment. You may need to update internal state of the comparator, for example retrieving
///    new values from DocValues.
/// * [FieldComparator::value] Return the sort value stored in the specified slot. This is only called at
///   the end of the search, in order to populate [FieldDoc::fields] when returning the top results.
///
/// # See
/// [LeafFieldComparator]
pub trait FieldComparator {
    /// Compare hit at slot1 with hit at slot2.
    ///
    /// # Parameters
    /// * `first`: slot to compare
    /// * `second`: slot to compare
    ///
    /// # Returns
    /// An Ordering indicating the relative order of the two slots.
    fn compare(&self, slot1: usize, slot2: usize) -> Ordering;

    /// Record the top value, for future calls to [LeafFieldComparator::compare_top]. This is only
    /// called for searches that use search_after (deep paging), and is called before any calls to
    /// [FieldComparator::get_leaf_comparator].
    fn set_top_value(&self, value: Box<dyn CustomField>);

    /// Return the actual value in the slot.
    ///
    /// # Parameters
    /// * `slot`: the slot to return the value for
    ///
    /// # Returns
    /// The value in the specified slot.
    fn value(&self, slot: usize) -> Box<dyn CustomField>;

    /// Get a per-segment [LeafFieldComparator] to collect the given [LeafReaderContext]. All
    /// docIDs supplied to this [LeafFieldComparator] are relative to the current reader (you must add
    /// docBase if you need to map it to a top-level docID).
    ///
    /// # Parameters
    /// * `context`: current reader context
    ///
    /// # Returns
    /// The comparator to use for this segment
    fn get_leaf_comparator(
        &self,
        ctx: &LeafReaderContext,
    ) -> Pin<Box<dyn Future<Output = IoResult<Box<dyn LeafFieldComparator>>>>>;

    /// Compares two values.
    fn compare_values(&self, first: Option<Box<dyn CustomField>>, second: Option<Box<dyn CustomField>>) -> Ordering {
        match (first, second) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(first), Some(second)) => first.cmp(second),
        }
    }

    /// Informs the comparator that sort is done on this single field. This is useful to enable some
    /// optimizations for skipping non-competitive documents.
    fn set_single_sort(&mut self) {}

    /// Informs the comparator that the skipping of documents should be disabled. This function is
    /// called by TopFieldCollector in cases when the skipping functionality should not be applied or
    /// not necessary. An example could be when search sort is a part of the index sort, and can be
    /// already efficiently handled by TopFieldCollector, and doing extra work for skipping in the
    /// comparator is redundant.
    fn disableSkipping(&mut self) {}
}
