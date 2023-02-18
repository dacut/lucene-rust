use {
    crate::util::{bit_util::NextHighestPowerOfTwo, hppc::bit_mixer::mix_phi_u32},
    std::{
        borrow::Borrow,
        cmp::{max, min},
        iter::IntoIterator,
        sync::atomic::{AtomicU32, Ordering},
    },
};

pub const DEFAULT_EXPECTED_ELEMENTS: usize = 4;
pub const DEFAULT_LOAD_FACTOR: f32 = 0.75;

static ITERATION_SEED: AtomicU32 = AtomicU32::new(0);

/// Minimal sane load factor (99 empty slots per 100)
pub const MIN_LOAD_FACTOR: f32 = 1.0 / 100.0;

/// Maximum sane load factor (1 empty slot per 100)
pub const MAX_LOAD_FACTOR: f32 = 99.0 / 100.0;

/// Minimum hash buffer size.
pub const MIN_HASH_ARRAY_LENGTH: usize = 4;

/// Maximum array size for hash containers (power-of-two and still allocable in Java, not a
/// negative int).
pub const MAX_HASH_ARRAY_LENGTH: usize = 0x80000000 >> 1;

/// A hash map of `u32` to `u32`, implemented using open addressing with linear
/// probing for collision resolution.
///
/// Mostly forked and trimmed from com.carrotsearch.hppc.IntIntHashMap
///
/// github: https://github.com/carrotsearch/hppc release 0.9.0
#[derive(Debug)]
pub struct IntIntHashMap {
    /// The array holding keys.
    pub keys: Vec<u32>,

    /// The array holding values.
    pub values: Vec<u32>,

    /// The number of stored keys (assigned key slots), excluding the special "empty" key, if any (use
    /// [IntIntHashMap::size] instead).
    pub(crate) assigned: u32,

    /// Mask for slot scans in [IntIntHashMap::keys].
    pub(crate) mask: u32, // ??

    /// Expand (rehash) [IntIntHashMap::keys] when [IntIntHashMap::assigned] hits this value.
    pub(crate) resize_at: u32,

    /// Special treatment for the "empty slot" key marker.
    pub(crate) has_empty_key: bool,

    /// The load factor for [IntIntHashMap::keys].
    pub(crate) load_factor: f32, // f64 in Java

    /// Seed used to ensure the hash iteration order is different from an iteration to another.
    pub(crate) iteration_seed: u32,
}

impl IntIntHashMap {
    /// New instance with sane defaults.
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_EXPECTED_ELEMENTS)
    }

    /// New instance with sane defaults.
    ///
    /// # Parameters
    /// * `expected_elements`: The expected number of elements guaranteed not to cause buffer expansion (inclusive).
    pub fn with_capacity(expected_elements: usize) -> Self {
        Self::with_capacity_and_load_factor(expected_elements, DEFAULT_LOAD_FACTOR)
    }

    /// New instance with the provided defaults.
    ///
    /// # Parameters
    /// * `expected_elements`: The expected number of elements guaranteed not to cause a rehash
    ///     (inclusive).
    /// * `load_factor`: The load factor for internal buffers. Insane load factors (zero, full
    ///     capacity) are rejected by [IntIntHashMap::verify_load_factor].
    pub fn with_capacity_and_load_factor(expected_elements: usize, load_factor: f32) -> Self {
        let load_factor = verify_load_factor(load_factor);
        let iteration_seed = ITERATION_SEED.fetch_add(1, Ordering::SeqCst);

        let mut result = Self {
            keys: Vec::with_capacity(expected_elements),
            values: Vec::with_capacity(expected_elements),
            assigned: 0,
            mask: 0,
            resize_at: 0,
            has_empty_key: false,
            load_factor,
            iteration_seed,
        };

        result.ensure_capacity(expected_elements);
        result
    }

    pub fn put(&mut self, key: u32, value: u32) -> Option<u32> {
        assert!(self.assigned < self.mask + 1);

        let mask = self.mask;
        let mut values = self.values.as_mut_slice();
        if key == 0 {
            self.has_empty_key = true;
            let previous_value = values[mask as usize + 1];
            self.values[mask as usize + 1] = value;
            return Some(previous_value);
        }

        let keys = self.keys.as_mut_slice();
        let slot = hash_key(key) & mask;

        loop {
            let mut existing = keys[slot as usize];
            if existing == 0 {
                break;
            }

            if existing == key {
                let previous_value = values[slot as usize];
                values[slot as usize] = value;
                return Some(previous_value);
            }

            slot = (slot + 1) & mask;
        }

        if self.assigned == self.resize_at {
            self.allocate_then_insert_then_rehash(slot, key, value);
        } else {
            keys[slot as usize] = key;
            values[slot as usize] = value;
        }

        self.assigned += 1;
        None
    }

    pub fn put_all<T>(&mut self, items: T)
    where
        T: IntoIterator,
        T::Item: Borrow<(u32, u32)>,
    {
        items.into_iter().for_each(|item| {
            let (key, value) = item.borrow();
            self.put(*key, *value);
        });
    }

    /// [Trove](http://trove4j.sourceforge.net)-inspired API method. An equivalent of the
    /// following code:
    ///
    /// ```ignore
    /// if !map.contains_key(key) {
    ///     map.put(key, value);
    /// }
    /// ```
    ///
    /// # Parameters
    /// * `key`: The key of the value to check.
    /// * `value`: The value to put if `key` does not exist.
    ///
    /// # Returns
    /// `true` if `key` did not exist and `value` was placed in the map.
    pub fn put_if_absent(&mut self, key: u32, value: u32) -> bool {
        let key_index = self.index_of(key);
        if !self.index_exists(key_index) {
            self.index_insert(key_index, key, value);
            true
        } else {
            false
        }
    }

    /// If `key` exists, `put_value` is inserted into the map, otherwise any existing value is incremented by
    /// `addition_value`.
    ///
    /// # Parameters
    /// * `key`: The key of the value to adjust.
    /// * `put_value`: The value to put if `key` does not exist.
    /// * `increment_value`: The value to add to the existing value if `key` exists.
    ///
    /// # Returns
    /// Returns the current value associated with `key` (after changes).
    pub fn put_or_add(&mut self, key: u32, mut put_value: u32, increment_value: u32) -> u32 {
        assert!(self.assigned < self.mask + 1);
        let values = self.values.as_mut_slice();

        let key_index = self.index_of(key);
        if self.index_exists(key_index) {
            put_value = values[key_index as usize] + increment_value;
            self.index_replace(key_index, put_value);
        } else {
            self.index_insert(key_index, key, put_value);
        }

        put_value
    }

    /// Adds `increment_value` to any existing value for the given `key` or
    /// inserts `increment_value` if `key` did not previously exist.
    ///
    /// # Parameters
    /// * `key`: The key of the value to adjust.
    /// * `increment_value`: The value to put or add to the existing value if `key` exists.
    ///
    /// # Returns
    /// Returns the current value associated with `key` (after changes).
    pub fn add_to(&mut self, key: u32, increment_value: u32) -> u32 {
        self.put_or_add(key, increment_value, increment_value)
    }

    pub fn remove(&mut self, key: u32) -> Option<u32> {
        let mask = self.mask;
        let values = self.values.as_mut_slice();
        if key == 0 {
            self.has_empty_key = false;
            let previous_value = values[mask as usize + 1];
            values[mask as usize + 1] = 0;
            return Some(previous_value);
        }

        let keys = self.keys.as_mut_slice();
        let slot = hash_key(key) & mask;

        loop {
            let existing = keys[slot as usize];
            if existing == 0 {
                return None;
            }

            if existing == key {
                let previous_value = values[slot as usize];
                self.shift_conflicting_keys(slot);
                return Some(previous_value);
            }

            slot = (slot + 1) & mask;
        }
    }

    pub fn get(&self, key: u32) -> Option<u32> {
        if key == 0 {
            if self.has_empty_key {
                Some(self.values[self.mask as usize + 1])
            } else {
                None
            }
        } else {
            let keys = self.keys.as_slice();
            let mask = self.mask;
            let mut slot = hash_key(key) & mask;
            loop {
                let existing = keys[slot as usize];
                if existing == 0 {
                    return None;
                }

                if existing == key {
                    return Some(self.values[slot as usize]);
                }

                slot = (slot + 1) & mask;
            }
        }
    }

    pub fn get_or_default(&self, key: u32, default_value: u32) -> u32 {
        self.get(key).unwrap_or(default_value)
    }

    pub fn contains_key(&self, key: u32) -> bool {
        self.get(key).is_some()
    }

    pub fn index_of(&self, key: u32) -> u32 {
        let mask = self.mask;

        if key == 0 {
            if self.has_empty_key {
                mask + 1
            } else {
                !(mask + 1)
            }
        } else {
            let keys = self.keys.as_slice();
            let mut slot = hash_key(key) & mask;
            loop {
                let existing = keys[slot as usize];
                if existing == 0 {
                    break;
                }

                if existing == key {
                    return slot;
                }

                slot = (slot + 1) & mask;
            }

            !slot
        }
    }

    pub fn index_exists(&self, index: u32) -> bool {
        assert!((index >= 0 && index <= self.mask) || (index == self.mask + 1 && self.has_empty_key));
        index >= 0
    }

    pub fn index_get(&self, index: u32) -> u32 {
        assert!((index < self.mask) || (index == self.mask + 1 && self.has_empty_key));
        self.values.as_slice()[index as usize]
    }

    pub fn index_replace(&mut self, index: u32, new_value: u32) -> u32 {
        assert!((index < self.mask) || (index == self.mask + 1 && self.has_empty_key));
        let values = self.values.as_mut_slice();
        let previous_value = values[index as usize];
        values[index as usize] = new_value;
        previous_value
    }

    pub fn index_insert(&mut self, index: u32, key: u32, value: u32) {
        let index = !index;
        let keys = self.keys.as_mut_slice();
        let values = self.values.as_mut_slice();

        if key == 0 {
            assert!(index == self.mask + 1);
            values[index as usize] = value;
            self.has_empty_key = true;
        } else {
            assert!(keys[index as usize] == 0);

            if self.assigned == self.resize_at {
                self.allocate_then_insert_then_rehash(index, key, value);
            } else {
                keys[index as usize] = key;
                values[index as usize] = value;
            }

            self.assigned += 1;
        }
    }

    pub fn index_remove(&mut self, index: u32) -> u32 {
        assert!((index <= self.mask) || (index == self.mask + 1 && self.has_empty_key));
        let values = self.values.as_mut_slice();
        let previous_value = values[index as usize];
        if index > self.mask {
            self.has_empty_key = false;
            values[index as usize] = 0;
        } else {
            self.shift_conflicting_keys(index);
        }

        previous_value
    }

    pub fn clear(&mut self) {
        self.assigned = 0;
        self.has_empty_key = false;
        self.keys.as_mut_slice().fill(0);
    }

    pub fn release(&mut self) {
        self.assigned = 0;
        self.has_empty_key = false;
        self.keys = Vec::new();
        self.values = Vec::new();
        self.ensure_capacity(DEFAULT_EXPECTED_ELEMENTS);
    }

    pub fn len(&self) -> usize {
        self.assigned as usize + if self.has_empty_key { 1 } else { 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Ensure this container can hold at least the given number of keys (entries) without resizing its
    /// buffers.
    ///
    /// # Parameters
    /// * `expected_elements`: The total number of keys, inclusive.
    pub fn ensure_capacity(&mut self, expected_elements: usize) {
        if expected_elements > self.resize_at as usize {
            let prev_keys = self.keys;
            let prev_values = self.values;
            self.keys = Vec::new();
            self.values = Vec::new();
            self.allocate_buffers(min_buffer_size(expected_elements, self.load_factor));
            if !self.is_empty() {
                self.rehash(&prev_keys, &prev_values);
            }
        }
    }

    /// Provides the next iteration seed used to build the iteration starting slot and offset
    /// increment. This method does not need to be synchronized, what matters is that each thread gets
    /// a sequence of varying seeds.
    pub fn next_iteration_seed(&self) -> u32 {
        self.iteration_seed = mix_phi_u32(self.iteration_seed);
        self.iteration_seed
    }

    /// Rehash from old buffers to new buffers.
    pub(crate) fn rehash(&mut self, from_keys: &[u32], from_values: &[u32]) {
        assert_eq!(from_keys.len(), from_values.len());
        assert!(check_power_of_two(from_keys.len() - 1));

        // Rehash all stored key/value pairs into the new buffers.
        let keys = self.keys;
        let values = self.values;
        let mask = self.mask;

        // Copy the zero element's slot, then rehash everything else.
        let mut from = from_keys.len() - 1;
        keys.as_mut_slice()[from] = from_keys[from];
        values.as_mut_slice()[from] = from_values[from];

        for from in (0..=(from-1)).rev() {
            let existing = from_keys[from];
            if existing != 0 {
                let mut slot = hash_key(existing) & mask;
                loop {
                    if keys.as_slice()[slot as usize] == 0 {
                        break;
                    }

                    slot = (slot + 1) & mask;
                }

                keys[slot as usize] = existing;
                values[slot as usize] = from_values[from];
            }
        }
    }

    /// Allocate new internal buffers. This method attempts to allocate and assign internal buffers automically
    /// (either allocations succeed or not).
    pub(crate) fn allocate_buffers(&mut self, array_size: usize) {
        assert!(array_size.is_power_of_two());
        assert!(array_size <= u32::MAX as usize);

        // Can't currently handle this in Rust.
        self.keys = vec![0; array_size + 1];
        self.values = vec![0; array_size + 1];
        self.resize_at = expand_at_count(array_size, self.load_factor) as u32;
        self.mask = (array_size - 1) as u32;
    }

    /// This method is invoked when there is a new key/ value pair to be inserted into the buffers but
    /// there is not enough empty slots to do so.
    ///
    /// New buffers are allocated. If this succeeds, we know we can proceed with rehashing so we
    /// assign the pending element to the previous buffer (possibly violating the invariant of having
    /// at least one empty slot) and rehash all keys, substituting new buffers at the end.
    pub(crate) fn allocate_then_insert_then_rehash(&mut self, slot: u32, pending_key: u32, pending_value: u32) {
        assert_eq!(self.assigned, self.resize_at);
        assert_eq!(self.keys.as_slice()[slot as usize], 0);
        assert_ne!(pending_key, 0);
        
        let prev_keys = self.keys;
        let prev_values = self.values;
        self.keys = Vec::new();
        self.values = Vec::new();

        // (Try to) allocate new buffers first. (If we OOM, we leave in a consistent state.) Not possible in Rust.
        self.allocate_buffers(next_buffer_size(self.mask as usize + 1, self.len(), self.load_factor));

        assert!(self.keys.len() > prev_keys.len());

        // We have succeeded at allocating new data so insert the pending key/value at
        // the free slot in the old arrays before rehashing.
        prev_keys.as_mut_slice()[slot as usize] = pending_key;
        prev_values.as_mut_slice()[slot as usize] = pending_value;

        // Rehash old keys, including the pending key.
        self.rehash(&prev_keys, &prev_values);
    }

    /// Shift all the slot-conflicting keys and values allocated to (and including) `slot`.
    pub(crate) fn shift_conflicting_keys(&mut self, gap_slot: u32) {
        let keys = self.keys;
        let values = self.values;
        let mask = self.mask;
    
        // Perform shifts of conflicting keys to fill in the gap.
        let mut distance = 0;
        loop {
            distance += 1;
            let slot = (gap_slot + (distance)) & mask;
            let existing = keys.as_slice()[slot as usize];
            if existing == 0 {
                break;
            }
    
            let ideal_slot = hash_key(existing);
            let shift = (slot - ideal_slot) & mask;

            if shift >= distance {
                // Entry at this position was originally at or before the gap slot.
                // Move the conflict-shifted entry to the gap's position and repeat the procedure
                // for any entries to the right of the current position, treating it
                // as the new gap.
                keys[gap_slot as usize] = existing;
                values[gap_slot as usize] = values[slot as usize];
                gap_slot = slot;
                distance = 0;
            }
    
        }
    }

    pub fn iter(&self) -> EntryIterator<'_> {
        EntryIterator::new(self)
    }
}

/// Validate load factor range and return it.
pub fn verify_load_factor(load_factor: f32) -> f32 {
    check_load_factor(load_factor, MIN_LOAD_FACTOR, MAX_LOAD_FACTOR);
    load_factor
}

/// Returns a hash code for the given key.
///
/// The output from this function should evenly distribute keys across the entire integer range.
pub(crate) fn hash_key(key: u32) -> u32 {
    assert!(key != 0); // Handled as a special case (empty slot marker).
    mix_phi_u32(key)
}

fn next_buffer_size(array_size: usize, elements: usize, load_factor: f32) -> usize {
    assert!(check_power_of_two(array_size));

    if array_size == MAX_HASH_ARRAY_LENGTH {
        panic!("Maximum array size exceeded for this load factor (elements: {elements}, load factor: {load_factor})");
    }

    array_size << 1
}

fn expand_at_count(array_size: usize, load_factor: f32) -> usize {
    assert!(check_power_of_two(array_size));

    // Take care of hash container invariant (there has to be at least one empty slot to ensure
    // the lookup loop finds either the element or an empty slot).
    min(array_size - 1, (array_size as f32 * load_factor).ceil() as usize)
}

fn check_power_of_two(array_size: usize) -> bool {
    assert!(array_size > 1);
    assert_eq!(array_size.next_highest_power_of_two(), array_size);
    true
}

fn min_buffer_size(elements: usize, load_factor: f32) -> usize {
    let mut length = ((elements as f32) / load_factor).ceil() as usize;
    if length == elements {
        length += 1;
    }

    length = max(MIN_HASH_ARRAY_LENGTH as usize, length.next_highest_power_of_two());

    if length > MAX_HASH_ARRAY_LENGTH {
        panic!("Maximum array size exceeded for this load factor (elements: {elements}, load factor: {load_factor})");
    }

    length
}

fn check_load_factor(load_factor: f32, min_allowed_inclusive: f32, max_allowed_inclusive: f32) {
    if load_factor < min_allowed_inclusive || load_factor > max_allowed_inclusive {
        panic!("The load factor should be in range [{min_allowed_inclusive:.2}, {max_allowed_inclusive:.2}]: {load_factor}");
    }
}

fn iteration_increment(seed: u32) -> u32 {
    ((seed & 7) << 1).wrapping_add(29)
}

/// An iterator implementation for [IntIntHashMap::iter]
pub struct EntryIterator<'a> {
    map: &'a IntIntHashMap,
    increment: u32,
    index: u32,
    slot: u32,
}

impl<'a> EntryIterator<'a> {
    pub fn new(map: &'a IntIntHashMap) -> Self {
        let seed = map.next_iteration_seed();
        let increment = iteration_increment(seed);

        Self {
            map,
            increment,
            index: 0,
            slot: seed & map.mask,
        }
    }
}

impl<'a> Iterator for EntryIterator<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        let mask = self.map.mask;
        while self.index < mask {
            self.index += 1;
            self.slot = (self.slot + self.increment) & mask;
            let existing = self.map.keys.as_slice()[self.slot as usize];
            if existing != 0 {
                return Some((existing, self.map.values.as_slice()[self.slot as usize]));                
            }

            if self.index == mask + 1 && self.map.has_empty_key {
                self.index += 1;
                let value = self.map.values.as_slice()[self.index as usize];
                return Some((0, value));
            }
        }

        None
    }
}

impl From<&[(u32, u32)]> for IntIntHashMap {
    fn from(items: &[(u32, u32)]) -> Self {
        let mut result = Self::new();
        result.put_all(items);
        result
    }
}
