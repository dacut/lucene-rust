use {
    crate::{
        index::{term_state::TermState, terms_enum::{TermsEnum, SeekStatus}},
        util::attribute_source::{AttributeSource, AttributeSourceBase},
    },
    std::{
        future::{ready, Future},
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
        pin::Pin,
    },
};

/// Default implementations for
///
/// * [crate::index::terms_enum::TermsEnum::attributes]
/// * [crate::index::terms_enum::SeekState::term_state]
/// * [Seek::seek_exact]
///
/// In some cases, the default implementation may be slow and consume huge memory, so subclass _should_
/// have its own implementation if possible.

pub fn attributes() -> Box<dyn AttributeSource> {
    Box::new(AttributeSourceBase::new())
}

pub fn term_state() -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<()>>>>>> {
    let ts = ();
    let pin_ts: Pin<Box<()>> = Box::pin(ts);
    Box::pin(ready(Ok(pin_ts)))
}

pub fn seek_exact(this: Pin<Box<dyn TermsEnum>>, text: &[u8]) -> Pin<Box<dyn Future<Output = IoResult<bool>>>> {
    Box::pin(async move {
        match this.as_mut().seek_ceil(text).await? {
            SeekStatus::Found => Ok(true),
            _ => Ok(false),
        }
    })
}

pub fn seek_exact_state<T>(
    this: Pin<Box<dyn TermsEnum>>,
    term: &[u8],
    state: Pin<Box<()>>,
) -> Pin<Box<dyn Future<Output = IoResult<()>>>> {
    Box::pin(async move {
        if this.as_mut().seek_exact(term).await? {
            Ok(())
        } else {
            Err(IoError::new(IoErrorKind::InvalidInput, format!("term {} does not exist", String::from_utf8_lossy(term))))
        }
    })
}

pub struct BaseTermState {}
impl TermState for BaseTermState {}
