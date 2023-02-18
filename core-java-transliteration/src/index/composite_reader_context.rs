use {
    crate::index::{
        composite_reader::CompositeReader,
        index_reader_context::{IndexReaderContext, IndexReaderContextImpl},
        leaf_reader_context::LeafReaderContext,
    },
    std::{
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
        sync::{Arc, Weak},
    },
};

pub struct CompositeReaderContext {
    index_reader_context: IndexReaderContextImpl,
    children: Vec<Arc<Self>>,
    leaves: Vec<Arc<LeafReaderContext>>,
    reader: Arc<dyn CompositeReader>,
}

// FIXME: add impl
// FIXME: add IndexReaderContext impl

impl IndexReaderContext for CompositeReaderContext {
    /// The reader context for this reader's immediate parent, or `None` if none
    fn get_parent(self: Arc<Self>) -> Option<Weak<CompositeReaderContext>> {
        self.index_reader_context.get_parent()
    }

    /// `true` if this context struct represents the top level reader within the hierarchical context
    fn is_top_level(self: Arc<Self>) -> bool {
        self.index_reader_context.is_top_level()
    }

    /// the doc base for this reader in the parent, `None` if parent is `None`.
    fn doc_base_in_parent(self: Arc<Self>) -> Option<i32> {
        self.index_reader_context.doc_base_in_parent()
    }

    /// the ord for this reader in the parent, `None` if parent is `None`.
    fn ord_in_parent(self: Arc<Self>) -> Option<i32> {
        self.index_reader_context.ord_in_parent()
    }

    /// Returns the [IndexReader], this context represents.
    // fn get_reader(&self) -> Arc<Pin<Box<dyn IndexReader>>>;

    /// Returns the context's leaves if this context is a top-level context. For convenience, if this
    /// is a [LeafReaderContext] this returns itself as the only leaf.
    ///
    /// # Note
    /// This is convenience method since leaves can always be obtained by walking the context
    /// tree using [IndexReaderContext::children].
    ///
    /// # Errors
    /// Returns Error(std::io::IoError(UnsupportedOperationException)) if this is not a top-level context.
    fn leaves(self: Arc<Self>) -> IoResult<Vec<Arc<LeafReaderContext>>> {
        if !self.is_top_level() {
            Err(IoError::new(IoErrorKind::Unsupported, "not a top-level context"))
        } else {
            Ok(self.leaves.clone())
        }
    }

    /// Returns the context's children iff this context is a composite context otherwise `None`.
    fn children(self: Arc<Self>) -> Option<Vec<Arc<CompositeReaderContext>>> {
        Some(self.children.clone())
    }
}
