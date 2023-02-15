use crate::index::{
    index_reader::IndexReader,
};

/// Instances of this reader type can only be used to get stored fields from the underlying
/// LeafReaders, but it is not possible to directly retrieve postings. To do that, get the
/// [LeafReaderContext] for all sub-readers via [::leaves].
///
/// IndexReader instances for indexes on disk are usually constructed with a call to one of the
/// static `DirectoryReader::open()` methods, e.g. [DirectoryReader::open_dir].
/// [DirectoryReader] implements the `CompositeReader` interface, it is not possible to
/// directly get postings.
///
/// Concrete subclasses of IndexReader are usually constructed with a call to one of the static
/// `open_*()` methods, e.g. [DirectoryReader::open_dir].
///
/// For efficiency, in this API documents are often referred to via _document numbers_,
/// non-negative integers which each name a unique document in the index. These document numbers are
/// ephemeral -- they may change as documents are added to and deleted from an index. Clients should
/// thus not rely on a given document having the same number between sessions.
///
/// # Note
/// [IndexReader] instances are completely thread safe, meaning multiple
/// threads can call any of its methods, concurrently.
pub trait CompositeReader: IndexReader {}

