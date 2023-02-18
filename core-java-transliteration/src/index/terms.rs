use {
    crate::{
        index::{
            leaf_reader::LeafReader,
            terms_enum::{EmptyTermsEnum, SeekStatus, TermsEnum},
        },
    },
    std::{
        future::{Future, ready},
        io::{ErrorKind as IoErrorKind, Result as IoResult},
        pin::Pin,
        sync::Arc,
    },
};

/// Access to the terms in a specific field. See [Fields].
pub trait Terms {
    /// Returns an iterator that will step through all terms.
    fn iter(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = Pin<Box<dyn TermsEnum>>>>>;

    /// Returns the number of terms for this field, or `None` if this measure isn't stored by the codec.
    /// Note that, just like other term measures, this measure does not take deleted documents into
    /// account.
    fn size(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<usize>>>>>;

    /// Returns the sum of [TermsEnum::total_term_freq] for all terms in this field. Note that, just
    /// like other term measures, this measure does not take deleted documents into account.
    fn get_sum_total_term_freq(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>>;

    /// Returns the sum of {@link TermsEnum#docFreq()} for all terms in this field. Note that, just
    /// like other term measures, this measure does not take deleted documents into account.
    fn get_sum_doc_freq(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>>;

    /// Returns the number of documents that have at least one term for this field. Note that, just
    /// like other term measures, this measure does not take deleted documents into account.
    fn get_doc_count(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>>;

    /// Returns true if documents in this field store per-document term frequency ([PostingsEnum::freq]).
    fn has_freqs(&self) -> bool;

    /// Returns true if documents in this field store offsets.
    fn has_offsets(&self) -> bool;

    /// Returns true if documents in this field store positions.
    fn has_positions(&self) -> bool;

    /// Returns true if documents in this field store payloads.
    fn has_payloads(&self) -> bool;

    /// Returns the smallest term (in lexicographic order) in the field. Note that, just like other
    /// term measures, this measure does not take deleted documents into account. This returns `None`
    /// when there are no terms.
    fn get_min(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<Vec<u8>>>>>> {
        let this = self;
        Box::pin(async move { this.iter().await.as_mut().next().await })
    }

    /// Returns the largest term (in lexicographic order) in the field. Note that, just like other term
    /// measures, this measure does not take deleted documents into account. This returns `None` when
    /// there are no terms.
    fn get_max(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<Vec<u8>>>>>> {
        let this = self;
        Box::pin(async move { 
            let size = this.as_ref().size().await?;

            match size {
                Some(0) => {
                    // empty: only possible from a FilteredTermsEnum...
                    return Ok(None);
                }
                Some(size) => {
                    let mut iterator = this.as_ref().iter().await;
                    match iterator.as_mut().seek_exact_ord(size as u64 - 1).await {
                        Ok(_) => return Ok(Some(iterator.as_ref().term().await?)),
                        Err(e) => match e.kind() {
                            IoErrorKind::Unsupported => (),
                            _ => return Err(e),
                        },
                    };
                }
                None => {}
            }
        
            // otherwise: binary search
            let mut iterator = this.as_ref().iter().await;
            let v = iterator.as_mut().next().await?;
        
            let Some(v) = v else {
                // empty: only possible from a FilteredTermsEnum...
                return Ok(None);
            };
        
            let mut scratch = vec![b'\0'];
        
            // Iterates over digits:
            loop {
                let mut low = 0;
                let mut high = 256;
        
                // Binary search current digit to find the highest
                // digit before END:
                while low != high {
                    let mid = (low + high) >> 1;
                    let scratch_len = scratch.len();
                    scratch[scratch_len - 1] = mid as u8;
        
                    if iterator.as_mut().seek_ceil(&scratch).await? == SeekStatus::End {
                        // Scratch was too high
                        if mid == 0 {
                            scratch.pop();
                            return Ok(Some(scratch));
                        }
        
                        high = mid;
                    } else {
                        // Scratch was too low; there is at least one term
                        // still after it:
                        if low == mid {
                            break;
                        }
        
                        low = mid;
                    }
                }
        
                // Proceed to next digit.
                scratch.push(b'\0');
            }
        
        })
    }
}

/// Am empty [Terms] which returns no terms.
#[derive(Clone, Debug, Default)]
pub struct EmptyTerms;

impl EmptyTerms {
    pub fn new() -> Self {
        Self
    }
}

impl Terms for EmptyTerms {
    fn iter(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = Pin<Box<dyn TermsEnum>>>>> {
        let ete = EmptyTermsEnum::new();
        let result: Pin<Box<dyn TermsEnum>> = Box::pin(ete);
        Box::pin(ready(result))
    }

    fn size(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<usize>>>>> {
        Box::pin(ready(Ok(Some(0))))
    }

    fn get_sum_total_term_freq(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        Box::pin(ready(Ok(0)))
    }

    fn get_sum_doc_freq(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        Box::pin(ready(Ok(0)))
    }

    fn get_doc_count(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        Box::pin(ready(Ok(0)))
    }

    fn has_freqs(&self) -> bool {
        false
    }

    fn has_offsets(&self) -> bool {
        false
    }

    fn has_positions(&self) -> bool {
        false
    }
 
    fn has_payloads(&self) -> bool {
        false
    }
}

pub async fn get_terms<LR>(reader: Arc<LR>, field: &str) -> IoResult<Pin<Box<dyn Terms>>>
where LR: LeafReader + ?Sized {
    let terms = reader.terms(field).await?;
    match terms {
        None => Ok(Box::pin(EmptyTerms::new())),
        Some(terms) => Ok(terms),
    }
}
