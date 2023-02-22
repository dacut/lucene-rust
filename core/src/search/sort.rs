use {
    crate::{
        io::{CodecReadExt, CodecWriteExt},
        BoxResult, LuceneError,
    },
    std::{
        fmt::Debug,
        io::{Read, Write},
    },
};

/// Encapsulates sort criteria for returned hits.
#[derive(Debug)]
pub struct Sort {
    /// The directive that make up the sort.
    fields: Vec<Box<dyn SortField>>,
}

impl Sort {
    /// Create a new Sort taken from the given directives.
    ///
    /// If `directives` is empty, an error is returned.
    pub fn from_fields(fields: Vec<Box<dyn SortField>>) -> Result<Self, LuceneError> {
        if fields.is_empty() {
            Err(LuceneError::MissingSortDirectives)
        } else {
            Ok(Self {
                fields,
            })
        }
    }

    /// Creates a new Sort that sorts by computed relevance score.
    pub fn by_relevance() -> Self {
        Self {
            fields: vec![Box::new(BasicSortField::document_score())],
        }
    }

    /// Returns the fields used in this sort.
    pub fn get_fields(&self) -> &[Box<dyn SortField>] {
        &self.fields
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SortFieldType {
    /// Sort by document score (relevance). Sort values are `f32` values and higher values are at the front.
    DocumentScore,

    /// Sort by document number (index order). Sort values are Integer and lower values are at the front.
    DocumentIndexOrder,

    /// Sort using term values from a field as strings. Lower values are at the front.
    String,

    /// Sort using term values from a field as encoded `i32` values. Lower values are at the front. The
    /// field must either not be indexed or indexed with [I32Point].
    I32,

    /// Sort using term values from a field as encoded `f32` values. Lower values are at the front. The
    /// field must either not be indexed or indexed with [F32Point].
    F32,

    /// Sort using term values from a field as encoded `i64` values. Lower values are at the front. The
    /// field  must either not be indexed or indexed with [I64Point].
    I64,

    /// Sort using term values from a field as encoded `f64` values. Lower values are at the front. The
    /// field must either not be indexed or indexed with [F64Point].
    F64,

    /// Sort using a custom comparator. This is currently unimplemented in Rust.
    Custom,

    /// Sort using term values from a field as strings, but comparing by value using `std::str::cmp` for
    /// all comparisions instead of ordinals.
    StringVal,
}

impl SortFieldType {
    /// Reads the SortFieldType from the given stream.
    pub fn read_from(r: &mut dyn Read) -> BoxResult<Self> {
        let type_name = CodecReadExt::read_string(r)?;

        // Need to match on the Java enum name, not the Rust enum name.
        match type_name.as_str() {
            "SCORE" => Ok(Self::DocumentScore),
            "DOC" => Ok(Self::DocumentIndexOrder),
            "STRING" => Ok(Self::String),
            "INT" => Ok(Self::I32),
            "FLOAT" => Ok(Self::F32),
            "LONG" => Ok(Self::I64),
            "DOUBLE" => Ok(Self::F64),
            "CUSTOM" => Ok(Self::Custom),
            "STRING_VAL" => Ok(Self::StringVal),
            _ => Err(LuceneError::UnknownSortFieldType(type_name).into()),
        }
    }

    /// Writes the SortFieldType to the given stream.
    pub fn write_to(&self, w: &mut dyn Write) -> BoxResult<()> {
        // Need to match on the Java enum name, not the Rust enum name.
        let type_name = match self {
            Self::DocumentScore => "SCORE",
            Self::DocumentIndexOrder => "DOC",
            Self::String => "STRING",
            Self::I32 => "INT",
            Self::F32 => "FLOAT",
            Self::I64 => "LONG",
            Self::F64 => "DOUBLE",
            Self::Custom => "CUSTOM",
            Self::StringVal => "STRING_VAL",
        };

        Ok(CodecWriteExt::write_string(w, type_name)?)
    }
}

/// Stores information about how to sort documents. If a SortField includes a field, the field must be indexed in
/// order to sort by it.
///
/// Sorting on a numeric field that is indexed with both doc values and points may use an optimization to skip
/// non-competitive documents. This optimization relies on the assumption that the same data is stored in these points
/// and doc values.
///
/// Sorting on a Sorted or SortedSet field that is indexed with both doc values and term index may use an  optimization
/// to skip non-competitive documents. This optimization relies on the assumption that the same data is stored in these
/// term index and doc values.
pub trait SortField: Debug {
    /// Returns the type of sort.
    fn get_field_type(&self) -> SortFieldType;

    /// Returns the name of the field (if any) to sort by.
    fn get_field_name(&self) -> Option<&str>;

    /// Whether the relevance score is needed to sort documents.
    fn needs_score(&self) -> bool {
        matches!(self.get_field_type(), SortFieldType::DocumentScore)
    }

    /// Whether the sort order should be reversed.
    fn is_reverse(&self) -> bool;

    /// What to replace missing values with.
    fn missing_value(&self) -> Option<MissingValue>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MissingValue {
    String(StringMissingValue),
    I32(i32),
    F32(f32),
    I64(i64),
    F64(f64),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StringMissingValue {
    First,
    Last,
}

impl From<StringMissingValue> for MissingValue {
    fn from(value: StringMissingValue) -> Self {
        Self::String(value)
    }
}

impl From<i32> for MissingValue {
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl From<f32> for MissingValue {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}

impl From<i64> for MissingValue {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<f64> for MissingValue {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

/// A provider that can create sort field (directives) from streams.
pub trait SortFieldProvider: Debug {
    /// The name of the provider.
    fn get_name(&self) -> &str;

    /// Reads a sort field from the given stream.
    fn read_sort_field(&self, r: &mut dyn Read) -> BoxResult<Box<dyn SortField>>;

    /// Writes a sort field to the given stream.
    fn write_sort_field(&self, w: &mut dyn Write, directive: &dyn SortField) -> BoxResult<()>;
}

/// Stores information about how to sort documents by terms in an individual field. Fields must be indexed in order
/// to sort by them.
///
/// Sorting on a numeric field that is indexed with both doc values and points may use an optimization to skip
/// non-competitive documents. This optimization relies on the assumption that the same data is stored in these points
/// and doc values.
///
/// Sorting on a Sorted or SortedSet field that is indexed with both doc values and term index may use an  optimization
/// to skip non-competitive documents. This optimization relies on the assumption that the same data is stored in these
/// term index and doc values.
#[derive(Debug)]
pub struct BasicSortField {
    field_type: SortFieldType,
    field_name: Option<String>,
    reverse: bool,
    missing_value: Option<MissingValue>,
}

impl BasicSortField {
    /// Create a new BasicSortField for a document score.
    pub fn document_score() -> Self {
        Self {
            field_type: SortFieldType::DocumentScore,
            field_name: None,
            reverse: false,
            missing_value: None,
        }
    }

    /// Create a new BasicSortField for a document index order.
    pub fn document_index_order() -> Self {
        Self {
            field_type: SortFieldType::DocumentIndexOrder,
            field_name: None,
            reverse: false,
            missing_value: None,
        }
    }

    /// Create a new BasicSortField for a string field.
    pub fn for_string_field(field_name: &str, missing_value: Option<StringMissingValue>) -> Self {
        let missing_value = missing_value.map(|sf| sf.into());

        Self {
            field_type: SortFieldType::String,
            field_name: Some(field_name.to_string()),
            reverse: false,
            missing_value,
        }
    }

    /// Create a new BasicSortField for a 32-bit integer field.
    pub fn for_i32_field(field_name: &str, missing_value: Option<i32>) -> Self {
        let missing_value = missing_value.map(|sf| sf.into());

        Self {
            field_type: SortFieldType::I32,
            field_name: Some(field_name.to_string()),
            reverse: false,
            missing_value,
        }
    }

    /// Create a new BasicSortField for a 32-bit float field.
    pub fn for_f32_field(field_name: &str, missing_value: Option<f32>) -> Self {
        let missing_value = missing_value.map(|sf| sf.into());

        Self {
            field_type: SortFieldType::F32,
            field_name: Some(field_name.to_string()),
            reverse: false,
            missing_value,
        }
    }

    /// Create a new BasicSortField for a 64-bit integer field.
    pub fn for_i64_field(field_name: &str, missing_value: Option<i64>) -> Self {
        let missing_value = missing_value.map(|sf| sf.into());

        Self {
            field_type: SortFieldType::I64,
            field_name: Some(field_name.to_string()),
            reverse: false,
            missing_value,
        }
    }

    /// Create a new BasicSortField for a 64-bit float field.
    pub fn for_f64_field(field_name: &str, missing_value: Option<f64>) -> Self {
        let missing_value = missing_value.map(|sf| sf.into());

        Self {
            field_type: SortFieldType::F64,
            field_name: Some(field_name.to_string()),
            reverse: false,
            missing_value,
        }
    }

    /// Create a new BasicSortField for a string field using the `std::string::cmp` comparator.
    pub fn for_string_val_field(field_name: &str) -> Self {
        Self {
            field_type: SortFieldType::StringVal,
            field_name: Some(field_name.to_string()),
            reverse: false,
            missing_value: None,
        }
    }

    /// Update the reverse flag.
    pub fn set_reverse(&mut self, reverse: bool) {
        self.reverse = reverse;
    }
}

impl SortField for BasicSortField {
    fn get_field_type(&self) -> SortFieldType {
        self.field_type
    }

    fn get_field_name(&self) -> Option<&str> {
        self.field_name.as_deref()
    }

    fn is_reverse(&self) -> bool {
        self.reverse
    }

    fn missing_value(&self) -> Option<MissingValue> {
        self.missing_value
    }
}

#[derive(Debug, Default)]
pub struct BasicSortDirectiveProvider {}

impl SortFieldProvider for BasicSortDirectiveProvider {
    fn get_name(&self) -> &str {
        "SortField"
    }

    fn read_sort_field(&self, r: &mut dyn Read) -> BoxResult<Box<dyn SortField>> {
        let field_name = r.read_string()?;
        let field_type = SortFieldType::read_from(r)?;
        let is_reverse = CodecReadExt::read_vi32(r)? == 1;
        let has_missing_value = CodecReadExt::read_vi32(r)? == 1;
        let mut sort_field = match field_type {
            SortFieldType::String => {
                let missing_value = if has_missing_value {
                    let order = CodecReadExt::read_vi32(r)?;
                    Some(if order == 1 {
                        StringMissingValue::First
                    } else {
                        StringMissingValue::Last
                    })
                } else {
                    None
                };

                BasicSortField::for_string_field(
                    &field_name,
                    missing_value,
                )
            }

            SortFieldType::I32 => {
                let missing_value = if has_missing_value {
                    Some(CodecReadExt::read_vi32(r)?)
                } else {
                    None
                };

                BasicSortField::for_i32_field(
                    &field_name,
                    missing_value,
                )
            }

            SortFieldType::F32 => {
                let missing_value = if has_missing_value {
                    Some(f32::from_bits(CodecReadExt::read_vi32(r)? as u32))
                } else {
                    None
                };

                BasicSortField::for_f32_field(
                    &field_name,
                    missing_value,
                )
            }

            SortFieldType::I64 => {
                let missing_value = if has_missing_value {
                    Some(CodecReadExt::read_vi64(r)?)
                } else {
                    None
                };

                BasicSortField::for_i64_field(
                    &field_name,
                    missing_value,
                )
            }

            SortFieldType::F64 => {
                let missing_value = if has_missing_value {
                    Some(f64::from_bits(CodecReadExt::read_vi64(r)? as u64))
                } else {
                    None
                };

                BasicSortField::for_f64_field(
                    &field_name,
                    missing_value,
                )
            }

            SortFieldType::StringVal => {
                if has_missing_value {
                    return Err(LuceneError::InvalidSortField("SortField of type StringVal cannot have a missing value".to_string()).into());
                };

                BasicSortField::for_string_val_field(
                    &field_name,
                )
            }

            SortFieldType::DocumentScore => {
                if has_missing_value {
                    return Err(LuceneError::InvalidSortField("SortField of type DocumentScore cannot have a missing value".to_string()).into());
                }

                BasicSortField::document_score()
            }

            SortFieldType::DocumentIndexOrder => {
                if has_missing_value {
                    return Err(LuceneError::InvalidSortField("SortField of type DocumentIndex cannot have a missing value".to_string()).into());
                }

                BasicSortField::document_index_order()
            }

            SortFieldType::Custom => {
                if has_missing_value {
                    return Err(LuceneError::InvalidSortField("SortField of type Custom cannot have a field name".to_string()).into());
                }

                unimplemented!("Custom sort fields are not implemented")
            }
        };
        sort_field.set_reverse(is_reverse);
        Ok(Box::new(sort_field))
    }

    fn write_sort_field(&self, w: &mut dyn Write, field: &dyn SortField) -> BoxResult<()> {
        w.write_string(field.get_field_name().unwrap_or(""))?;
        let field_type = field.get_field_type();
        field_type.write_to(w)?;
        w.write_vi32(if field.is_reverse() { 1 } else { 0 })?;
        match field.missing_value() {
            None => w.write_vi32(0)?,
            Some(missing_value) => match field_type {
                SortFieldType::String => {
                    let mv = match missing_value {
                        MissingValue::String(StringMissingValue::Last) => 0,
                        MissingValue::String(StringMissingValue::First) => 1,
                        _ => return Err(LuceneError::InvalidSortField("Invalid missing value for SortField of type String".to_string()).into()),
                    };
                    w.write_vi32(1)?;
                    w.write_vi32(mv)?;
                }
                SortFieldType::I32 => {
                    let mv = match missing_value {
                        MissingValue::I32(mv) => mv,
                        _ => return Err(LuceneError::InvalidSortField("Invalid missing value for SortField of type I32".to_string()).into()),
                    };
                    w.write_vi32(1)?;
                    w.write_vi32(mv)?;
                }
                SortFieldType::F32 => {
                    let mv = match missing_value {
                        MissingValue::F32(mv) => mv.to_bits() as i32,
                        _ => return Err(LuceneError::InvalidSortField("Invalid missing value for SortField of type F32".to_string()).into()),
                    };
                    w.write_vi32(1)?;
                    w.write_vi32(mv)?;
                }
                SortFieldType::I64 => {
                    let mv = match missing_value {
                        MissingValue::I64(mv) => mv,
                        _ => return Err(LuceneError::InvalidSortField("Invalid missing value for SortField of type I64".to_string()).into()),
                    };
                    w.write_vi32(1)?;
                    w.write_vi64(mv)?;
                }
                SortFieldType::F64 => {
                    let mv = match missing_value {
                        MissingValue::F64(mv) => mv.to_bits() as i64,
                        _ => return Err(LuceneError::InvalidSortField("Invalid missing value for SortField of type F64".to_string()).into()),
                    };
                    w.write_vi32(1)?;
                    w.write_vi64(mv)?;
                }
                _ => return Err(LuceneError::InvalidSortField(format!("SortField of type {field_type:?} cannot have a missing value")).into()),
            }
        }
        Ok(())
    }
}

#[allow(clippy::box_default)]
pub fn get_sort_field_provider(name: &str) -> Result<Box<dyn SortFieldProvider>, LuceneError> {
    match name {
        "SortField" => Ok(Box::new(BasicSortDirectiveProvider::default())),
        "SortedNumericSortField" => unimplemented!("SortedNumericSortField is not implemented"),
        "SortedSetSortField" => unimplemented!("SortedSetSortField is not implemented"),
        _ => Err(LuceneError::UnknownSortFieldProvider(name.to_string())),
    }
}
