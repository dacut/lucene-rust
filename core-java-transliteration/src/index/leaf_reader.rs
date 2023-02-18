use {
    crate::{
        index::{
            binary_doc_values::BinaryDocValues,
            field_infos::FieldInfos,
            index_reader::IndexReader,
            leaf_meta_data::LeafMetaData,
            leaf_reader_context::LeafReaderContext,
            numeric_doc_values::NumericDocValues,
            point_values::PointValues,
            postings_enum::{PostingsEnum, PostingsEnumFlags},
            sorted_doc_values::SortedDocValues,
            sorted_numeric_doc_values::SortedNumericDocValues,
            term::Term,
            terms::{get_terms, Terms},
            vector_values::VectorValues,
        },
        search::top_docs::TopDocs,
    },
    bitvec::vec::BitVec,
    std::{future::Future, io::Result as IoResult, pin::Pin, sync::Arc},
};

/// [LeafReader] is a subtrait, providing an interface for accessing an index. Search of
/// an index is done entirely through this abstract interface, so that any subclass which implements
/// it is searchable. IndexReaders implemented by this subclass do not consist of several
/// sub-readers, they are atomic. They support retrieval of stored fields, doc values, terms, and
/// postings.
///
/// For efficiency, in this API documents are often referred to via _document numbers_,
/// non-negative integers which each name a unique document in the index. These document numbers are
/// ephemeral -- they may change as documents are added to and deleted from an index. Clients should
/// thus not rely on a given document having the same number between sessions.
///
/// # Note
///
/// [IndexReader] instances are completely thread safe, meaning multiple
/// threads can call any of its methods, concurrently.
pub trait LeafReader: IndexReader + Send + Sync {
    fn get_context(self: Arc<Self>) -> Arc<Pin<Box<LeafReaderContext>>>;

    fn doc_freq(self: Arc<Self>, term: &Term) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        let this = self;
        Box::pin(async move {
            let terms = get_terms(this, term.field()).await?;
            let te = terms.as_ref().iter().await;
            if te.as_mut().seek_exact(term.bytes().unwrap()).await? {
                Ok(te.as_ref().doc_freq().await?)
            } else {
                Ok(0)
            }
        })
    }

    /// Returns the number of documents containing the term `t`. This method returns 0 if
    /// the term or field does not exist. This method does not take into account deleted documents
    /// that have not yet been merged away.
    fn total_term_freq(self: Arc<Self>, term: &Term) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        let this = self;
        Box::pin(async move {
            let terms = get_terms(self, term.field()).await?;
            let te = terms.as_ref().iter().await;
            if te.as_mut().seek_exact(term.bytes().unwrap()).await? {
                Ok(te.as_ref().total_term_freq().await?)
            } else {
                Ok(0)
            }
        })
    }

    fn get_sum_doc_freq(self: Arc<Self>, field: &str) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        let this = self;
        Box::pin(async move {
            let terms = this.terms(field).await?;
            match terms {
                None => Ok(0),
                Some(terms) => Ok(terms.as_ref().get_sum_doc_freq().await?),
            }
        })
    }

    fn get_doc_count(self: Arc<Self>, field: &str) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        let this = self;
        Box::pin(async move {
            let terms = this.terms(field).await?;
            match terms {
                None => Ok(0),
                Some(terms) => Ok(terms.as_ref().get_doc_count().await?),
            }
        })
    }

    fn get_sum_total_term_freq(self: Arc<Self>, field: &str) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        let this = self;
        Box::pin(async move {
            let terms = this.terms(field).await?;
            match terms {
                None => Ok(0),
                Some(terms) => Ok(terms.as_ref().get_sum_total_term_freq().await?),
            }
        })
    }

    /// Returns the [Terms] index for this field, or `None` if it has none.
    fn terms(self: Arc<Self>, field: &str)
        -> Pin<Box<dyn Future<Output = IoResult<Option<Pin<Box<dyn Terms>>>>>>>;

    /// Returns [PostingsEnum] for the specified term. This will return `None` if either the field
    /// or term does not exist.
    ///
    /// # Note
    /// The returned [PostingsEnum] may contain deleted docs.
    fn postings(
        self: Arc<Self>,
        term: &Term,
        flags: PostingsEnumFlags,
    ) -> Pin<Box<dyn Future<Output = IoResult<Option<Pin<Box<dyn PostingsEnum>>>>>>> {
        assert!(term.bytes().is_some());
        let this = self;
        Box::pin(async move {
            let terms = get_terms(this, term.field()).await?;
            let te = terms.as_ref().iter().await;
            if te.as_mut().seek_exact(term.bytes().unwrap()).await? {
                Ok(Some(te.as_ref().postings(None, flags).await?))
            } else {
                Ok(None)
            }
        })
    }

    /// Returns [NumericDocValues] for this field, or `None` if no numeric doc values were indexed
    /// for this field.
    fn get_numeric_doc_values(
        self: Arc<Self>,
        field: &str,
    ) -> Pin<Box<dyn Future<Output = IoResult<Option<Pin<Box<dyn NumericDocValues>>>>>>>;

    /// Returns [BinaryDocValues] for this field, or `None` if no binary doc values were indexed
    /// for this field.
    fn get_binary_doc_values(
        self: Arc<Self>,
        field: &str,
    ) -> Pin<Box<dyn Future<Output = IoResult<Option<Pin<Box<dyn BinaryDocValues>>>>>>>;

    /// Returns [SortedDocValues] for this field, or `None` if no sorted doc values were indexed
    /// for this field.
    fn get_sorted_doc_values(
        self: Arc<Self>,
        field: &str,
    ) -> Pin<Box<dyn Future<Output = IoResult<Option<Pin<Box<dyn SortedDocValues>>>>>>>;

    /// Returns [SortedNumericDocValues] for this field, or `None` if no sorted numeric doc values
    /// were indexed for this field.
    fn get_sorted_numeric_doc_values(
        self: Arc<Self>,
        field: &str,
    ) -> Pin<Box<dyn Future<Output = IoResult<Option<Pin<Box<dyn SortedNumericDocValues>>>>>>>;

    /// Returns [NumericDocValues] representing norms for this field, or `None` if no numeric doc values
    /// were indexed for this field.
    fn get_norm_values(
        self: Arc<Self>,
        field: &str,
    ) -> Pin<Box<dyn Future<Output = IoResult<Option<Pin<Box<dyn NumericDocValues>>>>>>>;

    /// Returns [Vectorvalues] for this field, or `None` if no vector values were indexed for this field.
    fn get_vector_values(
        self: Arc<Self>,
        field: &str,
    ) -> Pin<Box<dyn Future<Output = IoResult<Option<Pin<Box<dyn VectorValues>>>>>>>;

    /// Return the _k_ nearest neighbor documents as determined by comparison of their vector values for
    /// this field, to the given vector, by the field's similarity function. The score of each document
    /// is derived from the vector similarity in a way that ensures scores are positive and that a
    /// larger score corresponds to a higher ranking.
    ///
    /// The search is allowed to be approximate, meaning the results are not guaranteed to be the
    /// true _k_k closest neighbors. For large values of _k_ (for example when _k_ is close to the total
    /// number of documents), the search may also retrieve fewer than _k_ documents.
    ///
    /// The returned [TopDocs] will contain a [ScoreDoc] for each nearest neighbor,
    /// sorted in order of their similarity to the query vector (decreasing scores). The
    /// [TotalHits] contains the number of documents visited during the search. If the search stopped
    /// early because it hit `visited_limit`, it is indicated through the relation
    /// [total_hits::Relation::GreaterThanOrEqualTo].
    ///
    /// # Parameters
    /// * `field`: the vector field to search
    /// * `target`: the vector-valued query
    /// * `k`: the number of docs to return
    /// * `accept_docs`: [BitVec] that represents the allowed documents to match, or `None`
    ///   if they are all allowed to match.
    /// * `visited_limit` the maximum number of nodes that the search is allowed to visit
    ///
    /// # Returns
    /// The _k _nearest neighbor documents, along with their (searchStrategy-specific) scores.
    fn search_nearest_vectors_f32(
        self: Arc<Self>,
        field: &str,
        target: &[f32],
        k: usize,
        accept_docs: Option<BitVec>,
        visited_limit: usize,
    ) -> Pin<Box<dyn Future<Output = IoResult<TopDocs>>>>;

    /// Return the _k_ nearest neighbor documents as determined by comparison of their vector values for
    /// this field, to the given vector, by the field's similarity function. The score of each document
    /// is derived from the vector similarity in a way that ensures scores are positive and that a
    /// larger score corresponds to a higher ranking.
    ///
    /// The search is allowed to be approximate, meaning the results are not guaranteed to be the
    /// true _k_k closest neighbors. For large values of _k_ (for example when _k_ is close to the total
    /// number of documents), the search may also retrieve fewer than _k_ documents.
    ///
    /// The returned [TopDocs] will contain a [ScoreDoc] for each nearest neighbor,
    /// sorted in order of their similarity to the query vector (decreasing scores). The
    /// [TotalHits] contains the number of documents visited during the search. If the search stopped
    /// early because it hit `visited_limit`, it is indicated through the relation
    /// [total_hits::Relation::GreaterThanOrEqualTo].
    ///
    /// # Parameters
    /// * `field`: the vector field to search
    /// * `target`: the vector-valued query
    /// * `k`: the number of docs to return
    /// * `accept_docs`: [BitVec] that represents the allowed documents to match, or `None`
    ///   if they are all allowed to match.
    /// * `visited_limit` the maximum number of nodes that the search is allowed to visit
    ///
    /// # Returns
    /// The _k _nearest neighbor documents, along with their (searchStrategy-specific) scores.
    fn search_nearest_vectors_u8(
        self: Arc<Self>,
        field: &str,
        target: &[u8],
        k: usize,
        accept_docs: Option<BitVec>,
        visited_limit: usize,
    ) -> Pin<Box<dyn Future<Output = IoResult<TopDocs>>>>;

    /// Get the [FieldInfos] describing all fields in this reader.
    ///
    /// # Note
    /// Implementations should cache the [FieldInfo]s instance returned by this method such that
    /// subsequent calls to this method return a cloned instance.
    fn get_field_infos(self: Arc<Self>) -> Pin<Box<dyn Future<Output = IoResult<FieldInfos>>>>;

    /// Returns the [BitVec] representing live (not deleted) docs. A set bit indicates the doc ID
    /// has not been deleted. If this method returns `None` it means there are no deleted documents (all
    /// documents are live).
    fn get_live_docs(self: Arc<Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<BitVec>>>>>;

    /// Returns the [PointValues] used for numeric or spatial searches for the given field, or
    /// `None` if there are no point fields.
    fn get_point_values(&mut self, field: &str) -> IoResult<Option<Box<dyn PointValues>>>;

    /// Checks consistency of this reader.
    ///
    /// Note that this may be costly in terms of I/O, e.g. may involve computing a checksum value
    /// against large data files.
    fn check_integrity(self: Arc<Self>) -> Pin<Box<dyn Future<Output = IoResult<()>>>>;

    /// Return metadata about this leaf.
    fn get_metadata(self: Arc<Self>) -> LeafMetaData;
}
