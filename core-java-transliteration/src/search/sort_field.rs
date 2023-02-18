use {
    crate::{
        index::{
            doc_values::get_sorted,
            index_sorter::{IndexSorter, SortedDocValuesProvider, StringSorter},
            leaf_reader::LeafReader,
            sorted_doc_values::SortedDocValues,
        },
        search::{field_comparator_source::FieldComparatorSource, index_searcher::IndexSearcher},
    },
    std::{any::Any, cmp::Ordering, fmt::Debug, future::Future, io::Result as IoResult, pin::Pin, sync::Arc},
};

/// Represents sorting by document score (relevance).
pub const FIELD_SCORE: Box<SortFieldBase> = Box::new(SortFieldBase::new(None, Type::Score, false));

/// Represents sorting by document number (index order).
pub const FIELD_DOC: Box<SortFieldBase> = Box::new(SortFieldBase::new(None, Type::Doc, false));

/// Specifies the type of the terms to be sorted, or special types such as Custom
#[derive(Debug)]
pub enum Type {
    /// Sort by document score (relevance). Sort values are Float and higher values are at the front.
    Score,

    /// Sort by document number (index order). Sort values are Integer and lower values are at the
    /// front.
    Doc,

    /// Sort using term values as Strings. Sort values are String and lower values are at the front.
    String,

    /// Sort using term values as encoded Integers (i32). Sort values are Integer and lower values are at
    /// the front. Fields must either be not indexed, or indexed with [IntPoint].
    Int,

    /// Sort using term values as encoded Floats (f32). Sort values are Float and lower values are at the
    /// front. Fields must either be not indexed, or indexed with [FloatPoint].
    Float,

    /// Sort using term values as encoded Longs (i64). Sort values are Long and lower values are at the
    /// front. Fields must either be not indexed, or indexed with [LongPoint].
    Long,

    /// Sort using term values as encoded Doubles (f64). Sort values are Double and lower values are at the
    /// front. Fields must either be not indexed, or indexed with [DoublePoint].
    Double,

    /// Sort using a custom Comparator. Sort values are any Comparable and sorting is done according
    /// to natural order.
    Custom(Option<Box<dyn FieldComparatorSource>>),

    /// Sort using term values as Strings, but comparing by value (using [str::cmp]) for all
    /// comparisons. This is typically slower than [::String], which uses ordinals to do the
    /// sorting.
    StringVal,

    /// Force rewriting of SortField using [SortField::rewrite] before it can be
    /// used for sorting
    Rewriteable,
}

/// The trait bounds for custom field types.
pub trait CustomField: Any + Debug {
    /// Compare this field against another field of the same type.
    ///
    /// # Panic
    /// This method will panic if other is not of the same type as self.
    fn cmp(&self, other: Box<dyn CustomField>) -> Ordering;
}

/// This is a Rust workaround for Lucene's use of a Java Object for the missing value.
#[derive(Debug)]
pub enum MissingValue {}

/// Stores information about how to sort documents by terms in an individual field. Fields must be
/// indexed in order to sort by them.
///
/// Sorting on a numeric field that is indexed with both doc values and points may use an
/// optimization to skip non-competitive documents. This optimization relies on the assumption that
/// the same data is stored in these points and doc values.
///
/// Sorting on a Sorted/SortedSet field that is indexed with both doc values and term index may use an
/// optimization to skip non-competitive documents. This optimization relies on the assumption that
/// the same data is stored in these term index and doc values.
pub trait SortField: Debug {
    /// Returns the name of the field. Could return `None` if the sort is by Score or Doc.
    fn get_field(&self) -> Option<&str>;

    /// Returns the type of contents in the field.
    fn get_type(&self) -> Type;

    /// Returns whether the sort should be reversed.
    fn get_reverse(&self) -> bool;

    /// Rewrites this SortField.
    /// Implementations should override this define their rewriting behavior
    /// when this SortField is of type [Type::Rewriteable]
    fn rewrite(&mut self, searcher: &IndexSearcher) -> IoResult<()> {
        Ok(())
    }

    /// Whether the relevance score is needed to sort documents.
    fn needs_scores(&self) -> bool {
        matches!(self.get_type(), Type::Score)
    }

    /// Returns an [IndexSorter] used for sorting index segments by this SortField.
    ///
    /// If the SortField cannot be used for index sorting (for example, if it uses scores or other
    /// query-dependent values) then this method should return `None`.
    ///
    /// RUST FIXME:  
    /// SortFields that implement this method should also implement a companion [SortFieldProvider]
    /// to serialize and deserialize the sort in index segment headers
    fn get_index_sorter(&self) -> Option<Box<dyn IndexSorter>>;
}

const SORT_FIELD_PROVIDER_NAME: &str = "SortField";

#[derive(Debug)]
pub struct SortFieldBase {
    field: Option<String>,
    r#type: Type,
    reverse: bool,

    /// Used for sort_missing_first/sort_missing_last
    missing_value: Option<MissingValue>,

    /// Indicates if sort should be optimized with indexed data. Set to true by default.
    #[deprecated]
    optimized_sort_with_indexed_data: bool,
}

impl SortFieldBase {
    pub const fn new(field: Option<String>, r#type: Type, reverse: bool) -> Self {
        if field.is_none() && !matches!(r#type, Type::Score | Type::Doc) {
            panic!("field can only be None when type is Type::Score or Type::Doc");
        }

        #[allow(deprecated)]
        Self {
            field,
            r#type,
            reverse,
            missing_value: None,
            optimized_sort_with_indexed_data: true,
        }
    }
}

impl SortField for SortFieldBase {
    #[inline]
    fn get_field(&self) -> Option<&str> {
        self.field.as_ref().map(|s| s.as_str())
    }

    #[inline]
    fn get_type(&self) -> Type {
        self.r#type
    }

    #[inline]
    fn get_reverse(&self) -> bool {
        self.reverse
    }

    fn get_index_sorter(&self) -> Option<Box<dyn IndexSorter>> {
        match self.r#type {
            Type::String => {
                struct LeafReaderSortedDocValuesProvider {
                    field: String,
                }

                impl SortedDocValuesProvider for LeafReaderSortedDocValuesProvider {
                    fn get(
                        self: Pin<&Self>,
                        reader: Arc<dyn LeafReader>,
                    ) -> Pin<Box<dyn Future<Output = IoResult<Pin<Box<dyn SortedDocValues>>>>>> {
                        let field = self.field.clone();
                        Box::pin(async move { get_sorted(reader, field).await })
                    }
                }
                let sdvp = LeafReaderSortedDocValuesProvider {
                    field: self.field.clone(),
                };
                StringSorter::new(SORT_FIELD_PROVIDER_NAME, self.missing_value, self.reverse, sdvp)
            }
            _ => todo!(),
        }
    }
}
