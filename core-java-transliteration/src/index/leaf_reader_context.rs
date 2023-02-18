use {
    crate::index::{
        composite_reader_context::CompositeReaderContext,
        index_reader_context::{IndexReaderContext, IndexReaderContextImpl},
        leaf_reader::LeafReader,
    },
    std::{
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
        sync::{Arc, Weak},
    },
};

/// [IndexReaderContext] for [LeafReader] instances.
pub struct LeafReaderContext {
    index_reader_context: IndexReaderContextImpl,

    /// The reader's ord in the top-level's leaves array
    pub ord: i32,

    /// The reader's absolute doc base
    pub doc_base: i32,

    reader: Arc<dyn LeafReader>,

    // FIXME: This should probably be Weak
    leaves: Vec<Arc<LeafReaderContext>>,
}

impl LeafReaderContext {
    /// Creates a new [LeafReaderContext].
    pub fn new(
        parent: Option<Weak<CompositeReaderContext>>,
        reader: Arc<dyn LeafReader>,
        ord: i32,
        doc_base: i32,
        leaf_ord: i32,
        leaf_doc_base: i32,
    ) -> Arc<Self> {
        let lrc = Self {
            index_reader_context: IndexReaderContextImpl::new(parent, doc_base, ord),
            ord: leaf_ord,
            doc_base: leaf_doc_base,
            reader,
            leaves: Vec::new(),
        };

        let lrc = Arc::new(lrc);

        if lrc.index_reader_context.is_top_level() {
            lrc.leaves.push(lrc);
        }

        lrc
    }

    pub fn get_reader(self: Arc<Self>) -> Arc<dyn LeafReader> {
        self.reader.clone()
    }
}

impl IndexReaderContext for LeafReaderContext {
    fn get_parent(self: Arc<Self>) -> Option<Weak<CompositeReaderContext>> {
        self.index_reader_context.get_parent().clone()
    }

    fn is_top_level(self: Arc<Self>) -> bool {
        self.index_reader_context.is_top_level()
    }

    fn doc_base_in_parent(self: Arc<Self>) -> Option<i32> {
        self.index_reader_context.doc_base_in_parent()
    }

    fn ord_in_parent(self: Arc<Self>) -> Option<i32> {
        self.index_reader_context.ord_in_parent()
    }

    fn leaves(self: Arc<Self>) -> IoResult<Vec<Arc<LeafReaderContext>>> {
        if self.is_top_level() {
            Ok(self.leaves.clone())
        } else {
            Err(IoError::new(IoErrorKind::Unsupported, "This context is not a top-level context"))
        }
    }

    /// Returns the context's children iff this context is a composite context otherwise `None`.
    fn children(self: Arc<Self>) -> Option<Vec<Arc<CompositeReaderContext>>> {
        None
    }
}
