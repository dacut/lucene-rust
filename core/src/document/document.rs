use {
    crate::index::indexable_field::IndexableField,
    std::{fmt::{Display, Formatter, Result as FmtResult}, slice::Iter as SliceIter},
};

/// Documents are the unit of indexing and search.
///
/// A Document is a set of fields. Each field has a name and a textual value. A field may be
/// [stored]([crate::index::indexable_field_type::IndexableFieldType::stored]) with the document, in which
/// case it is returned with search hits on the document. Thus each document should typically contain
/// one or more stored fields which uniquely identify it.
///
/// Note that fields which are _not_ are _not_ available in documents
/// retrieved from the index, e.g. with [ScoreDoc::doc] or [StoredFields::document].
pub struct Document {
    fields: Vec<Box<dyn IndexableField>>,
}

impl Document {
    pub fn iter(&self) -> SliceIter<'_, Box<dyn IndexableField>> {
        self.fields.iter()
    }

    /// Adds a field to a document. Several fields may be added with the same name. In this case, if
    /// the fields are indexed, their text is treated as though appended for the purposes of search.
    ///
    /// Note that add like the remove_field(s) methods only makes sense prior to adding a document to
    /// an index. These methods cannot be used to change the content of an existing index! In order to
    /// achieve this, a document has to be deleted from an index and a new changed version of that
    /// document has to be added.
    pub fn add(&mut self, field: Box<dyn IndexableField>) {
        self.fields.push(field);
    }

    /// Removes field with the specified name from the document. If multiple fields exist with this
    /// name, this method removes the first field that has been added. If there is no field with the
    /// specified name, the document remains unchanged.
    ///
    /// Note that the remove_field(s) methods like the add method only make sense prior to adding a
    /// document to an index. These methods cannot be used to change the content of an existing index!
    /// In order to achieve this, a document has to be deleted from an index and a new changed version
    /// of that document has to be added.
    /// 
    /// Returns `true` if a field has been removed, `false` otherwise. This differs from the Lucene
    /// Java implementation which returns void.
    pub fn remove_field(&mut self, name: &str) -> bool {
        for i in 0..self.fields.len() {
            if self.fields[i].name() == name {
                self.fields.swap_remove(i);
                return true;
            }
        }

        false
    }

    /// Removes all fields with the given name from the document. If there is no field with the
    /// specified name, the document remains unchanged.
    ///
    /// Note that the remove_field(s) methods like the add method only make sense prior to adding a
    /// document to an index. These methods cannot be used to change the content of an existing index!
    /// In order to achieve this, a document has to be deleted from an index and a new changed version
    /// of that document has to be added.
    /// 
    /// Returns the number of fields removed. This differs from the Lucene Java implementation which
    /// returns void.
    pub fn remove_fields(&mut self, name: &str) -> usize {
        let start_size = self.fields.len();
        self.fields.retain(|field| field.name() != name);
        start_size - self.fields.len()
    }

    /// Returns a Vec of byte slices for the fields that have the name specified as the method parameter.
    /// This method returns an empty Vec when there are no matching fields.
    pub fn get_binary_values(&self, name: &str) -> Vec<&[u8]> {
        self.fields
            .iter()
            .filter_map(|field| 
            if field.name() == name {
                field.binary_value()
            } else {
                None
            })
            .collect()
    }

    /// Returns an slice for the first (or only) field that has the name specified as the
    /// method parameter. This method will return `None` if no binary fields with the
    /// specified name are available. There may be non-binary fields with the same name.
    pub fn get_binary_value(&self, name: &str) -> Option<&[u8]> {
        for field in self.fields {
            if field.name() == name {
                return field.binary_value();
            }
        }

        None
    }
 
    /// Returns a Vec of [IndexableField] references with the given name. This method returns an
    /// empty Vec when there are no matching fields.
    pub fn get_field(&self, name: &str) -> Vec<&Box<dyn IndexableField>> {
        self.fields
            .iter()
            .filter(|field| field.name() == name)
            .collect()
    }

    /// Returns a Vec of all the fields in a document.
    /// 
    /// Note that fields which are _not_ stored are _not_ available in documents retrieved from the
    /// index, e.g., [StoredFields::document].
    pub fn get_fields(&self) -> Vec<&Box<dyn IndexableField>> {
        self.fields.iter().collect()
    }

    /// Returns an Vec of values of the field specified as the method parameter. This method returns
    /// an empty array when there are no matching fields. For a numeric [StoredField]
    /// it returns the string value of the number. If you want the actual numeric field
    /// instances back, use [Self::get_fields].
    pub fn get_values(&self, name: &str) -> Vec<&str> {
        self.fields.iter().filter_map(|field| {
            if field.name() == name {
                field.string_value()
            } else {
                None
            }
        }).collect()
    }

    /// Returns the string value of the field with the given name if any exist in this document, or
    /// `None`. If multiple fields exist with this name, this method returns the first value added. If
    /// only binary fields with this name exist, returns `None`. For a numeric [StoredField] it
    /// returns the string value of the number. If you want the actual numeric field instance back, use
    /// [Self::get_field].
    pub fn get(&self, name: &str) -> Option<&str> {
        for field in self.fields {
            if field.name() == name {
                return field.string_value();
            }
        }

        None
    }

    /// Removes all fields from the document.
    pub fn clear(&mut self) {
        self.fields.clear();
    }
}

impl Display for Document {
    /// Prints the fields of a document for human consumption.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Document<")?;

        for (i, field) in self.fields.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }

            write!(f, "{}", field)?;
        }

        write!(f, ">")
    }
}
