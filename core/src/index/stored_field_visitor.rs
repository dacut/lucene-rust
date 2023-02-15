use {
    crate::index::field_info::FieldInfo,
    std::io::Result as IoResult,
};

/// Provides a low-level means of accessing the stored field values in an index. See 
/// [crate::index::stored_fields::StoredFields::document].
/// 
/// # Note
/// A [StoredFieldVisitor] implementation should not try to load or visit other
/// stored documents in the same reader because the implementation of stored fields for most codecs
/// is not reentrant and you will see strange exceptions as a result.
/// 
/// See [crate::index::document_stored_field_visitor::DocumentStoredFieldVisitor], which is a
/// [StoredFieldVisitor] that build the [crate::index::document::Document] containing all stored fields.
///  This is used by [crate::index::stored_fields::StoredFields::document].
pub trait StoredFieldVisitor {
    /// Process a binary field
    fn binary_field(&mut self, field_info: &FieldInfo, value: &[u8]) -> IoResult<()>;
}