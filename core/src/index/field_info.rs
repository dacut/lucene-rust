use {
    crate::index::{
        doc_values_type::DocValuesType,
        index_options::IndexOptions,
        point_values::{MAX_INDEX_DIMENSIONS, MAX_NUM_BYTES},
        vector_encoding::VectorEncoding,
        vector_similarity_function::VectorSimilarityFunction,
    },
    std::{
        collections::HashMap,
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
    },
};

/// Access to the Field Info file that describes document fields and whether or not they are indexed.
/// Each segment has a separate Field Info file.
#[derive(Clone, Debug)]
pub struct FieldInfo {
    /// Field's name
    pub name: String,

    /// Internal field number
    pub number: i32,

    doc_values_type: DocValuesType,

    /// True if any document indexed term vectors
    store_term_vector: bool,

    /// Omit norms associated with indexed fields
    omit_norms: bool,

    index_options: IndexOptions,

    /// Whether self field stores payloads together with term positions
    store_payloads: bool,

    attributes: HashMap<String, String>,

    dv_gen: Option<u64>,

    /// If both of these are positive it means self field indexed points.
    point_dimension_count: u32,

    point_index_dimension_count: u32,

    point_num_bytes: u32,

    /// If self is a positive value, it means self field indexes vectors.
    vector_dimension: u32,

    vector_encoding: Option<VectorEncoding>,

    vector_similarity_function: Option<VectorSimilarityFunction>,

    /// Whether self field is used as the soft-deletes field
    soft_deletes_field: bool,
}

impl FieldInfo {
    pub fn new(
        name: &str,
        number: i32,
        mut store_term_vector: bool,
        mut omit_norms: bool,
        mut store_payloads: bool,
        index_options: IndexOptions,
        doc_values: DocValuesType,
        dv_gen: Option<u64>,
        attributes: HashMap<String, String>,
        point_dimension_count: u32,
        point_index_dimension_count: u32,
        point_num_bytes: u32,
        vector_dimension: u32,
        vector_encoding: Option<VectorEncoding>,
        vector_similarity_function: Option<VectorSimilarityFunction>,
        soft_deletes_field: bool,
    ) -> IoResult<Self> {
        if matches!(index_options, IndexOptions::None) {
            store_term_vector = false;
            store_payloads = false;
            omit_norms = false;
        }

        let result = Self {
            name: name.to_string(),
            number,
            doc_values_type: doc_values,
            index_options,
            store_term_vector,
            store_payloads,
            omit_norms,
            dv_gen,
            attributes,
            point_dimension_count,
            point_index_dimension_count,
            point_num_bytes,
            vector_dimension,
            vector_encoding,
            vector_similarity_function,
            soft_deletes_field,
        };

        result.check_consistency()?;
        Ok(result)
    }

    /// Check correctness of the FieldInfo options
    pub fn check_consistency(&self) -> IoResult<()> {
        let name = self.name.as_str();

        // No need to check for null/none on self.index_options (not possible in Rust)

        if !matches!(self.index_options, IndexOptions::None) {
            // Cannot store payloads unless positions are indexed.
            if self.store_payloads && !self.index_options.positions_indexed() {
                return Err(IoError::new(
                    IoErrorKind::InvalidData,
                    format!("indexed field '{name}' cannot have payloads without positions"),
                ));
            }
        } else {
            if self.store_term_vector {
                return Err(IoError::new(
                    IoErrorKind::InvalidData,
                    format!("non-indexed field '{name}' cannot store term vectors"),
                ));
            }

            if self.store_payloads {
                return Err(IoError::new(
                    IoErrorKind::InvalidData,
                    format!("non-indexed field '{name}' cannot store payloads"),
                ));
            }

            if self.omit_norms {
                return Err(IoError::new(
                    IoErrorKind::InvalidData,
                    format!("non-indexed field '{name}' cannot omit norms"),
                ));
            }
        }

        // No need to check for null/none on self.doc_values_type (not possible in Rust)

        if self.dv_gen.is_none() && matches!(self.doc_values_type, DocValuesType::None) {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                format!("field '{name}' cannot have a docvalues update generation without having docvalues"),
            ));
        }

        // No need to check for negative values on point_dimension_count, point_index_dimension_count and point_num_bytes (not possible in Rust)

        if self.point_dimension_count != 0 && self.point_num_bytes == 0 {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                format!(
                    "point_num_bytes must be > 0 when point_dimension_count={} (field: '{name}')",
                    self.point_dimension_count
                ),
            ));
        }

        if self.point_index_dimension_count != 0 && self.point_dimension_count == 0 {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                format!("point_index_dimension_count must be 0 when point_dimension_count=0 (field: '{name}')"),
            ));
        }

        if self.point_num_bytes != 0 && self.point_dimension_count == 0 {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                format!(
                    "point_dimension_count must be > 0 when point_num_bytes={} (field: '{name}')",
                    self.point_num_bytes
                ),
            ));
        }

        // No need to check for null/none on vector_similarity_function (not possible in Rust)
        // No need to check for negative values on vector_dimension (not possible in Rust)

        Ok(())
    }

    /// Verify that the provided FieldInfo has the same schema as self FieldInfo
    pub fn verify_same_schema(&self, o: &Self) -> IoResult<()> {
        let field_name = self.name.as_str();
        verify_same_index_options(field_name, self.index_options, o.get_index_options())?;
        if matches!(self.index_options, IndexOptions::None) {
            verify_same_omit_norms(field_name, self.omit_norms, o.omit_norms)?;
            verify_same_store_term_vectors(field_name, self.store_term_vector, o.store_term_vector)?;
        }

        verify_same_doc_values_type(field_name, self.doc_values_type, o.doc_values_type)?;
        verify_same_points_options(
            field_name,
            self.point_dimension_count,
            self.point_index_dimension_count,
            self.point_num_bytes,
            o.point_dimension_count,
            o.point_index_dimension_count,
            o.point_num_bytes,
        )?;
        verify_same_vector_options(
            field_name,
            self.vector_dimension,
            self.vector_encoding,
            self.vector_similarity_function,
            o.vector_dimension,
            o.vector_encoding,
            o.vector_similarity_function,
        )
    }

    /// Record that this field is indexed with points, with the specified number of dimensions and bytes per dimension.
    pub fn set_point_dimensions(
        &mut self,
        dimension_count: u32,
        index_dimension_count: u32,
        num_bytes: u32,
    ) -> IoResult<()> {
        let name = self.name.as_str();

        if dimension_count == 0 {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                format!(r#"point_dimension_count must be > 0; got {dimension_count}) for field="{name}""#),
            ));
        }

        if index_dimension_count > MAX_INDEX_DIMENSIONS as u32 {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                format!(
                    r#"point_index_dimension_count must be <= PointValues::MAX_INDEX_DIMENSIONS (= {MAX_INDEX_DIMENSIONS}); got {index_dimension_count}) for field="{name}""#
                ),
            ));
        }

        if index_dimension_count > dimension_count {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                format!(
                    r#"point_index_dimension_count must be <= point_dimension_count (= {dimension_count}); got {index_dimension_count}) for field="{name}""#
                ),
            ));
        }

        if num_bytes == 0 {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                format!(r#"point_num_bytes must be > 0; got {num_bytes}) for field="{name}""#),
            ));
        }

        if num_bytes > MAX_NUM_BYTES as u32 {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                format!(
                    r#"point_num_bytes must be <= PointValues::MAX_NUM_BYTES (= {MAX_NUM_BYTES}); got {num_bytes}) for field="{name}""#
                ),
            ));
        }

        if self.point_dimension_count != 0 && self.point_dimension_count != dimension_count {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                format!(
                    r#"cannot change point dimension count from {} to {dimension_count} for field="{name}""#,
                    self.point_dimension_count
                ),
            ));
        }

        if self.point_index_dimension_count != 0 && self.point_index_dimension_count != index_dimension_count {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                format!(
                    r#"cannot change point index dimension count from {} to {index_dimension_count} for field="{name}""#,
                    self.point_index_dimension_count
                ),
            ));
        }

        if self.point_num_bytes != 0 && self.point_num_bytes != num_bytes {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                format!(
                    r#"cannot change point num bytes from {} to {num_bytes} for field="{name}""#,
                    self.point_num_bytes
                ),
            ));
        }

        self.point_dimension_count = dimension_count;
        self.point_index_dimension_count = index_dimension_count;
        self.point_num_bytes = num_bytes;

        self.check_consistency()
    }

    pub fn get_point_dimension_count(&self) -> u32 {
        self.point_dimension_count
    }

    pub fn get_point_index_dimension_count(&self) -> u32 {
        self.point_index_dimension_count
    }

    pub fn get_point_num_bytes(&self) -> u32 {
        self.point_num_bytes
    }

    pub fn get_vector_dimension(&self) -> u32 {
        self.vector_dimension
    }

    pub fn get_vector_encoding(&self) -> Option<VectorEncoding> {
        self.vector_encoding
    }

    pub fn get_vector_similarity_function(&self) -> Option<VectorSimilarityFunction> {
        self.vector_similarity_function
    }

    pub fn set_doc_values_type(&mut self, r#type: DocValuesType) -> IoResult<()> {
        if !matches!(self.doc_values_type, DocValuesType::None)
            && !matches!(r#type, DocValuesType::None)
            && self.doc_values_type != r#type
        {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                format!(
                    r#"cannot change doc values type from {:?} to {:?} for field="{}""#,
                    self.doc_values_type, r#type, self.name
                ),
            ));
        }

        self.doc_values_type = r#type;
        self.check_consistency()
    }

    pub fn get_index_options(&self) -> IndexOptions {
        self.index_options
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_field_number(&self) -> i32 {
        self.number
    }

    pub fn get_doc_values_type(&self) -> DocValuesType {
        self.doc_values_type
    }

    pub fn set_doc_values_gen(&mut self, dv_gen: Option<u64>) -> IoResult<()> {
        self.dv_gen = dv_gen;
        self.check_consistency()
    }

    pub fn get_doc_values_gen(&self) -> Option<u64> {
        self.dv_gen
    }

    pub fn set_store_term_vectors(&mut self) -> IoResult<()> {
        self.store_term_vector = true;
        self.check_consistency()
    }

    pub fn set_store_payloads(&mut self) -> IoResult<()> {
        if self.index_options.positions_indexed() {
            self.store_payloads = true;
        }
        self.check_consistency()
    }

    pub fn omits_norms(&self) -> bool {
        self.omit_norms
    }

    pub fn set_omits_norms(&mut self) -> IoResult<()> {
        if matches!(self.index_options, IndexOptions::None) {
            return Err(IoError::new(IoErrorKind::InvalidData, "cannot omit norms: this field is not indexed"));
        }
        self.omit_norms = true;
        self.check_consistency()
    }

    pub fn has_norms(&self) -> bool {
        return !matches!(self.index_options, IndexOptions::None) && !self.omit_norms;
    }

    pub fn has_payloads(&self) -> bool {
        self.store_payloads
    }

    pub fn has_vectors(&self) -> bool {
        self.store_term_vector
    }

    pub fn has_vector_values(&self) -> bool {
        self.vector_dimension > 0
    }

    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(|s| s.as_str())
    }

    /// Puts a codec attribute value.
    ///
    /// This is a key-value mapping for the field that the codec can use to store additional
    /// metadata, and will be available to the codec when reading the segment via
    /// [FieldInfo::get_attribute].
    ///
    /// If a value already exists for the key in the field, it will be replaced with the new value.
    /// If the value of the attributes for a same field is changed between the documents, the   behaviour
    /// after merge is undefined.
    pub fn put_attribute(&self, key: &str, value: &str) -> Option<String> {
        self.attributes.insert(key.to_string(), value.to_string())
    }

    /// Returns internal codec attributes map.
    pub fn attributes(&self) -> HashMap<String, String> {
        self.attributes.clone()
    }

    /// Returns true if this field is configured and used as the soft-deletes field. See
    /// [crate::index::index_writer_config::soft_deletes_field].
    pub fn is_soft_deletes_field(&self) -> bool {
        self.soft_deletes_field
    }
}

/// Verify that the provided index options are the same
pub(crate) fn verify_same_index_options(field_name: &str, a: IndexOptions, b: IndexOptions) -> IoResult<()> {
    if a != b {
        Err(IoError::new(
            IoErrorKind::InvalidData,
            format!(
                "cannot change field \"{field_name}\" from index options={a:?} to inconsistent index options={b:?}"
            ),
        ))
    } else {
        Ok(())
    }
}

/// Verify that the provided DocValuesTypes are the same.
pub(crate) fn verify_same_doc_values_type(field_name: &str, a: DocValuesType, b: DocValuesType) -> IoResult<()> {
    if a != b {
        Err(IoError::new(
            IoErrorKind::InvalidData,
            format!(
                "cannot change field \"{field_name}\" from doc values type={a:?} to inconsistent doc values type={b:?}"
            ),
        ))
    } else {
        Ok(())
    }
}

/// Verify that the provided store term vectors options are the same
pub(crate) fn verify_same_store_term_vectors(field_name: &str, a: bool, b: bool) -> IoResult<()> {
    if a != b {
        Err(IoError::new(
            IoErrorKind::InvalidData,
            format!("cannot change field \"{field_name}\" from store term vectors={a} to inconsistent store term vectors={b}"),
        ))
    } else {
        Ok(())
    }
}

/// Verify that the provided omit norms are the same
pub(crate) fn verify_same_omit_norms(field_name: &str, a: bool, b: bool) -> IoResult<()> {
    if a != b {
        Err(IoError::new(
            IoErrorKind::InvalidData,
            format!("cannot change field \"{field_name}\" from omit norms={a} to inconsistent omit norms={b}"),
        ))
    } else {
        Ok(())
    }
}

/// Verify that the provided points indexing options are the same.
pub(crate) fn verify_same_points_options(
    field_name: &str,
    a_point_dimension_count: u32,
    a_index_dimension_count: u32,
    a_num_bytes: u32,
    b_point_dimension_count: u32,
    b_index_dimension_count: u32,
    b_num_bytes: u32,
) -> IoResult<()> {
    if a_point_dimension_count != b_point_dimension_count
        || a_index_dimension_count != b_index_dimension_count
        || a_num_bytes != b_num_bytes
    {
        Err(IoError::new(
            IoErrorKind::InvalidData,
            format!("cannot change field \"{field_name}\" from points dimension count={a_point_dimension_count}, index dimension count={a_index_dimension_count}, num bytes={a_num_bytes} to inconsistent points dimension count={b_point_dimension_count}, index dimension count={b_index_dimension_count}, num bytes={b_num_bytes}"),
        ))
    } else {
        Ok(())
    }
}

/// Verify that the provided vector indexing options are the same.
pub(crate) fn verify_same_vector_options(
    field_name: &str,
    a_dim: u32,
    a_enc: Option<VectorEncoding>,
    a_vsf: Option<VectorSimilarityFunction>,
    b_dim: u32,
    b_enc: Option<VectorEncoding>,
    b_vsf: Option<VectorSimilarityFunction>,
) -> IoResult<()> {
    if a_dim != b_dim || a_enc != b_enc || a_vsf != b_vsf {
        Err(IoError::new(
            IoErrorKind::InvalidData,
            format!("cannot change field \"{field_name}\" from vector dimension={a_dim}, vector encoding={a_enc:?}, vector similarity function={a_vsf:?} to inconsistent vector dimension={b_dim}, vector encoding={b_enc:?}, vector similarity function={b_vsf:?}"),
        ))
    } else {
        Ok(())
    }
}
