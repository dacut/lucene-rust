use {
    crate::index::{
        doc_values_type::DocValuesType, index_options::IndexOptions, vector_encoding::VectorEncoding,
        vector_similarity_function::VectorSimilarityFunction,
    },
    std::collections::HashMap,
};

/// Describes the properties of a field.
pub trait IndexableFieldType {
    /// True if the field's value should be stored.
    fn stored(&self) -> bool;

    /// True if this field's value should be analyzed by the [Analyzer].
    ///
    /// This has no effect if [Self::index_options] returns [IndexOptions::None].
    fn tokenized(&self) -> bool;

    /// True if this field's indexed form should be also stored into term vectors.
    ///
    /// This builds a miniature inverted-index for this field which can be accessed in a
    /// document-oriented way from [crate::index::term_vectors::TermVectors::get_field]
    ///
    /// This option is illegal if [Self::index_options] returns [IndexOptions::None].
    fn store_term_vectors(&self) -> bool;

    /// True if this field's token character offsets should also be stored into term vectors.
    ///
    /// This option is illegal if term vectors are not enabled for the field
    /// ([Self::store_term_vectors] returns false).
    fn store_term_vector_offsets(&self) -> bool;

    /// True if this field's token positions should also be stored into the term vectors.
    ///
    /// This option is illegal if term vectors are not enabled for the field
    /// ([Self::store_term_vectors] returns false).
    fn store_term_vector_positions(&self) -> bool;

    /// True if this field's token payloads should also be stored into the term vectors.
    ///
    /// This option is illegal if term vector positions are not enabled for the field
    /// [Self::store_term_vectors] returns false].
    fn store_term_vector_payloads(&self) -> bool;

    /// True if normalization values should be omitted for the field.
    ///
    /// This saves memory, but at the expense of scoring quality (length normalization will be
    /// disabled), and if you omit norms, you cannot use index-time boosts.
    fn omit_norms(&self) -> bool;

    /// [IndexOptions] describing what should be recorded into the inverted index.
    fn index_options(&self) -> IndexOptions;

    /// [DocValues]([DocValuesType]): how the field's value will be indexed into docValues
    fn doc_values_type(&self) -> DocValuesType;

    /// If this is positive (representing the number of point dimensions), the field is indexed as a point.
    fn point_dimension_count(&self) -> u32;

    /// The number of dimensions used for the index key.
    fn point_index_dimension_count(&self) -> u32;

    /// The number of bytes in each dimension's values.
    fn point_num_bytes(&self) -> usize;

    /// The number of dimensions of the field's vector value.
    fn vector_dimension(&self) -> u32;

    /// The [VectorEncoding] of the field's vector value.
    fn vector_encoding(&self) -> Option<VectorEncoding>;

    /// The [VectorSimilarityFunction] of the field's vector value.
    fn vector_similarity_function(&self) -> Option<VectorSimilarityFunction>;

    /// Attributes for the field type.
    fn get_attributes(&self) -> HashMap<String, String>;
}
