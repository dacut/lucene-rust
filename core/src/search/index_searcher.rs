const MAX_CLAUSE_COUNT: usize = 1024;

/// By default we count hits accurately up to 1000. This makes sure that we don't spend most time
/// on computing hit counts
const TOTAL_HITS_THRESHOLD: usize = 1000;

/// Thresholds for index slice allocation logic.
const MAX_DOCS_PER_SLICE: usize = 250_000;

const MAX_SEGMENTS_PER_SLICE: usize = 5;

/// Implements search over a single IndexReader.
///
/// Applications usually need only call the inherited [IndexSearcher::search] method. For
/// performance reasons, if your index is unchanging, you should share a single IndexSearcher
/// instance across multiple searches instead of creating a new one per-search. If your index has
/// changed and you wish to see the changes reflected in searching, you should use 
/// [DirectoryReader::open_if_changed] to obtain a new reader and then create a new
/// `IndexSearcher` from that. Also, for low-latency turnaround it's best to use a near-real-time
/// reader ({@link DirectoryReader#open(IndexWriter)}). Once you have a new {@link IndexReader}, it's
/// relatively cheap to create a new IndexSearcher from it.
///
/// # Note
/// The [::search] and [search_after] methods are configured to only count
/// top hits accurately up to 1,000 and may return a [total_hits::Relation] lower bound
/// of the hit count if the hit count is greater than or equal to 1,000. On queries that
/// match lots of documents, counting the number of hits may take much longer than computing the top
/// hits so this trade-off allows to get some minimal information about the hit count without slowing
/// down search too much. The [TopDocs::score_docs] slice is always accurate however. If this
/// behavior doesn't suit your needs, you should create collectors manually with either {@link
/// [TopScoreDocCollector::create] or [TopFieldCollector::create] and call 
/// [::search_collector].
#[derive(Debug)]
pub struct IndexSearcher {
    // TODO: implement
}
