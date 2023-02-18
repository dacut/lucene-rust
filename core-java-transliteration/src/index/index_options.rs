/// Controls how much information is stored in the postings lists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexOptions {
    // NOTE: order is important here; FieldInfo uses this
    // order to merge two conflicting IndexOptions (always
    // "downgrades" by picking the lowest).
    /// Not indexed
    None,

    /// Only documents are indexed: term frequencies and positions are omitted. Phrase and other
    /// positional queries on the field will throw an exception, and scoring will behave as if any term
    /// in the document appears only once.
    Docs,

    /// Only documents and term frequencies are indexed: positions are omitted. This enables normal
    /// scoring, except Phrase and other positional queries will throw an exception.
    DocsAndFreqs,

    /// Indexes documents, frequencies and positions. This is a typical default for full-text search:
    /// full scoring is enabled and positional queries are supported.
    DocsAndFreqsAndPositions,

    /// Indexes documents, frequencies, positions and offsets. Character offsets are encoded alongside
    /// the positions.
    DocsAndFreqsAndPositionsAndOffsets,
}

impl IndexOptions {
    pub fn positions_indexed(&self) -> bool {
        matches!(self, IndexOptions::DocsAndFreqsAndPositions | IndexOptions::DocsAndFreqsAndPositionsAndOffsets)
    }
}