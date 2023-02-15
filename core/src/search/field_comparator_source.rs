use {crate::search::field_comparator::FieldComparator, std::fmt::Debug};

/// Provides a [FieldCOmparator] for custom field sorting.
pub trait FieldComparatorSource: Debug {
    /// Creates a comparator for the field in the given index
    fn new_comparator(&self, field_name: &str, num_hits: usize, enable_skipping: bool, reversed: bool) -> Box<dyn FieldComparator>;
}
