use std::{any::Any, fmt::Debug};

/// Rust-only enum to encapsulate the type and value for a field.
///
/// The Java implementation of Lucene defaults to java.lang.Object, sometimes via type erasure,
/// and a lot of dynamic dispatch that isn't possible with Rust's restrictions on object-safe
/// traits.
#[derive(Debug)]
pub enum FieldValue {
    /// Represents a string value.
    String(String),
    /// Represents an integer value.
    Int(i32),
    /// Represents a float value.
    Float(f32),
    /// Represents a long value.
    Long(i64),
    /// Represents a double value.
    Double(f64),
    /// Represents a custom value.
    Custom(Box<dyn CustomFieldValue>),
}

pub trait CustomFieldValue: Any + Debug {}
