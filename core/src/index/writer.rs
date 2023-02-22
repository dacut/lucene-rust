/// Hard limit on maximum number of documents that may be added to the index. If you try to add
/// more than this you will encounter a [crate::LuceneError::TooManyDocs] error.
pub const MAX_DOCS: u32 = i32::MAX as u32 - 128;

/// Maximum value of the token position in an indexed field.
pub const MAX_POSITION: u32 = i32::MAX as u32 - 128;
