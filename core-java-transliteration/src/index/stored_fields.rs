use {
    crate::index::{stored_field_visitor::StoredFieldVisitor},
    crate::document::document::Document,
    std::{
        future::Future,
        io::Result as IoResult,
        pin::Pin,
    }
};

/// API for reading stored fields.
pub trait StoredFields {
    /// Returns the stored fields of the `n`th [Document] in this
    /// index. This is just sugar for using [DocumentStoredFieldVisitor].
    ///
    /// # Note
    /// For performance reasons, this method does not check if the requested document
    ///  is deleted, and therefore asking for a deleted document may yield unspecified results. Usually
    /// this is not required, however you can test if the doc is deleted by checking the [Bits]
    /// returned from [MultiBits::get_live_docs].
    /// 
    /// Only the content of a field is returned, if that field was stored during indexing. Metadata like
    /// boost, omitNorm, IndexOptions, tokenized, etc., are not preserved.
    ///
    /// # Errors
    /// * [std::io::Error] with [std::io::ErrorKind::InvalidData] if the index is corrupt
    /// * [std::io::Error]  if there is a low-level IO error
    fn document(self: Pin<&mut Self>, doc_id: i32) -> Pin<Box<dyn Future<Output = IoResult<Box<Document>>>>>;
  
    /// Visits the fields of a stored document for custom processing/loading of each field. If you simply
    /// want to load all fields, use [StoredFields::document]. If you want to load a subset, use
    /// [DocumentStoredFieldVisitor].
    fn visit_fields(self: Pin<&mut Self>, doc_id: i32, visitor: Box<dyn StoredFieldVisitor>) -> Pin<Box<dyn Future<Output = IoResult<()>>>>;

    /// Like [StoredFields::document] but only loads the specified fields.
    /// 
    /// Note that this is simply sugar for [DocumentStoredFIeldVisitor::from_set].
    fn document_fields(self: Pin<&mut Self>, doc_id: i32, fields: &[&str]) -> Pin<Box<dyn Future<Output = IoResult<Document>>>>;
}
