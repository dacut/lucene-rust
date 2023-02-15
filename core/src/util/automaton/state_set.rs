use {
    crate::util::{
        automaton::{frozen_int_set::FrozenIntSet, int_set::IntSet, state::State},
        hppc::int_int_hash_map::IntIntHashMap,
    },
    std::cell::{Cell, RefCell},
    std::hash::{Hash, Hasher},
};

/// A thin wrapper of {@link IntIntHashMap} Maps from state in integer representation to its
/// reference count. Whenever the count of a state is 0, that state will be removed from the set
#[derive(Debug)]
pub(crate) struct StateSet {
    inner: IntIntHashMap,
    hash_code: u64,
    hash_updated: bool,
    array_updated: Cell<bool>,
    array_cache: RefCell<Vec<u32>>,
}

impl StateSet {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: IntIntHashMap::with_capacity(capacity),
            hash_code: 0,
            hash_updated: true,
            array_updated: Cell::new(true),
            array_cache: RefCell::new(Vec::new()),
        }
    }

    /// Add the state into this set. If it is already there, increase its reference count by 1.
    pub fn incr(&mut self, state: State) {
        if self.inner.add_to(state.0, 1) == 1 {
            self.key_changed()
        }
    }

    /// Decrease the reference count of the state. If the count decreases to 0, remove the state from this
    /// set
    pub fn decr(&mut self, state: State) {
        assert!(self.inner.contains_key(state.0));

        let key_index = self.inner.index_of(state.0);
        let count = self.inner.index_get(key_index) - 1;
        if count == 0 {
            self.inner.index_remove(key_index);
            self.key_changed();
        } else {
            self.inner.index_replace(key_index, count);
        }
    }

    pub fn reset(&mut self) {
        self.inner.clear();
        self.key_changed();
    }

    /// Create a snapshot of this int set associated with a given state. The snapshot will not retain
    /// any frequency information about the elements of this set, only existence.
    ///
    /// # Parameters
    /// * `state`: the state to associate with the frozen set.
    pub fn freeze(&self, state: State) -> FrozenIntSet {
        self.update_array_cache();
        FrozenIntSet::new(self.as_slice().to_vec(), self.hash_code, state)
    }

    #[inline]
    fn key_changed(&mut self) {
        self.hash_updated = false;
        self.array_updated = Cell::new(false);
    }

    fn update_array_cache(&self) {
        if !self.array_updated.get() {
            let mut array_cache = Vec::with_capacity(self.inner.len());

            for (_, value) in self.inner.iter() {
                array_cache.push(value);
            }

            // We need to sort this array since the equality method depends on this.
            array_cache.sort_unstable();
            self.array_cache.replace(array_cache);
            self.array_updated.set(true);
        }
    }

    /// Return a slice of this int set's values.
    pub fn as_slice(&self) -> &[u32] {
        self.update_array_cache();
        self.array_cache.borrow().as_slice()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Hash for StateSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash_code);
    }
}

impl From<StateSet> for IntSet {
    fn from(state_set: StateSet) -> Self {
        Self::State(state_set)
    }
}
