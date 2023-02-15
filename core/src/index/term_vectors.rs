use {
    crate::index::{fields::Fields, terms::Terms},
    std::{future::Future, io::Result as IoResult, pin::Pin},
};

/// API for reading term vectors.
pub trait TermVectors {
    /// Returns term vectors for this document, or `None` if term vectors were not indexed.
    ///
    /// The returned Fields instance acts like a single-document inverted index (the docID will be
    /// 0). If offsets are available they are in an [OffsetAttribute] available from the [PostingsEnum].
    fn get(self: Pin<&mut Self>, doc: i32) -> Pin<Box<dyn Future<Output = IoResult<Option<Pin<Box<dyn Fields>>>>>>>;

    /// Retrieve term vector for this document and field, or `None` if term vectors were not indexed.
    ///
    /// The returned Terms instance acts like a single-document inverted index (the docID will be
    /// 0). If offsets are available they are in an {@link OffsetAttribute} available from the {@link
    /// PostingsEnum}.
    fn get_field(
        self: Pin<&mut Self>,
        doc: i32,
        field: &str,
    ) -> Pin<Box<dyn Future<Output = IoResult<Option<Pin<Box<dyn Terms>>>>>>> {
        let this = self;
        Box::pin(async move {
            let vectors = this.get(doc).await?;

            match vectors {
                Some(vectors) => Ok(vectors.as_ref().terms(field).await?),
                None => Ok(None),
            }
        })
    }
}
