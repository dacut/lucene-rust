use {
    crate::index::{
        doc_values_iterator::DocValuesIterator, doc_values_type::DocValuesType, leaf_reader::LeafReader,
        sorted_doc_values::SortedDocValues,
    },
    std::{
        future::{ready, Future},
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
        num::NonZeroU32,
        pin::Pin,
        sync::Arc,
    },
};

/// Returns [SortedDocValues] for the field, or [SortedDocValues::empty] if it has none.
///
/// # Returns
/// docvalues instance, or an empty instance if `field` does not exist in this reader.
///
/// # Errors
/// * [IoError] of kind [IoErrorKind::InvalidInput] if `field` exists, but was not indexed with docvalues.
/// * [IoError] of kind [IoErrorKind::InvalidInput] if `field` has docvalues, but the type is not [DocValuesType::Sorted].
/// * [IoError] if an I/O error occurs.
pub async fn get_sorted(mut reader: Arc<dyn LeafReader>, field: &str) -> IoResult<Pin<Box<dyn SortedDocValues>>> {
    match reader.get_sorted_doc_values(field).await? {
        None => {
            check_field(reader, field, vec![DocValuesType::Sorted]).await?;
            Ok(empty_sorted())
        }
        Some(dv) => Ok(dv),
    }
}

// helper method: to give a nice error when LeafReader.getXXXDocValues returns null.
async fn check_field(r#in: Arc<dyn LeafReader>, field: &str, expected: Vec<DocValuesType>) -> IoResult<()> {
    match r#in.get_field_infos().await?.get_field_info(field) {
        None => Ok(()),
        Some(fi) => {
            let acutal = fi.get_doc_values_type();
            let expected = if expected.len() == 1 {
                format!("expected={:?})", expected[0])
            } else {
                format!(
                    "expected one of {})",
                    expected.iter().map(|t| format!("{:?}", t)).collect::<Vec<_>>().join(", ")
                )
            };

            Err(IoError::new(
                IoErrorKind::InvalidInput,
                format!("unexpected docvalues type {acutal:?} for field '{field}' {expected}. Re-index with correct docvalues type."
                ),
            ))
        }
    }
}

/// An empty [SortedDocValues] that returns Vec::<u8>::new() for every document.
#[derive(Debug)]
pub struct EmptySorted {
    doc_id: Option<NonZeroU32>,
}

impl SortedDocValues for EmptySorted {
    fn ord_value(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<i32>>>> {
        assert!(false, "EmptySorted::ord_value should never be called");
        unreachable!();
    }

    fn lookup_ord(self: Pin<&mut Self>, ord: i32) -> Pin<Box<dyn Future<Output = IoResult<Option<Vec<u8>>>>>> {
        Box::pin(ready(None))
    }

    fn get_value_count(self: Pin<&Self>) -> Pin<Box<dyn Future<Output = IoResult<usize>>>> {
        Box::pin(ready(Ok(0)))
    }

    fn doc_id(self: Pin<&Self>) -> Option<NonZeroU32> {
        self.doc_id
    }

    fn next_doc(self: Pin<&mut Self>) -> Pin<Box<dyn Future<Output = IoResult<Option<i32>>>>> {
        self.doc_id = None;
        Box::pin(ready(None))
    }
    fn advance(self: Pin<&mut Self>, target: Option<i32>) -> Pin<Box<dyn Future<Output = IoResult<Option<i32>>>>> {
        self.doc_id = None;
        Box::pin(ready(None))
    }

    fn cost(self: Pin<&Self>) -> u64 {
        0
    }
}

impl DocValuesIterator for EmptySorted {
    fn advance_exact(self: Pin<&mut Self>, target: NonZeroU32) -> Pin<Box<dyn Future<Output = IoResult<bool>>>> {
        self.doc_id = Some(target);
        Box::pin(ready(false))
    }
}
