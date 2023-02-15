use {
    crate::{
        index::{
            impacts_enum::ImpactsEnum,
            postings_enum::{PostingsEnum, PostingsEnumFlags},
            term_state::TermState,
            terms_enum::{SeekStatus, TermsEnum},
        },
        util::{attribute_source::AttributeSource, bytes_iterator::BytesIterator},
    },
    pin_project::pin_project,
    std::{
        cmp::Ordering,
        future::{ready, Future},
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
        pin::Pin,
    },
};

/// Return value, if term should be accepted or the iteration should `End`. The
/// `*Seek` values denote, that after handling the current term the enum should call
/// [FilteredTermsEnum::next_seek_term] ]nextSeekTerm} and step forward.
#[derive(Debug, Eq, PartialEq)]
pub enum AcceptStatus {
    /// Accept the term and position the enum at the next term.
    Yes,

    /// Accept the term and advance ([FilteredTermsEnum::next_seek_term]) to the next term.
    YesAndSeek,

    /// Reject the term and position the enum at the next term.
    No,

    /// Reject the term and advance ([FilteredTermsEnum::next_seek_term]) to the next term.
    NoAndSeek,

    /// Reject the term and stop enumerating.
    End,
}

/// Abstract class for enumerating a subset of all terms.
///
/// Term enumerations are always ordered by `&[u8]` raw ordering. Each term in the
/// enumeration is greater than all that precede it.
///
/// # Note
/// Consumers of this enum cannot call the seek methods, it is forward only;
/// it returns [IoError] with an [IoErrorKind::Unsupported] kind when a seeking method is called.
pub trait FilteredTermsEnum: TermsEnum {
    /// Return if term is accepted, not accepted or the iteration should ended (and possibly seek).
    fn accept(self: Pin<&mut Self>, term: &[u8]) -> Pin<Box<dyn Future<Output = IoResult<AcceptStatus>>>>;

    /// Use this method to set the initial {@link BytesRef} to seek before iterating. This is a
    /// convenience method for implementations that do not implement [::next_seek_term]. If the initial
    /// seek term is `None`, the enum is empty.
    ///
    /// You can only use this method, if you keep the default implementation of [::next_seek_term].
    fn set_initial_seek_term(self: Pin<&mut Self>, term: Option<&[u8]>) -> Pin<Box<dyn Future<Output = IoResult<()>>>>;

    /// On the first call to [::next] or if [::accept] returns
    /// [AcceptStatus::YesAndSeek] or [AcceptStatus::NoAndSeek], this method will be called to
    /// eventually seek the underlying TermsEnum to a new position. On the first call,
    /// `current_term` will be `None`, later calls will provide the term the underlying enum is
    /// positioned at. This method returns per default only one time the initial seek term and then
    /// `None`, so no repositioning is ever done.
    ///
    /// Override this method, if you want a more sophisticated TermsEnum, that repositions the
    /// iterator during enumeration. If this method always returns `None` the enum is empty.
    ///
    /// # Note
    /// This method should always provide a greater term than the last
    /// enumerated term, else the behaviour of this enum violates the contract for TermsEnums.
    fn next_seek_term(
        self: Pin<&mut Self>,
        current_term: Option<&[u8]>,
    ) -> Pin<Box<dyn Future<Output = IoResult<Option<Vec<u8>>>>>>;
}

/// Partial base implementation of [FilteredTermsEnum] that provides a default implementation.
#[derive(Debug)]
#[pin_project]
pub struct FilteredTermsEnumBase<T> {
    initial_seek_term: Option<Vec<u8>>,
    do_seek: bool,

    /// Which term the enum is currently positioned to.
    actual_term: Option<Vec<u8>>,

    /// The delegate `TermsEnum`.
    #[pin]
    tenum: T,
}

impl<T> FilteredTermsEnumBase<T> {
    /// Creates a [FilteredTermsEnum] on a [TermsEnum].
    pub fn new(tenum: T) -> Self {
        Self::new_with_seek(tenum, true)
    }

    /// Creates a [FilteredTermsEnum] on a [TermsEnum] with the specified starting seek value.
    pub fn new_with_seek(tenum: T, start_with_seek: bool) -> Self {
        Self {
            initial_seek_term: None,
            do_seek: start_with_seek,
            actual_term: None,
            tenum,
        }
    }
}

impl<T> FilteredTermsEnumBase<T>
where
    T: TermsEnum,
{
    /// Use this method to set the initial Vec<u8> to seek before iterating. This is a
    /// convenience method for subclasses that do not override [::next_seek_term]. If the initial
    /// seek term is `None` (default), the enum is empty.
    ///
    /// You can only use this method if you keep the default implementation of [::next_seek_term].
    pub fn set_initial_seek_term(
        self: Pin<&mut Self>,
        term: Option<&[u8]>,
    ) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
        self.initial_seek_term = term.map(|t| t.to_vec());
        Box::pin(async move { Ok(()) })
    }

    /// On the first call to [TermsEnum::next] or if [TermsEnum::accept] returns
    /// [AcceptStatus::YesAndSeek] or [AcceptStatus::NO_AND_SEEK], this method will be called to
    /// eventually seek the underlying [TermsEnum] to a new position. On the first call,
    /// `current_term` will be `None`, later calls will provide the term the underlying enum is
    /// positioned at. This method returns per default only one time the initial seek term and then
    /// `None`, so no repositioning is ever done.
    ///
    /// Override this method if you want a more sophisticated TermsEnum, that repositions the
    /// iterator during enumeration. If this method always returns `None` the enum is empty.
    ///
    /// # Note
    /// This method should always provide a greater term than the last
    /// enumerated term, else the behaviour of this enum violates the contract for TermsEnums.
    pub fn next_seek_term(
        self: Pin<&mut Self>,
        current_term: Option<&[u8]>,
    ) -> Pin<Box<dyn Future<Output = IoResult<Option<Vec<u8>>>>>> {
        let result = self.initial_seek_term;
        self.initial_seek_term = None;

        Box::pin(async move { Ok(result) })
    }

    fn next_with_filterer<F: FilteredTermsEnum>(
        self: Pin<&mut Self>,
        filterer: Pin<&mut F>,
    ) -> Pin<Box<dyn Future<Output = IoResult<Option<Vec<u8>>>>>> {
        let this = self.project();
        Box::pin(async move {
            loop {
                // Seek or forward the iterator
                if *this.do_seek {
                    *this.do_seek = false;
                    let t = self.next_seek_term(self.actual_term.map(|t| t.as_slice())).await?;

                    // Make sure we always seek forward:
                    assert!(
                        this.actual_term.is_none()
                            || t.is_none()
                            || t.unwrap().cmp(this.actual_term.as_ref().unwrap()) == Ordering::Greater
                    );

                    if t.is_none() || this.tenum.seek_ceil(t.as_ref().unwrap()).await? == SeekStatus::End {
                        // no more terms to seek to or enum exhausted
                        break (Ok(None));
                    }

                    let actual_term = this.tenum.as_ref().term().await?;
                    *this.actual_term = Some(actual_term);
                } else {
                    *this.actual_term = this.tenum.next().await?;
                    if this.actual_term.is_none() {
                        // enum exhausted
                        break (Ok(None));
                    }
                }

                assert!(this.actual_term.is_some());
                let actual_term = this.actual_term.as_ref().unwrap();

                match filterer.accept(actual_term).await? {
                    AcceptStatus::YesAndSeek => {
                        *this.do_seek = true;
                        break (Ok(Some(actual_term.clone())));
                    }

                    AcceptStatus::Yes => break (Ok(Some(actual_term.clone()))),

                    AcceptStatus::NoAndSeek => {
                        // invalid term, seek next time
                        *this.do_seek = true;
                    }

                    AcceptStatus::End => {
                        // we are supposed to end the enum
                        break (Ok(None));
                    }

                    AcceptStatus::No => {
                        // we just iterate again
                    }
                }
            }
        })
    }
}

impl<T> TermsEnum for FilteredTermsEnumBase<T>
where
    T: TermsEnum,
{
    #[inline]
    fn attributes(&self) -> Box<dyn AttributeSource> {
        self.tenum.attributes()
    }

    fn term(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Vec<u8>>>>> {
        let this = self.project_ref();
        Box::pin(async move { this.tenum.term().await })
    }

    fn doc_freq(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        let this = self.project_ref();
        Box::pin(async move { this.tenum.doc_freq().await })
    }

    fn total_term_freq(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        let this = self.project_ref();
        Box::pin(async move { this.tenum.total_term_freq().await })
    }

    fn seek_ceil(self: Pin<&mut Self>, term: &[u8]) -> Pin<Box<dyn Future<Output = IoResult<SeekStatus>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "seek_ceil is not supported by FilteredTermsEnum"))))
    }

    fn seek_exact(self: Pin<&mut Self>, text: &[u8]) -> Pin<Box<dyn Future<Output = IoResult<bool>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "seek_exact is not supported by FilteredTermsEnum"))))
    }

    fn seek_exact_ord(self: Pin<&mut Self>, ord: u64) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "seek_exact_ord is not supported by FilteredTermsEnum"))))
    }

    fn ord(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<u64>>>> {
        let this = self.project_ref();
        Box::pin(async move { this.tenum.ord().await })
    }

    fn postings(
        self: Pin<&Self>,
        reuse: Option<Pin<Box<dyn PostingsEnum>>>,
        flags: PostingsEnumFlags,
    ) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn PostingsEnum>>>>>> {
        let this = self.project_ref();
        Box::pin(async move { this.tenum.postings(reuse, flags).await })
    }

    fn impacts(self: Pin<&Self>, flags: u32) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn ImpactsEnum>>>>>> {
        let this = self.project_ref();
        Box::pin(async move { this.tenum.impacts(flags).await })
    }

    fn seek_exact_state(
        self: Pin<&mut Self>,
        term: &[u8],
        state: Pin<Box<dyn TermState>>,
    ) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "seek_exact_state is not supported by FilteredTermsEnum"))))
    }

    fn term_state(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn TermState>>>>>> {
        let this = self.project_ref();
        Box::pin(async move { this.tenum.term_state().await })
    }

}

impl<T> BytesIterator for FilteredTermsEnumBase<T>
where
    T: BytesIterator,
{
    fn next(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<Vec<u8>>>>>> {
        let this = self.project();
        this.tenum.next()
    }
}
