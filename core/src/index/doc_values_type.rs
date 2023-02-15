/// DocValues types. Note that DocValues is strongly typed, so a field cannot have different types
/// across different documents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocValuesType {
    /// No doc values for this field.
    None,

    /// A per-document Number.
    Numeric,

    /// A per-document Vec<u8>. Values may be larger than 32766 bytes, but different codecs may enforce
    /// their own limits.
    Binary,

    /// A pre-sorted Vec<u8>. The stored Vec<u8> is presorted and allows access via document id, ordinal
    /// and by-value. Values must be `<= 32766` bytes.
    Sorted,

    /// A pre-sorted Vec<Number>. Fields with this type store numeric values in sorted order.
    SortedNumeric,

    /// A pre-sorted BTreeSet<Vec<u8>>. The stored Vec<u8> is presorted and allows access via document id,
    /// ordinal and by-value. Values must be `code <= 32766` bytes.
    SortedSet,
}