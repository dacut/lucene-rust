use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::from_utf8,
};

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Term {
    field: String,
    bytes: Option<Vec<u8>>,
}

impl Term {
    /// Constructs a Term with the given field and bytes.
    pub fn new(field: &str, bytes: &[u8]) -> Self {
        Self {
            field: field.to_string(),
            bytes: Some(bytes.to_vec()),
        }
    }

    /// Constructs a Term with the given field and empty text. This serves two purposes:
    /// # Reuse of a Term with the same field.
    /// # Pattern for a query.
    pub fn new_empty(field: &str) -> Self {
        Self {
            field: field.to_string(),
            bytes: None,
        }
    }

    /// Returns the field of this term. The field indicates the part of a document which this term came
    /// from.
    #[inline]
    pub fn field(&self) -> &str {
        &self.field
    }

    /// Returns the text of this term. In the case of words, this is simply the text of the word. In
    /// the case of dates and other types, this is an encoding of the object as a string.
    pub fn text(&self) -> Option<String> {
        match self.bytes {
            None => None,
            Some(bytes) => match from_utf8(bytes.as_slice()) {
                Ok(s) => Some(s.to_string()),
                Err(_) => {
                    // Return the hex encoded bytes.
                    let mut result = String::with_capacity(bytes.len() * 3 + 2);
                    result.push('[');
                    for (i, byte) in bytes.iter().enumerate() {
                        if i > 0 {
                            result.push(' ');
                        }
                        result.push_str(&format!("{:02x}", byte));
                    }
                    result.push(']');
                    Some(result)
                }
            },
        }
    }

    /// Returns the bytes of this term
    pub fn bytes(&self) -> Option<&[u8]> {
        match self.bytes {
            None => None,
            Some(ref bytes) => Some(bytes.as_slice()),
        }
    }    
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self.bytes.as_ref() {
            Some(bytes) => match from_utf8(bytes.as_slice()) {
                Ok(s) => write!(f, "{}:{}", self.field, s),
                Err(_) => write!(f, "{}:{:?}", self.field, bytes),
            },
            None => write!(f, "{}:", self.field),
        }
    }
}
