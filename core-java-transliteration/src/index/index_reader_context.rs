use {
    crate::index::{composite_reader_context::CompositeReaderContext, leaf_reader_context::LeafReaderContext},
    std::{
        io::Result as IoResult,
        sync::{Arc, Weak},
    },
};

pub trait IndexReaderContext {
    /// The reader context for this reader's immediate parent, or `None` if none
    fn get_parent(self: Arc<Self>) -> Option<Weak<CompositeReaderContext>>;

    /// `true` if this context struct represents the top level reader within the hierarchical context
    fn is_top_level(self: Arc<Self>) -> bool;

    /// the doc base for this reader in the parent, `None` if parent is `None`.
    fn doc_base_in_parent(self: Arc<Self>) -> Option<i32>;

    /// the ord for this reader in the parent, `None` if parent is `None`.
    fn ord_in_parent(self: Arc<Self>) -> Option<i32>;

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
    fn leaves(self: Arc<Self>) -> IoResult<Vec<Arc<LeafReaderContext>>>;

    /// Returns the context's children iff this context is a composite context otherwise `None`.
    fn children(self: Arc<Self>) -> Option<Vec<Arc<CompositeReaderContext>>>;
}

/// Partial base implementation of [IndexReaderContext] for use by [LeafReaderContext] and
/// [CompositeReaderContext].
pub struct IndexReaderContextImpl {
    /// The reader context for this reader's immediate parent, or `None` if none
    parent: Option<Weak<CompositeReaderContext>>,

    /// The doc base for this reader in the parent, 0 if parent is `None`.
    doc_base_in_parent: i32,

    /// The ord for this reader in the parent, 0 if parent is `None`.
    ord_in_parent: i32,
}

impl IndexReaderContextImpl {
    pub fn new(
        parent: Option<Weak<CompositeReaderContext>>,
        doc_base_in_parent: i32,
        ord_in_parent: i32,
    ) -> Self {
        Self {
            parent,
            doc_base_in_parent,
            ord_in_parent,
        }
    }

    pub fn get_parent(&self) -> Option<Weak<CompositeReaderContext>> {
        self.parent
    }

    pub fn is_top_level(&self) -> bool {
        self.parent.is_none()
    }

    pub fn doc_base_in_parent(&self) -> Option<i32> {
        match self.parent {
            None => None,
            Some(_) => Some(self.doc_base_in_parent),
        }
    }

    pub fn ord_in_parent(&self) -> Option<i32> {
        match self.parent {
            None => None,
            Some(_) => Some(self.ord_in_parent),
        }
    }
}
