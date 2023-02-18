use {
    crate::util::attribute::Attribute,
    std::{any::TypeId, collections::HashMap},
};

/// An AttributeSource contains a list of different Attributes, and methods to add and
/// get them.
pub trait AttributeSource {
    /// Adds an attribute to this AttributeSource.
    ///
    /// If an attribute of the existing type already exists, it will be replaced.
    fn add_attribute(&mut self, attribute: Box<dyn Attribute>);

    /// Returns an attribute of the given type, or None if no such attribute exists.
    fn get_attribute(&self, r#type: TypeId) -> Option<&dyn Attribute>;

    /// Returns true if this AttributeSource has any attributes.
    fn has_attributes(&self) -> bool;

    /// Returns true if this AttributeSource has an attribute of the given type.
    fn has_attribute(&self, r#type: TypeId) -> bool;

    /// Resets all Attributes in this AttributeSource by calling [Attribute::clear] on each
    /// Attribute implementation.
    fn clear_attributes(&mut self);

    /// Resets all Attributes in this AttributeSource by calling [Attribute::end] on each
    /// Attribute implementation.
    fn end_attributes(&mut self);

    /// Removes all attributes and their implementations from this AttributeSource.
    fn remove_all_attributes(&mut self);

    /// Returns the current state of all attributes.
    fn get_state(&self) -> HashMap<TypeId, Box<dyn Attribute>>;

    /// Restores the state of all attributes.
    fn set_state(&mut self, state: HashMap<TypeId, Box<dyn Attribute>>);
}

/// Base implementation of AttributeSource.
#[derive(Debug, Default)]
pub struct AttributeSourceBase {
    attributes: HashMap<TypeId, Box<dyn Attribute>>,
}

impl AttributeSourceBase {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AttributeSource for AttributeSourceBase {
    fn add_attribute(&mut self, attribute: Box<dyn Attribute>) {
        self.attributes.insert(attribute.type_id(), attribute);
    }

    fn get_attribute(&self, r#type: TypeId) -> Option<&dyn Attribute> {
        self.attributes.get(&r#type).map(|a| a.as_ref())
    }

    fn has_attributes(&self) -> bool {
        !self.attributes.is_empty()
    }

    fn has_attribute(&self, r#type: TypeId) -> bool {
        self.attributes.contains_key(&r#type)
    }

    fn clear_attributes(&mut self) {
        for value in self.attributes.values() {
            value.clear();
        }
    }

    fn end_attributes(&mut self) {
        for value in self.attributes.values() {
            value.end();
        }
    }

    fn remove_all_attributes(&mut self) {
        self.attributes.clear();
    }

    fn get_state(&self) -> HashMap<TypeId, Box<dyn Attribute>> {
        let state = HashMap::with_capacity(self.attributes.len());
        for (key, value) in self.attributes.iter() {
            state.insert(*key, value.clone_box());
        }
        state
    }

    fn set_state(&mut self, state: HashMap<TypeId, Box<dyn Attribute>>) {
        self.attributes = state;
    }
}
