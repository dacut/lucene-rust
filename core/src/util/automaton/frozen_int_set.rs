use {
    crate::util::automaton::{int_set::IntSet, state::State},
    std::hash::{Hash, Hasher},
};

pub(crate) struct FrozenIntSet {
    values: Vec<u32>,
    state: State,
    hash_code: u64,
}

impl FrozenIntSet {
    pub(crate) fn new(values: Vec<u32>, hash_code: u64, state: State) -> Self {
        Self {
            values,
            state,
            hash_code,
        }
    }

    pub(crate) fn get_state(&self) -> State {
        self.state
    }

    pub fn as_slice(&self) -> &[u32] {
        &self.values
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn values(&self) -> &[u32] {
        &self.values
    }
}

impl Hash for FrozenIntSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash_code);
    }
}

impl AsRef<[u32]> for FrozenIntSet {
    fn as_ref(&self) -> &[u32] {
        self.as_slice()
    }
}

impl From<FrozenIntSet> for IntSet {
    fn from(frozen: FrozenIntSet) -> Self {
        Self::Frozen(frozen)
    }
}
