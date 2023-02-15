use {
    crate::index::{
        doc_values_type::DocValuesType,
        field_info::{
            verify_same_doc_values_type, verify_same_index_options, verify_same_omit_norms, verify_same_points_options,
            verify_same_store_term_vectors, verify_same_vector_options, FieldInfo,
        },
        index_options::IndexOptions,
        index_reader::IndexReader,
        vector_encoding::VectorEncoding,
        vector_similarity_function::VectorSimilarityFunction,
    },
    std::{
        collections::HashMap,
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
        iter::Iterator,
        sync::Arc,
    },
};

/// Collection of [FieldInfo]s (accessible by number or by name).
#[derive(Debug)]
pub struct FieldInfos {
    has_freq: bool,
    has_postings: bool,
    has_prox: bool,
    has_payloads: bool,
    has_offsets: bool,
    has_vectors: bool,
    has_norms: bool,
    has_doc_values: bool,
    has_point_values: bool,
    has_vector_values: bool,
    soft_deletes_field: Option<String>,

    // used only by field_info
    by_number: Vec<Option<FieldInfo>>,

    by_name: HashMap<String, FieldInfo>,
}

impl TryFrom<&[FieldInfo]> for FieldInfos {
    type Error = IoError;

    fn try_from(infos: &[FieldInfo]) -> IoResult<Self> {
        let mut has_vectors = false;
        let mut has_postings = false;
        let mut has_prox = false;
        let mut has_payloads = false;
        let mut has_offsets = false;
        let mut has_freq = false;
        let mut has_norms = false;
        let mut has_doc_values = false;
        let mut has_point_values = false;
        let mut has_vector_values = false;
        let mut soft_deletes_field = None;
        let mut size = 0;
        let mut by_number: Vec<Option<FieldInfo>> = Vec::with_capacity(10);
        let mut by_name = HashMap::new();

        for info in infos {
            if info.number < 0 {
                return Err(IoError::new(
                    IoErrorKind::InvalidData,
                    format!("invalid field number: {} for field {}", info.number, info.name),
                ));
            }

            if by_number.len() <= info.number as usize {
                by_number.resize(info.number as usize + 1, None);
            }

            if let Some(previous) = by_number[info.number as usize] {
                return Err(IoError::new(
                    IoErrorKind::InvalidData,
                    format!("duplicate field numbers: {} and {} have: {}", previous.name, info.name, info.number),
                ));
            }

            by_number[info.number as usize] = Some(info.clone());
            let previous = by_name.insert(info.name, info.clone());
            if let Some(previous) = previous {
                return Err(IoError::new(
                    IoErrorKind::InvalidData,
                    format!("duplicate field names: {} and {} have: {}", previous.number, info.number, info.name),
                ));
            }

            let index_options = info.get_index_options();
            has_vectors |= info.has_vectors();
            has_postings |= !matches!(index_options, IndexOptions::None);
            has_prox |= index_options.positions_indexed();
            has_freq |= !matches!(index_options, IndexOptions::Docs);
            has_offsets |= matches!(index_options, IndexOptions::DocsAndFreqsAndPositionsAndOffsets);
            has_norms |= info.has_norms();
            has_doc_values |= !matches!(info.get_doc_values_type(), DocValuesType::None);
            has_point_values |= info.get_point_dimension_count() != 0;
            has_vector_values |= info.get_vector_dimension() != 0;
            if info.is_soft_deletes_field() {
                if let Some(soft_deletes_field) = soft_deletes_field {
                    if soft_deletes_field != info.name {
                        return Err(IoError::new(
                            IoErrorKind::InvalidData,
                            format!("duplicate soft deletes fields [{}, {}]", soft_deletes_field, info.name),
                        ));
                    }
                } else {
                    soft_deletes_field = Some(info.name.clone());
                }
            }
        }

        Ok(Self {
            has_vectors,
            has_postings,
            has_prox,
            has_payloads,
            has_offsets,
            has_freq,
            has_norms,
            has_doc_values,
            has_point_values,
            has_vector_values,
            soft_deletes_field,
            by_number,
            by_name,
        })
    }
}

impl FieldInfos {
    /// Call this to get the (merged) FieldInfos for a composite reader.
    ///
    /// # Note
    /// The returned field numbers will likely not correspond to the actual field numbers in
    /// the underlying readers, and codec metadata [FieldInfo::get_attribute] will be
    /// unavailable.
    pub async fn get_merged_field_infos(reader: Arc<dyn IndexReader>) -> IoResult<Self> {
        let mut leaves = reader.leaves()?;

        match leaves.len() {
            0 => Ok(Self::empty()),
            1 => {
                let reader = leaves.pop().unwrap().get_reader();
                reader.get_field_infos().await
            }
            _ => {
                let mut soft_deletes_field = None;

                for ctx in leaves.clone() {
                    let reader = ctx.get_reader();
                    let field_infos = reader.get_field_infos().await?;
                    if let Some(sdf) = field_infos.get_soft_deletes_field() {
                        soft_deletes_field = Some(sdf.to_string());
                        break;
                    }
                }

                let mut builder = Builder::new(FieldNumbers::new(soft_deletes_field));
                for ctx in leaves {
                    let reader = ctx.get_reader();
                    for fi in reader.get_field_infos().await?.iter() {
                        builder.add(fi)?;
                    }
                }
                builder.finish()
            }
        }
    }

    /// Create a new, empty FieldInfos instance.
    pub fn empty() -> Self {
        let empty: [FieldInfo; 0] = [];
        FieldInfos::try_from(empty.as_slice()).unwrap()
    }

    /// Returns a set of names of fields that have a terms index. The order is undefined.
    pub async fn get_indexed_fields(reader: Arc<dyn IndexReader>) -> IoResult<Vec<String>> {
        let leaves = reader.leaves()?;
        let mut result = Vec::new();
        for ctx in leaves {
            let reader = ctx.get_reader();
            let field_infos = reader.get_field_infos().await?;
            for field_info in field_infos.iter() {
                if field_info.get_index_options() != IndexOptions::None {
                    result.push(field_info.get_name().to_string());
                }
            }
        }

        Ok(result)
    }

    /// Returns true if any fields have freqs
    pub fn has_freq(&self) -> bool {
        self.has_freq
    }

    /// Returns true if any fields have postings
    pub fn has_postings(&self) -> bool {
        self.has_postings
    }

    /// Returns true if any fields have positions
    pub fn has_prox(&self) -> bool {
        self.has_prox
    }

    /// Returns true if any fields have payloads
    pub fn has_payloads(&self) -> bool {
        self.has_payloads
    }

    /// Returns true if any fields have offsets
    pub fn has_offsets(&self) -> bool {
        self.has_offsets
    }

    /// Returns true if any fields have vectors
    pub fn has_vectors(&self) -> bool {
        self.has_vectors
    }

    /// Returns true if any fields have norms
    pub fn has_norms(&self) -> bool {
        self.has_norms
    }

    /// Returns true if any fields have doc values
    pub fn has_doc_values(&self) -> bool {
        self.has_doc_values
    }

    /// Returns true if any fields have point values
    pub fn has_point_values(&self) -> bool {
        self.has_point_values
    }

    /// Returns true if any fields have vector values
    pub fn has_vector_values(&self) -> bool {
        self.has_vector_values
    }

    /// Returns the soft deletes field name if exists; otherwise returns None.
    pub fn get_soft_deletes_field(&self) -> Option<&str> {
        self.soft_deletes_field.as_deref()
    }

    /// Returns the number of fields.
    pub fn size(&self) -> usize {
        self.by_name.len()
    }

    /// Returns an iterator over all the fieldinfo objects present, ordered by ascending field number.
    pub(crate) fn iter<'a>(&'a self) -> Iter<'a> {
        Iter {
            by_number: &self.by_number,
            pos: 0,
        }
    }

    /// Return the fieldinfo object referenced by the field name.
    pub fn get_field_info(&self, field: &str) -> Option<&FieldInfo> {
        self.by_name.get(field)
    }

    /// Return the fieldinfo object referenced by the fieldNumber.
    pub fn get_field_info_by_number(&self, field_number: i32) -> Option<&FieldInfo> {
        if field_number < 0 || field_number as usize >= self.by_number.len() {
            return None;
        }

        self.by_number[field_number as usize].as_ref()
    }
}

pub(crate) struct FieldDimensions {
    pub dimension_count: u32,
    pub index_dimension_count: u32,
    pub dimension_num_bytes: u32,
}

impl FieldDimensions {
    pub(crate) fn new(dimension_count: u32, index_dimension_count: u32, dimension_num_bytes: u32) -> Self {
        Self {
            dimension_count,
            index_dimension_count,
            dimension_num_bytes,
        }
    }
}

pub(crate) struct FieldVectorProperties {
    pub num_dimensions: u32,
    pub vector_encoding: Option<VectorEncoding>,
    pub similarity_function: Option<VectorSimilarityFunction>,
}

impl FieldVectorProperties {
    pub(crate) fn new(
        num_dimensions: u32,
        vector_encoding: Option<VectorEncoding>,
        similarity_function: Option<VectorSimilarityFunction>,
    ) -> Self {
        Self {
            num_dimensions,
            vector_encoding,
            similarity_function,
        }
    }
}

pub(crate) struct FieldNumbers {
    number_to_name: HashMap<u32, String>,
    name_to_number: HashMap<String, u32>,
    index_options: HashMap<String, IndexOptions>,

    // Used to enforced that a given field never changes DV type, even across segments/IndexWriter sessions
    doc_values_type: HashMap<String, DocValuesType>,
    dimensions: HashMap<String, FieldDimensions>,
    vector_props: HashMap<String, FieldVectorProperties>,
    omit_norms: HashMap<String, bool>,
    store_term_vectors: HashMap<String, bool>,

    lowest_unassigned_field_number: u32,

    // The soft-deletes field from IWC to enforce a single soft-deletes field
    soft_deletes_field_name: Option<String>,
}

impl FieldNumbers {
    pub(crate) fn new(soft_deletes_field_name: Option<String>) -> Self {
        Self {
            number_to_name: HashMap::new(),
            name_to_number: HashMap::new(),
            index_options: HashMap::new(),
            doc_values_type: HashMap::new(),
            dimensions: HashMap::new(),
            vector_props: HashMap::new(),
            omit_norms: HashMap::new(),
            store_term_vectors: HashMap::new(),
            lowest_unassigned_field_number: 0,
            soft_deletes_field_name,
        }
    }

    pub(crate) fn verify_field_info(&self, fi: &FieldInfo) -> IoResult<()> {
        let field_name = fi.get_name();
        self.verify_soft_deleted_field_name(field_name, fi.is_soft_deletes_field())?;

        if self.name_to_number.contains_key(field_name) {
            self.verify_same_schema(fi)?;
        }

        Ok(())
    }

    /// Returns the global field number for the given field name. If the name does not exist yet it
    /// tries to add it with the given preferred field number assigned if possible otherwise the
    /// first unassigned field number is used as the field number.
    pub(crate) fn add_or_get(&mut self, fi: &FieldInfo) -> IoResult<u32> {
        let field_name = fi.get_name();
        self.verify_soft_deleted_field_name(field_name, fi.is_soft_deletes_field()).unwrap();
        match self.name_to_number.get(field_name) {
            Some(field_number) => {
                self.verify_same_schema(fi)?;
                Ok(*field_number)
            }
            None => {
                // First time we've seen this field in this index.
                let preferred = fi.number;
                let field_number = if fi.number != -1 && self.number_to_name.contains_key(&(preferred as u32)) {
                    // We can use this number globally.
                    preferred as u32
                } else {
                    // find a new field number.
                    loop {
                        if self.number_to_name.contains_key(&self.lowest_unassigned_field_number) {
                            self.lowest_unassigned_field_number += 1;
                        } else {
                            break;
                        }
                    }
                    self.lowest_unassigned_field_number
                };

                self.number_to_name.insert(field_number, field_name.to_string());
                self.name_to_number.insert(field_name.to_string(), field_number);
                self.index_options.insert(field_name.to_string(), fi.get_index_options());
                if !matches!(fi.get_index_options(), IndexOptions::None) {
                    self.store_term_vectors.insert(field_name.to_string(), fi.has_vectors());
                    self.omit_norms.insert(field_name.to_string(), fi.omits_norms());
                }
                self.doc_values_type.insert(field_name.to_string(), fi.get_doc_values_type());
                self.dimensions.insert(
                    field_name.to_string(),
                    FieldDimensions::new(
                        fi.get_point_dimension_count(),
                        fi.get_point_index_dimension_count(),
                        fi.get_point_num_bytes(),
                    ),
                );
                self.vector_props.insert(
                    field_name.to_string(),
                    FieldVectorProperties::new(
                        fi.get_vector_dimension(),
                        fi.get_vector_encoding(),
                        fi.get_vector_similarity_function(),
                    ),
                );
                Ok(field_number)
            }
        }
    }

    fn verify_soft_deleted_field_name(&self, field_name: &str, is_soft_deletes_field: bool) -> IoResult<()> {
        if is_soft_deletes_field {
            if let Some(soft_deletes_field_name) = &self.soft_deletes_field_name {
                if soft_deletes_field_name != field_name {
                    Err(IoError::new(
                        IoErrorKind::InvalidData,
                        format!(
                            "cannot configure [{soft_deletes_field_name}] as soft-deletes; this index uses [{field_name}] as soft-deletes already"
                        ),
                    ))
                } else {
                    Ok(())
                }
            } else {
                Err(IoError::new(
                    IoErrorKind::InvalidData,
                    format!("this index has {field_name} as soft-deletes already but soft-deletes field is not configured in IWC")
                ))
            }
        } else if let Some(soft_deletes_field_name) = self.soft_deletes_field_name.as_ref() {
            if soft_deletes_field_name == field_name {
                Err(IoError::new(
                        IoErrorKind::InvalidData,
                        format!(
                            "cannot configure [{field_name}] as a regular field; this index uses [{field_name}] as soft-deletes already"
                        ),
                    ))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn verify_same_schema(&self, fi: &FieldInfo) -> IoResult<()> {
        let field_name = fi.get_name();
        let current_opts = self
            .index_options
            .get(field_name)
            .ok_or_else(|| IoError::new(IoErrorKind::InvalidData, format!("field [{field_name}] is not indexed")))?;
        verify_same_index_options(field_name, *current_opts, fi.get_index_options())?;

        if !matches!(current_opts, IndexOptions::None) {
            let cur_store_term_vector = self.store_term_vectors.get(field_name).ok_or_else(|| {
                IoError::new(IoErrorKind::InvalidData, format!("field [{field_name}] is not indexed"))
            })?;
            verify_same_store_term_vectors(field_name, *cur_store_term_vector, fi.has_vectors())?;
        }

        let cur_omit_norms = self
            .omit_norms
            .get(field_name)
            .ok_or_else(|| IoError::new(IoErrorKind::InvalidData, format!("field [{field_name}] is not indexed")))?;
        verify_same_omit_norms(field_name, *cur_omit_norms, fi.omits_norms())?;

        let current_dv_type = self
            .doc_values_type
            .get(field_name)
            .ok_or_else(|| IoError::new(IoErrorKind::InvalidData, format!("field [{field_name}] is not indexed")))?;
        verify_same_doc_values_type(field_name, *current_dv_type, fi.get_doc_values_type())?;

        let dims = self
            .dimensions
            .get(field_name)
            .ok_or_else(|| IoError::new(IoErrorKind::InvalidData, format!("field [{field_name}] is not indexed")))?;
        verify_same_points_options(
            field_name,
            dims.dimension_count,
            dims.index_dimension_count,
            dims.dimension_num_bytes,
            fi.get_point_dimension_count(),
            fi.get_point_index_dimension_count(),
            fi.get_point_num_bytes(),
        )?;

        let props = self
            .vector_props
            .get(field_name)
            .ok_or_else(|| IoError::new(IoErrorKind::InvalidData, format!("field [{field_name}] is not indexed")))?;
        verify_same_vector_options(
            field_name,
            props.num_dimensions,
            props.vector_encoding,
            props.similarity_function,
            fi.get_vector_dimension(),
            fi.get_vector_encoding(),
            fi.get_vector_similarity_function(),
        )
    }

    /// This function is called from [IndexWriter] to verify if doc values of the field can be
    /// updated. If the field with this name already exists, we verify that it is doc values only
    /// field. If the field doesn't exists and the parameter field_must_exist is false, we create a new
    /// field in the global field numbers.
    pub(crate) fn verify_or_create_dv_only_field(
        &mut self,
        field_name: &str,
        dv_type: DocValuesType,
        field_must_exist: bool,
    ) -> IoResult<()> {
        if !self.name_to_number.contains_key(field_name) {
            if field_must_exist {
                Err(IoError::new(
                    IoErrorKind::InvalidData,
                    format!("Can't update [{dv_type:?}] doc values; the field [{field_name}] doesn't exist"),
                ))
            } else {
                // create dv only field
                let fi = FieldInfo::new(
                    field_name,
                    -1,
                    false,
                    false,
                    false,
                    IndexOptions::None,
                    dv_type,
                    None,
                    HashMap::new(),
                    0,
                    0,
                    0,
                    0,
                    Some(VectorEncoding::Float32),
                    Some(VectorSimilarityFunction::Euclidean),
                    self.soft_deletes_field_name == Some(field_name.to_string()),
                )?;
                self.add_or_get(&fi);
                Ok(())
            }
        } else {
            // verify that field is doc values only field with the give doc values type
            let field_dv_type = self.doc_values_type.get(field_name).unwrap();
            if field_dv_type != &dv_type {
                return Err(IoError::new(
                    IoErrorKind::InvalidData,
                    format!("Can't update [{dv_type:?}] doc values; the field [{field_name}] has inconsistent doc values type of [{field_dv_type:?}]")
                ));
            }

            if let Some(fdimensions) = self.dimensions.get(field_name) {
                if fdimensions.dimension_count != 0 {
                    return Err(IoError::new(
                        IoErrorKind::InvalidData,
                        format!("Can't update [{dv_type:?}] doc values; the field [{field_name}] must be doc values only field, but is also indexed with points.")
                    ));
                }
            }

            if let Some(ioptions) = self.index_options.get(field_name) {
                if !matches!(ioptions, IndexOptions::None) {
                    return Err(IoError::new(
                        IoErrorKind::InvalidData,
                        format!("Can't update [{dv_type:?}] doc values; the field [{field_name}] must be doc values only field, but is also indexed with postings.")
                    ));
                }
            }

            if let Some(fvp) = self.vector_props.get(field_name) {
                if fvp.num_dimensions != 0 {
                    return Err(IoError::new(
                        IoErrorKind::InvalidData,
                        format!("Can't update [{dv_type:?}] doc values; the field [{field_name}] must be doc values only field, but is also indexed with vectors.")
                    ));
                }
            }

            Ok(())
        }
    }

    /// Construct a new FieldInfo based on the options in global field numbers.
    pub(crate) fn construct_field_info(
        &mut self,
        field_name: &str,
        dv_type: DocValuesType,
        new_field_number: i32,
    ) -> Option<FieldInfo> {
        let Some(field_number) = self.name_to_number.get(field_name) else {
            return None
        };

        let Some(dv_type0) = self.doc_values_type.get(field_name) else {
            return None
        };

        if dv_type0 != &dv_type {
            return None;
        }

        let is_soft_deletes_field = self.soft_deletes_field_name == Some(field_name.to_string());

        Some(
            FieldInfo::new(
                field_name,
                new_field_number,
                false,
                false,
                false,
                IndexOptions::None,
                dv_type,
                None,
                HashMap::new(),
                0,
                0,
                0,
                0,
                Some(VectorEncoding::Float32),
                Some(VectorSimilarityFunction::Euclidean),
                is_soft_deletes_field,
            )
            .unwrap(),
        )
    }
}

pub(crate) struct Builder {
    by_name: HashMap<String, FieldInfo>,
    global_field_numbers: FieldNumbers,
    finished: bool,
}

impl Builder {
    /// Creates a new instance with the given [FieldNumbers]
    pub(crate) fn new(global_field_numbers: FieldNumbers) -> Self {
        Self {
            by_name: HashMap::new(),
            global_field_numbers,
            finished: false,
        }
    }

    pub(crate) fn get_soft_deletes_field_name(&self) -> Option<&str> {
        self.global_field_numbers.soft_deletes_field_name.as_ref().map(|s| s.as_str())
    }

    /// Adds the provided FieldInfo to this Builder if this field doesn't exist in this Builder. Also
    /// adds a new field with its schema options to the global FieldNumbers if the field doesn't
    /// exist globally in the index. The field number is reused if possible for consistent field
    /// numbers across segments.
    ///
    /// If the field already exists:
    /// 1. the provided FieldInfo's schema is checked against the
    ///    existing field and
    /// 2. the provided FieldInfo's attributes are added to the existing
    /// FieldInfo's attributes.
    pub(crate) fn add(&mut self, field_info: &FieldInfo) -> IoResult<FieldInfo> {
        self.add_with_dvgen(field_info, None)
    }

    /// Adds the provided FieldInfo with the provided dvGen to this Builder if this field doesn't
    /// exist in this Builder. Also adds a new field with its schema options to the global
    /// FieldNumbers if the field doesn't exist globally in the index. The field number is reused if
    /// possible for consistent field numbers across segments.
    ///
    /// If the field already exists:
    /// 1. the provided FieldInfo's schema is checked against the
    ///    existing field and
    /// 2. the provided FieldInfo's attributes are added to the existing
    /// FieldInfo's attributes.
    pub(crate) fn add_with_dvgen(&mut self, fi: &FieldInfo, dv_gen: Option<u64>) -> IoResult<FieldInfo> {
        if let Some(mut cur_fi) = self.get_field_info_mut(fi.get_name()) {
            cur_fi.verify_same_schema(fi)?;
            fi.attributes().iter().for_each(|(k, v)| {
                cur_fi.put_attribute(k.as_str(), v.as_str());
            });
            if fi.has_payloads() {
                cur_fi.set_store_payloads();
            }
            Ok(cur_fi.clone())
        } else {
            // This field wasn't yet added to this in-RAM segment's FieldInfo,
            // so now we get a global number for this field.
            // If the field was seen before then we'll get the same name and number,
            // else we'll allocate a new one
            self.assert_not_finished()?;
            let field_number = self.global_field_numbers.add_or_get(fi)?;
            let fi_new = FieldInfo::new(
                fi.get_name(),
                field_number as i32,
                fi.has_vectors(),
                fi.omits_norms(),
                fi.has_payloads(),
                fi.get_index_options(),
                fi.get_doc_values_type(),
                dv_gen,
                fi.attributes(),
                fi.get_point_dimension_count(),
                fi.get_point_index_dimension_count(),
                fi.get_point_num_bytes(),
                fi.get_vector_dimension(),
                fi.get_vector_encoding(),
                fi.get_vector_similarity_function(),
                fi.is_soft_deletes_field(),
            )
            .unwrap();
            self.by_name.insert(fi_new.get_name().to_string(), fi_new.clone());
            Ok(fi_new)
        }
    }

    pub(crate) fn get_field_info(&self, field_name: &str) -> Option<&FieldInfo> {
        self.by_name.get(field_name)
    }

    pub(crate) fn get_field_info_mut(&mut self, field_name: &str) -> Option<&mut FieldInfo> {
        self.by_name.get_mut(field_name)
    }

    fn assert_not_finished(&self) -> IoResult<bool> {
        if self.finished {
            Err(IoError::new(IoErrorKind::InvalidData, "Builder was already finished; cannot add new fields"))
        } else {
            Ok(true)
        }
    }

    pub(crate) fn finish(self) -> IoResult<FieldInfos> {
        let field_infos: Vec<FieldInfo> = self.by_name.into_values().collect();
        FieldInfos::try_from(field_infos.as_slice())
    }
}

/// Iterator over the fields by number in a FieldInfos.
pub struct Iter<'a> {
    by_number: &'a Vec<Option<FieldInfo>>,
    pos: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a FieldInfo;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.by_number.len() {
            if let Some(ref field_info) = self.by_number[self.pos] {
                self.pos += 1;
                return Some(field_info);
            }

            self.pos += 1;
        }

        None
    }
}
