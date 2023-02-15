use {
    crate::{
        analysis::{analyzer::Analyzer, token_stream::TokenStream},
        index::indexable_field_type::IndexableFieldType,
        util::number::Number,
    },
    std::fmt::Display,
    tokio::io::AsyncRead,
};

/// Represents a single field for indexing.
pub trait IndexableField: Display {
    /// Field name
    fn name(&self) -> String;

    /// [IndexableFieldType] describing the properties of this field.
    fn field_type(&self) -> Box<dyn IndexableFieldType>;

    /// Creates the TokenStream used for indexing this field. If appropriate, implementations should
    /// use the given Analyzer to create the TokenStreams.
    ///
    /// # Parameters
    /// * `analyzer`: Analyzer that should be used to create the TokenStreams from
    /// * `reuse` TokenStream for a previous instance of this field _name_. This allows custom
    ///   field types (like StringField and NumericField) that do not use the analyzer to still have
    ///   good performance. Note: the passed-in type may be inappropriate, for example if you mix up
    ///   different types of Fields for the same field name. So it's the responsibility of the
    ///   implementation to check.
    ///
    /// # Returns
    /// [TokenStream] value for indexing the document. Should always return a non-`None` value if
    ///     the field is to be indexed
    fn token_stream(&self, analyzer: Box<dyn Analyzer>, reuse: Option<Box<dyn TokenStream>>) -> Option<Box<dyn TokenStream>>;

    /// `Some` if this field has a binary value.
    fn binary_value(&self) -> Option<&[u8]>;

    /// `Some` if this field has a string value.
    fn string_value(&self) -> Option<&str>;

    /// `Some` if this reader has an AsyncRead value.
    fn reader_value(&self) -> Option<Box<dyn AsyncRead + Unpin>>;

    /// `Some` if this field has a numeric value.
    fn numeric_value(&self) -> Option<Number>;
}
