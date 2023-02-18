use {
    crate::{
        index::{
            automaton_terms_enum::AutomatonTermsEnum,
            doc_values_iterator::DocValuesIterator,
            single_terms_enum::SingleTermsEnum,
            sorted_doc_values_terms_enum::SortedDocValuesTermsEnum,
            terms_enum::{EmptyTermsEnum, TermsEnum},
        },
        search::doc_id_set_iterator::DocIdSetIterator,
        util::automaton::compiled_automaton::CompiledAutomaton,
    },
    std::{
        cmp::Ordering,
        fmt::Debug,
        future::{ready, Future},
        io::Result as IoResult,
        pin::Pin,
    },
};

/// The result from [SortedDocValues::lookup_term]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LookupResult {
    /// The key exists and this is its ordinal.
    Found(i32),

    /// The key does not exist; this is where it would be inserted.
    NotFound(i32),
}

pub trait SortedDocValues: DocValuesIterator {
    /// Returns the ordinal for the current docID. It is illegal to call this method after
    /// [DocValuesIterator::advance_exact] returned `false`.
    fn ord_value(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<u32>>>>;

    /// Retrieves the value for the specified ordinal.
    fn lookup_ord(self: Pin<&mut Self>, ord: i32) -> Pin<Box<dyn Future<Output = IoResult<Option<Vec<u8>>>>>>;

    /// Returns the number of unique values.
    fn get_value_count(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>>;

    /// If `key` exists, returns its ordinal, else returns `None`.
    fn lookup_term(self: Pin<&mut Self>, key: &[u8]) -> Pin<Box<dyn Future<Output = IoResult<LookupResult>>>> {
        let this = self;

        Box::pin(async move {
            let mut low = 0;
            let mut high = (this.as_ref().get_value_count().await? - 1) as i32;

            while low <= high {
                let mid = (low + high) >> 1;
                let term = this.as_mut().lookup_ord(mid).await?.unwrap();
                match term.as_slice().cmp(key) {
                    Ordering::Less => low = mid + 1,
                    Ordering::Greater => high = mid - 1,
                    Ordering::Equal => return Ok(LookupResult::Found(mid)),
                }
            }

            Ok(LookupResult::NotFound(low))
        })
    }

    /// Returns a [TermsEnum] over the values. The enum supports [TermsEnum::ord] and [TermsEnum::seek_exact].
    fn terms_enum(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<SortedDocValuesTermsEnum>>>> {
        Box::pin(ready(Ok(SortedDocValuesTermsEnum::new(self))))
    }

    fn intersect(
        self: Pin<&mut Self>,
        automaton: CompiledAutomaton,
    ) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn TermsEnum>>>>>>
    where
        Self: Sized,
    {
        let this = self;
        Box::pin(async move { intersect(this, automaton).await })
    }

    // Ugly hack to bring DocValuesIterator and DocIdSetIterator methods into the vtable.
    // See https://github.com/rust-lang/rfcs/issues/2765
    fn advance_exact(self: Pin<&mut Self>, target: u32) -> Pin<Box<dyn Future<Output = IoResult<bool>>>> {
        DocValuesIterator::advance_exact(self, target)
    }

    fn doc_id(self: Pin<&Self>) -> Option<u32> {
        DocIdSetIterator::doc_id(self)
    }

    fn next_doc(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<u32>>>>> {
        DocIdSetIterator::next_doc(self)
    }

    fn advance(self: Pin<&mut Self>, target: Option<u32>) -> Pin<Box<dyn Future<Output = IoResult<Option<u32>>>>> {
        DocIdSetIterator::advance(self, target)
    }

    fn slow_advance(self: Pin<&mut Self>, target: u32) -> Pin<Box<dyn Future<Output = IoResult<Option<u32>>>>> {
        DocIdSetIterator::slow_advance(self, target)
    }

    fn cost(self: Pin<&Self>) -> u64 {
        DocIdSetIterator::cost(self)
    }
}

async fn intersect<T>(mut this: Pin<&mut T>, automaton: CompiledAutomaton) -> IoResult<Pin<Box<dyn TermsEnum>>>
where
    T: SortedDocValues + ?Sized,
{
    match automaton {
        CompiledAutomaton::None => Ok(Box::pin(EmptyTermsEnum::new())),
        CompiledAutomaton::All => {
            let te = this.as_mut().terms_enum().await?;
            let te: Pin<Box<dyn TermsEnum>> = Box::pin(te);
            Ok(te)
        }
        CompiledAutomaton::Single(s) => {
            let te = this.as_mut().terms_enum().await?;
            let ste = SingleTermsEnum::new(te, s.get_term().to_vec());
            Ok(Box::pin(ste))
        }
        CompiledAutomaton::Normal(n) => {
            let te = this.as_mut().terms_enum().await?;
            Ok(Box::pin(AutomatonTermsEnum::new(te, &automaton)))
        }
    }
}
