use {
    crate::index::{
        index_reader_context::IndexReaderContext, leaf_reader_context::LeafReaderContext,
        term::Term, term_vectors::TermVectors,
        stored_fields::StoredFields,
    },
    std::{
        fmt::Debug,
        future::Future,
        hash::{Hash, Hasher},
        io::{Result as IoResult},
        pin::Pin,
        sync::{Arc},
    },
};

pub trait IndexReader: Debug {
    /// Returns a [TermVectors] reader for the term vectors of this index.
    ///
    /// # Example
    ///
    /// TopDocs hits = searcher.search(query, 10);
    /// TermVectors termVectors = reader.termVectors();
    /// for (ScoreDoc hit : hits.scoreDocs) {
    ///  Fields vector = termVectors.get(hit.doc);
    ///
    /// # Returns
    /// A result containing a [TermVectors] instance or an [IoError] if there is a low-level IO error
    fn term_vectors(self: Arc<Self>) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn TermVectors>>>>>>;

    /// Returns the number of documents in this index.
    ///
    /// # Note
    /// This operation may run in `O(maxDoc)`. Implementations that can't return this
    /// number in constant-time should cache it.
    fn num_docs(self: Arc<Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>>;

    /// Returns one greater than the largest possible document number. This may be used to, e.g.,
    /// determine how big to allocate an array which will have an element for every document number in
    /// an index.
    fn max_doc(self: Arc<Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>>;

    /// Returns the number of deleted documents.
    ///
    /// # Note
    ///  This operation may run in `O(maxDoc)`.
    fn num_deleted_docs(self: Arc<Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        let this = self;
        Box::pin(async move {
            index_reader_num_deleted_docs(this).await
        })
    }

    /// Returns a [StoredFields] reader for the stored fields of this index.
    ///
    fn stored_fields(self: Arc<Self>) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn StoredFields>>>>>>;

    /// Returns true if any documents have been deleted. Implementers should consider overriding this
    /// method if [max_doc] or [num_docs] are not constant-time operations.
    fn has_deletions(self: Arc<Self>) -> Pin<Box<dyn Future<Output = IoResult<bool>>>> {
        let this = self;
        Box::pin(async move {
            index_reader_has_deletions(this).await
        })
    }

    /// Expert: Returns the root [IndexReaderContext] for this [IndexReader]'s sub-reader
    /// tree.
    ///
    /// Iff this reader is composed of sub readers, i.e. this reader being a composite reader, this
    /// method returns a {@link CompositeReaderContext} holding the reader's direct children as well as
    /// a view of the reader tree's atomic leaf contexts. All sub-[IndexReaderContext] instances
    /// referenced from this readers top-level context are private to this reader and are not shared
    /// with another context tree. For example, IndexSearcher uses this API to drive searching by one
    /// atomic leaf reader at a time. If this reader is not composed of child readers, this method
    /// returns an {@link LeafReaderContext}.
    ///
    /// Note: Any of the sub-[CompositeReaderContext] instances referenced from this top-level
    /// context do not support [CompositeReaderContext::leaves]. Only the top-level context
    /// maintains the convenience leaf-view for performance reasons.
    fn get_context(self: Arc<Self>) -> Arc<dyn IndexReaderContext>;

    /// Returns the reader's leaves, or itself if this reader is atomic. This is a convenience method calling
    /// get_context().leaves().
    fn leaves(self: Arc<Self>) -> IoResult<Vec<Arc<LeafReaderContext>>> {
        self.get_context().leaves()
    }

    /// Returns the number of documents containing the `term`. This method returns 0 if the
    /// term or field does not exists. This method does not take into account deleted documents that
    /// have not yet been merged away.
    fn doc_freq(self: Arc<Self>, term: Term) -> Pin<Box<dyn Future<Output = IoResult<usize>>>>;

    /// Returns the total number of occurrences of {@code term} across all documents (the sum of the
    /// freq() for each doc that has this term). Note that, like other term measures, this measure does
    /// not take deleted documents into account.
    fn total_term_freq(self: Arc<Self>, term: Term) -> Pin<Box<dyn Future<Output = IoResult<usize>>>>;

    /// Returns the sum of {@link TermsEnum#docFreq()} for all terms in this field. Note that, just
    /// like other term measures, this measure does not take deleted documents into account.
    fn get_sum_doc_freq(self: Arc<Self>, field: &str) -> Pin<Box<dyn Future<Output = IoResult<usize>>>>;

    /// Returns the number of documents that have at least one term for this field. Note that, just
    /// like other term measures, this measure does not take deleted documents into account.
    fn get_doc_count(self: Arc<Self>, field: &str) -> Pin<Box<dyn Future<Output = IoResult<usize>>>>;

    /// Returns the sum of {@link TermsEnum#totalTermFreq} for all terms in this field. Note that, just
    /// like other term measures, this measure does not take deleted documents into account.
    fn get_sum_total_term_freq(self: Arc<Self>, field: &str) -> Pin<Box<dyn Future<Output = IoResult<usize>>>>;
}

async fn index_reader_num_deleted_docs<IR>(this: Arc<IR>) -> IoResult<usize>
where
    IR: IndexReader + ?Sized
{
    Ok(this.clone().max_doc().await? - this.num_docs().await?)
}

async fn index_reader_has_deletions<IR>(this: Arc<IR>) -> IoResult<bool>
where
    IR: IndexReader + ?Sized
{
    Ok(this.num_deleted_docs().await? > 0)
}

/// A utility class that gives hooks in order to help build a cache based on the data that is
/// contained in this index.
/// 
/// # Example
/// 
/// Cache the number of documents that match a query per reader.
/// 
/// TODO: Convert this to Rust.
/// ```ignore
/// public class QueryCountCache {
/// 
///   private final Query query;
///   private final Map<IndexReader.CacheKey, Integer> counts = new ConcurrentHashMap<>();
/// 
///   // Create a cache of query counts for the given query
///   public QueryCountCache(Query query) {
///     this.query = query;
///   }
/// 
///   // Count the number of matches of the query on the given IndexSearcher
///   public int count(IndexSearcher searcher) throws IOException {
///     IndexReader.CacheHelper cacheHelper = searcher.getIndexReader().getReaderCacheHelper();
///     if (cacheHelper == null) {
///       // reader doesn't support caching
///       return searcher.count(query);
///     } else {
///       // make sure the cache entry is cleared when the reader is closed
///       cacheHelper.addClosedListener(counts::remove);
///       return counts.computeIfAbsent(cacheHelper.getKey(), cacheKey ->; {
///         try {
///           return searcher.count(query);
///         } catch (IOException e) {
///           throw new UncheckedIOException(e);
///         }
///       });
///     }
///   }
/// 
/// }
/// ```
/// 
pub trait CacheHelper {
    /// Get a key that the resource can be cached on.
    fn get_key(&self) -> CacheKey;

    /// Add a [ClosedListener] which will be called when the resource guarded by [CacheHelper::get_key] is closed.
    fn add_closed_listener(&mut self, listener: Box<dyn ClosedListener>);
}  

/// A cache key identifying a resource that is being cached on.
/// 
/// This differs from the Java implementation in that the interior is used to store uniqueness, and the
/// CacheKey itself can be cloned and compared at will.
#[derive(Debug, Clone)]
pub struct CacheKey {
    inner: Arc<CacheKeyInner>,
}

impl CacheKey {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(CacheKeyInner),
        }
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for CacheKey {}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.inner).hash(state)
    }
}

#[derive(Debug)]
pub struct CacheKeyInner;

/// A listener that is called when a resource gets closed.
pub trait ClosedListener {
    /// Invoked when the resource (segment core or index reader) that is being cached on is closed.
    fn on_close(self: Pin<&Self>, key: CacheKey) -> Pin<Box<dyn Future<Output = IoResult<()>>>>;
}
