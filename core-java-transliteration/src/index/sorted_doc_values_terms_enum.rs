use {
    crate::{
        index::{
            impacts_enum::ImpactsEnum,
            postings_enum::{PostingsEnum, PostingsEnumFlags},
            base_terms_enum,
            ord_term_state::OrdTermState,
            sorted_doc_values::{LookupResult, SortedDocValues},
            terms_enum::{SeekStatus, TermsEnum},
            term_state::TermState,
        },
        util::{attribute_source::AttributeSource, bytes_iterator::BytesIterator},
    },
    pin_project::pin_project,
    std::{
        future::{ready, Future},
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
        pin::Pin,
        ptr::NonNull,
    },
};

#[derive(Debug)]
#[pin_project]
pub struct SortedDocValuesTermsEnum<'a> {
    values: NonNull<dyn SortedDocValues + 'a>,
    current_ord: Option<i32>,
    scratch: Vec<u8>,
}

unsafe impl<'a> Send for SortedDocValuesTermsEnum<'a> {}
unsafe impl<'a> Sync for SortedDocValuesTermsEnum<'a> {}

impl<'a> SortedDocValuesTermsEnum<'a> {
    pub fn new(sorted_doc_values: Pin<&'a mut dyn SortedDocValues>) -> Self {
        Self {
            // Safety: We never move data out of values.
            values: unsafe { sorted_doc_values.get_unchecked_mut() }.into(),
            current_ord: None,
            scratch: Vec::new(),
        }
    }
}

impl<'a> TermsEnum for SortedDocValuesTermsEnum<'a> {
    fn attributes(&self) -> Box<dyn AttributeSource> {
        base_terms_enum::attributes()
    }

    fn term(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Vec<u8>>>>> {
        Box::pin(ready(Ok(self.scratch.clone())))
    }

    fn ord(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<u64>>>> {
        let this = self.project_ref();
        Box::pin(ready(Ok(this.current_ord.unwrap() as u64)))
    }

    fn seek_exact(self: Pin<&mut Self>, text: &[u8]) -> Pin<Box<dyn Future<Output = IoResult<bool>>>> {
        let this = self;

        Box::pin(async move {
            // Safety: This is pinned via the lifetime of the struct.
            let values = unsafe { Pin::new_unchecked(this.values.as_mut()) };
            match values.lookup_term(text).await? {
                LookupResult::Found(ord) => {
                    this.current_ord = Some(ord);
                    this.scratch = text.to_vec();
                    Ok(true)
                }
                LookupResult::NotFound(_) => Ok(false),
            }
        })
    }

    fn seek_ceil(self: Pin<&mut Self>, text: &[u8]) -> Pin<Box<dyn Future<Output = IoResult<SeekStatus>>>> {
        let this = self.project();
        Box::pin(async move {
            // Safety: This is pinned via the lifetime of the struct.
            let values = unsafe { Pin::new_unchecked(this.values.as_mut()) };

            match values.lookup_term(text).await? {
                LookupResult::Found(ord) => {
                    *this.current_ord = Some(ord);
                    *this.scratch = text.to_vec();
                    Ok(SeekStatus::Found)
                }
                LookupResult::NotFound(ord) => {
                    *this.current_ord = Some(ord);
                    if ord >= values.as_ref().get_value_count().await? as i32 {
                        Ok(SeekStatus::End)
                    } else {
                        match values.as_mut().lookup_ord(ord).await? {
                            Some(data) => *this.scratch = data,
                            None => this.scratch.clear(),
                        };
                        Ok(SeekStatus::NotFound)
                    }
                }
            }
        })
    }

    fn seek_exact_ord(self: Pin<&mut Self>, ord: u64) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
        let this = self.project();
        Box::pin(async move {
            // Safety: This is pinned via the lifetime of the struct.
            let values = unsafe { Pin::new_unchecked(this.values.as_mut()) };

            assert!(ord >= 0 && ord < values.as_ref().get_value_count().await? as u64);
            *this.current_ord = Some(ord as i32);
            match values.lookup_ord(ord as i32).await? {
                Some(data) => *this.scratch = data,
                None => this.scratch.clear(),
            };
            Ok(())
        })
    }

    fn seek_exact_state(
        self: Pin<&mut Self>,
        term: &[u8],
        _state: Pin<Box<dyn TermState>>,
    ) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
        Box::pin(async move {
            if self.seek_exact(term).await? {
                Ok(())
            } else {
                Err(IoError::new(IoErrorKind::InvalidData, format!("term {} not found", String::from_utf8_lossy(term))))
            }
        })
    }

    fn doc_freq(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "doc_freq not supported for SortedDocValuesTermsEnum"))))
    }

    fn total_term_freq(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "total_term_freq not supported for SortedDocValuesTermsEnum"))))
    }

    fn postings(
        self: Pin<&Self>,
        reuse: Option<Pin<Box<dyn PostingsEnum>>>,
        flags: PostingsEnumFlags,
    ) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn PostingsEnum>>>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, "postings not supported for SortedDocValuesTermsEnum"))))
    }

    fn impacts(self: Pin<&Self>, flags: u32) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn ImpactsEnum>>>>>> {
        Box::pin(ready(Err(IoError::new(IoErrorKind::Unsupported, format!("impacts not supported for SortedDocValuesTermsEnum")))))
    }

    fn term_state(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn TermState>>>>>> {
        let ts: Pin<Box<dyn TermState>> = Box::pin(OrdTermState::new(self.current_ord.map(|value| value as i64).unwrap_or(-1)));

        Box::pin(ready(Ok(ts)))
    }
}

impl<'a> BytesIterator for SortedDocValuesTermsEnum<'a> {
    fn next(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<Vec<u8>>>>>> {
        let this = self;
        Box::pin(async move {
            let ord = this.current_ord.unwrap_or(0) + 1;
            this.current_ord = Some(ord);
            let values = unsafe { Pin::new_unchecked(this.values.as_mut()) };
            if ord >= values.as_ref().get_value_count().await? as i32 {
                Ok(None)
            } else {
                match values.lookup_ord(ord).await? {
                    Some(data) => this.scratch = data,
                    None => this.scratch.clear(),
                };
                Ok(Some(this.scratch.clone()))
            }
        })
    }
}
