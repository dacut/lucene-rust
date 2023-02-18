use std::{any::Any, fmt::Debug};

/// Trait for  attributes that can be added to a [crate::util::attribute_source::AttributeSource].
///
/// Attributes are used to add data in a dynamic, yet type-safe way to a source of usually
/// streamed objects, e.g. a [crate::analysis::token_stream::TokenStream].
pub trait Attribute: Any + Debug {
    /// Clears the values in this Attribute and resets it to its default value. If this
    /// implementation implements more than one Attribute interface it clears all.
    fn clear(&mut self) {}

    /// Clears the values in this AttributeImpl and resets it to its value at the end of the field. If
    /// this implementation implements more than one Attribute interface it clears all.
    fn end(&mut self) {
        self.clear()
    }

    /// Clone trait for [Attribute] that preserves object safety in Rust.
    fn clone_box(&self) -> Box<dyn Attribute>;
}
