use crate::{search::sort::Sort, util::version::Version};

#[derive(Debug)]
pub struct LeafMetaData {
    created_version_major: i32,
    min_version: Option<Version>,
    sort: Option<Sort>,
}

impl LeafMetaData {
    pub fn new(created_version_major: i32, min_version: Option<Version>, sort: Option<Sort>) -> Self {
        Self {
            created_version_major,
            min_version,
            sort,
        }
    }

    /// Get the Lucene version that created this index. This can be used to implement backward
    /// compatibility on top of the codec API. A return value of `6` indicates that the created
    /// version is unknown.
    #[inline]
    pub fn get_created_version_major(&self) -> i32 {
        self.created_version_major
    }

    /// Return the minimum Lucene version that contributed documents to this index, or `None` if
    /// this information is not available.
    #[inline]
    pub fn get_min_version(&self) -> Option<Version> {
        self.min_version
    }

    /// Return the order in which documents from this index are sorted, or `None` if documents
    /// are in no particular order.
    #[inline]
    pub fn get_sort(&self) -> Option<Sort> {
        self.sort
    }
}
