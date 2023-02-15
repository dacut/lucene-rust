use {
    crate::{
        search::{
            index_searcher::IndexSearcher,
            sort_field::{SortField, FIELD_DOC, FIELD_SCORE},
        },
    },
    std::io::Result as IoResult,
};

/// Encapsulates sort criteria for returned hits.
///
/// A `Sort` can be created with an empty constructor, yielding an object that will instruct
/// searches to return their hits sorted by relevance; or it can be created with one or more [SortField]s.
#[derive(Debug)]
pub struct Sort {
    fields: Vec<Box<dyn SortField>>,
}

impl Sort {
    /// Sets the sort to the given criteria in succession: the first SortField is checked first, but if
    /// it produces a tie, then the second SortField is used to break the tie, etc. Finally, if there
    /// is still a tie after all SortFields are checked, the internal Lucene docid is used to break it.
    ///
    /// # Panics
    /// Panics if `fields` is empty.
    pub fn new(fields: Vec<Box<dyn SortField>>) -> Self {
        if fields.is_empty() {
            panic!("Sort must contain at least one field")
        }

        Self {
            fields,
        }
    }

    /// Represents sorting by computed relevance. Using this sort criteria returns the same results as
    /// calling [IndexSearcher::search] without a sort criteria,
    /// only with slightly more overhead.
    pub fn relevance() -> Self {
        Self {
            fields: vec![],
        }
    }

    /// Represents sorting by index order.
    pub fn index_order() -> Self {
        Self::new(vec![FIELD_DOC])
    }

    /// Sorts by computed relevance. This is the same sort criteria as calling [IndexSearcher::search] without a sort criteria, only with
    /// slightly more overhead.
    pub fn computed_relevance() -> Self {
        Self::new(vec![FIELD_SCORE])
    }

    /// Returns the sort criteria.
    pub fn get_sort(&self) -> &[Box<dyn SortField>] {
        &self.fields
    }

    /// Rewrites the SortFields in this Sort, returning a new Sort if any of the fields changes during
    /// their rewriting.
    ///
    /// # Parameters
    /// `searcher`: [IndexSearcher] to use in the rewriting
    fn rewrite(&self, searcher: &IndexSearcher) -> IoResult<Sort> {
        let mut rewritten_sort_fields = Vec::with_capacity(self.fields.len());

        for field in self.fields {
            field.rewrite(searcher)?;
            rewritten_sort_fields.push(field);
        }

        Ok(Sort::new(rewritten_sort_fields))
    }
}
