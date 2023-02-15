use {
    crate::{
        index::{
            filtered_terms_enum::{AcceptStatus, FilteredTermsEnum, FilteredTermsEnumBase},
            impacts_enum::ImpactsEnum,
            postings_enum::{PostingsEnum, PostingsEnumFlags},
            term_state::TermState,
            terms_enum::{SeekStatus, TermsEnum},
        },
        util::{attribute_source::AttributeSource, bytes_iterator::BytesIterator},
    },
    pin_project::pin_project,
    std::{
        future::{ready, Future},
        io::Result as IoResult,
        pin::Pin,
    },
};

/**
 * Implementation of [FilteredTermsEnum] for enumerating a single term.
 *
 * For example, this can be used by [MultiTermQuery]s that need only visit one term, but
 * want to preserve MultiTermQuery semantics such as [MultiTermQuery::get_rewrite_method].
 */
#[derive(Debug)]
#[pin_project]
pub struct SingleTermsEnum<T> {
    #[pin]
    filtered_terms_enum: FilteredTermsEnumBase<T>,
    single_ref: Vec<u8>,
}

impl<T> SingleTermsEnum<T> {
    pub fn new(terms_enum: T, single: Vec<u8>) -> Self {
        Self {
            filtered_terms_enum: FilteredTermsEnumBase::new(terms_enum),
            single_ref: single,
        }
    }
}

impl<T> FilteredTermsEnum for SingleTermsEnum<T>
where
    T: TermsEnum,
{
    fn accept(self: Pin<&mut Self>, term: &[u8]) -> Pin<Box<dyn Future<Output = IoResult<AcceptStatus>>>> {
        let result = if term == self.single_ref { AcceptStatus::Yes } else { AcceptStatus::No };

        Box::pin(ready(Ok(result)))
    }

    fn set_initial_seek_term(self: Pin<&mut Self>, term: Option<&[u8]>) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
        let this = self.project();
        this.filtered_terms_enum.set_initial_seek_term(term)
    }

    fn next_seek_term(
        self: Pin<&mut Self>,
        current_term: Option<&[u8]>,
    ) -> Pin<Box<dyn Future<Output = IoResult<Option<Vec<u8>>>>>> {
        let this = self.project();
        this.filtered_terms_enum.next_seek_term(current_term)
    }
}

impl<T> TermsEnum for SingleTermsEnum<T>
where
    T: TermsEnum,
{
    /// Returns the related attributes.
    fn attributes(&self) -> Box<dyn AttributeSource> {
        self.filtered_terms_enum.attributes()
    }

    fn term(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Vec<u8>>>>> {
        let this = self.project_ref();
        this.filtered_terms_enum.term()
    }

    fn ord(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<u64>>>> {
        let this = self.project_ref();
        this.filtered_terms_enum.ord()
    }

    fn doc_freq(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        let this = self.project_ref();
        this.filtered_terms_enum.doc_freq()
    }

    fn total_term_freq(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        let this = self.project_ref();
        this.filtered_terms_enum.total_term_freq()
    }

    fn postings(
        self: Pin<&Self>,
        reuse: Option<Pin<Box<dyn PostingsEnum>>>,
        flags: PostingsEnumFlags,
    ) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn PostingsEnum>>>>>> {
        let this = self.project_ref();
        this.filtered_terms_enum.postings(reuse, flags)
    }

    fn impacts(self: Pin<&Self>, flags: u32) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn ImpactsEnum>>>>>> {
        let this = self.project_ref();
        this.filtered_terms_enum.impacts(flags)
    }

    fn seek_exact_state(
        self: Pin<&mut Self>,
        term: &[u8],
        state: Pin<Box<dyn TermState>>,
    ) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
        let this = self.project();
        this.filtered_terms_enum.seek_exact_state(term, state)
    }

    fn term_state(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn TermState>>>>>> {
        let this = self.project_ref();
        this.filtered_terms_enum.term_state()
    }

    fn seek_exact(self: Pin<&mut Self>, text: &[u8]) -> Pin<Box<dyn Future<Output = IoResult<bool>>>> {
        let this = self.project();
        this.filtered_terms_enum.seek_exact(text)
    }

    fn seek_ceil(self: Pin<&mut Self>, text: &[u8]) -> Pin<Box<dyn Future<Output = IoResult<SeekStatus>>>> {
        let this = self.project();
        let fte = this.filtered_terms_enum;
        fte.seek_ceil(text)
    }

    fn seek_exact_ord(self: Pin<&mut Self>, ord: u64) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
        let this = self.project();
        this.filtered_terms_enum.seek_exact_ord(ord)
    }
}

impl<T> BytesIterator for SingleTermsEnum<T>
where
    T: BytesIterator,
{
    fn next(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<Vec<u8>>>>>> {
        let this = self.project();
        this.filtered_terms_enum.next()
    }
}
