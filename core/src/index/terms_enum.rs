use {
    crate::{
        index::{
            impacts_enum::ImpactsEnum,
            postings_enum::{PostingsEnum, PostingsEnumFlags},
            term_state::TermState,
        },
        util::{attribute_source::{AttributeSource, AttributeSourceBase}, bytes_iterator::BytesIterator},
    },
    std::{
        fmt::Debug,
        future::{ready, Future},
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
        pin::Pin,
    },
};

/// Represents returned result from [TermsEnum::seek_ceil].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SeekStatus {
    /// The term was not found, and the end of iteration was hit.
    End,

    /// The precise term was found.
    Found,

    /// A different term was found after the requested term
    NotFound,
}

/// Iterator to seek ([TermsEnum::seek_ceil], [TermsEnum::seek_exact]) or step through
/// ([TermsEnum::next] terms to obtain frequency information ([TermsEnum::doc_freq]), [PostingsEnum]
/// for the current term ([TermsEnum::postings].
///
/// Term enumerations are always ordered by BytesRef.compareTo, which is Unicode sort order if the
/// terms are UTF-8 bytes. Each term in the enumeration is greater than the one before it.
///
/// The TermsEnum is unpositioned when you first obtain it and you must first successfully call
/// [TermsEnum::next] or one of the `seek` methods.
pub trait TermsEnum: BytesIterator + Debug {
    /// Returns the related attributes.
    fn attributes(&self) -> Box<dyn AttributeSource>;

    /// Returns the current term. Do not call this when the enum is unpositioned.
    fn term(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Vec<u8>>>>>;

    /// Returns ordinal position for current term. This is an optional method (the codec may return
    /// an unsupported operation IoError). Do not call this when the enum is unpositioned.
    fn ord(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<u64>>>>;

    /// Attempts to seek to the exact term, returning true if the term is found. If this returns false,
    /// the enum is unpositioned. For some codecs, seekExact may be substantially faster than [::seekCeil].
    ///
    /// # Returns
    /// Returns `true` if the term is found, `false` if the enum is unpositioned.
    fn seek_exact(self: Pin<&mut Self>, text: &[u8]) -> Pin<Box<dyn Future<Output = IoResult<bool>>>>;

    /// Seeks to the specified term, if it exists, or to the next (ceiling) term. Returns [SeekStatus] to
    /// indicate whether exact term was found, a different term was found, or EOF was hit. The target
    /// term may be before or after the current term. If this returns [SeekStatus::End], the enum is
    /// unpositioned.
    fn seek_ceil(self: Pin<&mut Self>, text: &[u8]) -> Pin<Box<dyn Future<Output = IoResult<SeekStatus>>>>;

    /// Seeks to the specified term by ordinal (position) as previously returned by [::ord]. The
    /// target ord may be before or after the current ord, and must be within bounds.
    fn seek_exact_ord(self: Pin<&mut Self>, ord: u64) -> Pin<Box<dyn Future<Output = IoResult<()>>>>;

    /// Returns the number of documents containing the current term. Do not call this when the enum is
    /// unpositioned. [SeekStatus::End].
    fn doc_freq(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>>;

    /// Returns the total number of occurrences of this term across all documents (the sum of the
    /// freq() for each doc that has this term). Note that, like other term measures, this measure does
    /// not take deleted documents into account.
    fn total_term_freq(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>>;

    /// Get [PostingsEnum] for the current term. Do not call this when the enum is unpositioned.
    ///
    /// # Note
    /// The returned iterator may return deleted documents, so deleted documents have
    /// to be checked on top of the [PostingsEnum].
    ///
    /// Use this method if you only require documents and frequencies, and do not need any proximity
    /// data. This method is equivalent to {@link #postings(PostingsEnum, int) postings(reuse,
    /// PostingsEnum.FREQS)}
    ///
    /// # Parameters
    /// * `reuse`: pass a prior [PostingsEnum] for possible reuse
    /// * `flags`: specifies which optional per-document values you require; see [PostingsEnum::Freqs].
    fn postings(
        self: Pin<&Self>,
        reuse: Option<Pin<Box<dyn PostingsEnum>>>,
        flags: PostingsEnumFlags,
    ) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn PostingsEnum>>>>>>;

    /// Return an [ImpactsEnum].
    fn impacts(self: Pin<&Self>, flags: u32) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn ImpactsEnum>>>>>>;

    /// Expert: Seeks a specific position by [TermState] previously obtained from [::term_state].
    /// Callers should maintain the [TermState] to use this method. Low-level
    /// implementations may position the [TermsEnum] without re-seeking the term dictionary.
    ///
    /// Seeking by [TermState] should only be used iff the state was obtained from the same
    /// [TermsEnum] instance.
    ///
    /// # Notes
    /// Using this method with an incompatible [TermState] might leave this [TermsEnum] in undefined
    /// state. On a segment level [TermState] instances are compatible only iff the source and the
    /// target [TermsEnum] operate on the same field. If operating on segment level, TermState
    /// instances must not be used across segments.
    ///
    /// A seek by [TermState] might not restore the [AttributeSource]'s state.
    /// [AttributeSource] states must be maintained separately if this method is used.
    ///
    /// # Parameters
    /// * `term`: the term the TermState corresponds to
    /// * `state` the [TermState]
    fn seek_exact_state(
        self: Pin<&mut Self>,
        term: &[u8],
        state: Pin<Box<dyn TermState>>,
    ) -> Pin<Box<dyn Future<Output = IoResult<()>>>>;

    /// Expert: Returns the TermsEnums internal state to position the TermsEnum without re-seeking the
    /// term dictionary.
    ///
    /// # Note
    /// A seek by [TermState] might not capture the [AttributeSource]'s state.
    /// Callers must maintain the [AttributeSource] states separately
    fn term_state(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn TermState>>>>>>;
}

#[derive(Debug)]
pub struct EmptyTermsEnum;

impl EmptyTermsEnum {
    pub const fn new() -> Self {
        EmptyTermsEnum
    }
}

impl TermsEnum for EmptyTermsEnum {
    fn attributes(&self) -> Box<dyn AttributeSource> {
        Box::new(AttributeSourceBase::new())
    }

    fn term(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Vec<u8>>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "term not supported for EmptyTermsEnum"))))
    }

    fn ord(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<u64>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "ord not supported for EmptyTermsEnum"))))
    }

    fn seek_exact(self: Pin<&mut Self>, text: &[u8]) -> Pin<Box<dyn Future<Output = IoResult<bool>>>> {
        let this = self;
        Box::pin(async move {
            match this.seek_ceil(text).await? {
                SeekStatus::Found => Ok(true),
                _ => Ok(false),
            }
        })
    }

    fn seek_ceil(self: Pin<&mut Self>, text: &[u8]) -> Pin<Box<dyn Future<Output = IoResult<SeekStatus>>>> {
        Box::pin(ready(Ok(SeekStatus::End)))
    }

    fn seek_exact_ord(self: Pin<&mut Self>, ord: u64) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
        Box::pin(ready(Ok(())))
    }

    fn doc_freq(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "doc_freq not supported for EmptyTermsEnum"))))
    }

    fn total_term_freq(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "total_term_freq not supported for EmptyTermsEnum"))))
    }

    fn postings(
        self: Pin<&Self>,
        _reuse: Option<Pin<Box<dyn PostingsEnum>>>,
        _flags: PostingsEnumFlags,
    ) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn PostingsEnum>>>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "postings not supported for EmptyTermsEnum"))))
    }

    fn impacts(self: Pin<&Self>, _flags: u32) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn ImpactsEnum>>>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "impacts not supported for EmptyTermsEnum"))))
    }

    fn term_state(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn TermState>>>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "term_state not supported for EmptyTermsEnum"))))
    }

    fn seek_exact_state(
        self: Pin<&mut Self>,
        _term: &[u8],
        _state: Pin<Box<dyn TermState>>,
    ) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "seek_exact_state not supported for EmptyTermsEnum"))))
    }
}

impl BytesIterator for EmptyTermsEnum {
    fn next(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<Vec<u8>>>>>> {
        Box::pin(ready(Ok(None)))
    }
}
