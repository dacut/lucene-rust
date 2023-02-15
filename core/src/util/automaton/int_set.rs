use {crate::util::automaton::{frozen_int_set::FrozenIntSet, state_set::StateSet}, std::hash::{Hash, Hasher}, std::collections::hash_map::DefaultHasher};

pub enum IntSet {
    Frozen(FrozenIntSet),
    State(StateSet),
}

impl IntSet {
    pub fn len(&self) -> usize {
        match self {
            IntSet::Frozen(frozen) => frozen.len(),
            IntSet::State(state_set) => state_set.len(),
        }
    }

    pub fn as_slice(&self) -> &[u32] {
        match self {
            IntSet::Frozen(frozen) => frozen.as_slice(),
            IntSet::State(state_set) => state_set.as_slice(),
        }
    }
}

impl Hash for IntSet {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        match self {
            IntSet::Frozen(frozen) => frozen.hash(hasher),
            IntSet::State(state_set) => state_set.hash(hasher),
        }
    }
}

impl PartialEq for IntSet {
    fn eq(&self, other: &Self) -> bool {
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        self.hash(&mut h1);
        other.hash(&mut h2);
        h1.finish() == h2.finish() && self.as_slice() == other.as_slice()
    }
}

impl Eq for IntSet {}