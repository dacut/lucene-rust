use {
    crate::util::{
        long_values::LongValues,
        packed::packed_ints::{bits_required, check_block_size, get_mutable, NullReader, Reader},
    },
    std::{
        cmp::{max, min},
        fmt::Debug,
        io::Result as IoResult,
        iter::Iterator,
        marker::PhantomData,
        ops::Index,
        ptr::NonNull,
    },
};

pub(crate) const DEFAULT_PAGE_SIZE: u32 = 256;
const MIN_PAGE_SIZE: u32 = 64;

// More than 1M doesn't really makes sense with these appending buffers
// since their goal is to try to have small numbers of bits per value
const MAX_PAGE_SIZE: u32 = 1 << 20;

pub trait PackedLongValues: LongValues + Debug {
    type Iter<'a>
    where
        Self: 'a;

    /// Get the number of values in this array.
    fn size(&self) -> usize;
    fn decode_block(&self, block: u32, dest: &mut [i64]) -> usize;
    fn get(&self, block: u32, element: usize) -> i64;
    fn iter(&self) -> Self::Iter<'_>;

    fn values_slice(&self) -> &[Box<dyn Reader>];
    fn get_page_shift(&self) -> u32;
    fn get_page_mask(&self) -> u32;
}

#[derive(Debug)]
pub struct BasicPackedLongValues {
    pub(crate) values: Vec<Box<dyn Reader>>,
    pub(crate) page_shift: u32,
    pub(crate) page_mask: u32,
    pub(crate) size: usize,
}

impl BasicPackedLongValues {
    pub(crate) fn new(page_shift: u32, page_mask: u32, values: Vec<Box<dyn Reader>>, size: usize) -> Self {
        Self {
            page_shift,
            page_mask,
            values,
            size,
        }
    }
}

impl PackedLongValues for BasicPackedLongValues {
    type Iter<'a> = Iter<'a, Self>;

    #[inline]
    fn size(&self) -> usize {
        self.size
    }

    fn decode_block(&self, block: u32, dest: &mut [i64]) -> usize {
        let vals = self.values.as_slice()[block as usize];
        let size = vals.size();
        let mut k = 0;

        while k < size && k < dest.len() {
            k += vals.get_range(k, &mut dest[k..]);
        }

        k
    }

    #[inline]
    fn get(&self, block: u32, element: usize) -> i64 {
        self.values.as_slice()[block as usize].get(element)
    }

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        Iter::new(self)
    }

    #[inline]
    fn values_slice(&self) -> &[Box<dyn Reader>] {
        self.values.as_slice()
    }

    #[inline]
    fn get_page_shift(&self) -> u32 {
        self.page_shift
    }

    #[inline]
    fn get_page_mask(&self) -> u32 {
        self.page_mask
    }
}

impl LongValues for BasicPackedLongValues {}

impl Index<usize> for BasicPackedLongValues {
    type Output = i64;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.size());
        let block = index >> self.page_shift;
        let element = index & self.page_mask as usize;
        &PackedLongValues::get(self, block as u32, element)
    }
}

/// An iterator over packed long values.
#[derive(Debug)]
pub struct Iter<'a, T> {
    long_values: NonNull<T>,
    current_values: Vec<i64>,
    v_off: u32,
    p_off: usize,
    current_count: usize, // number of entries of the current page,
    _phantom: PhantomData<&'a T>,
}

unsafe impl<'a, T> Send for Iter<'a, T> where T: Send {}
unsafe impl<'a, T> Sync for Iter<'a, T> where T: Sync {}

impl<'a, T> Iter<'a, T>
where
    T: PackedLongValues,
{
    pub fn new(long_values: &'a T) -> Self {
        let result = Self {
            long_values: long_values.into(),
            current_values: Vec::with_capacity(long_values.get_page_mask() as usize + 1),
            v_off: 0,
            p_off: 0,
            current_count: 0,
            _phantom: PhantomData,
        };

        result.fill_block();
        result
    }

    fn fill_block(&mut self) {
        let lv = unsafe { self.long_values.as_ref() };
        if self.v_off == lv.values_slice().len() as u32 {
            self.current_count = 0;
        } else {
            self.current_count = lv.decode_block(self.v_off, &mut self.current_values);
            assert!(self.current_count > 0);
        }
    }

    fn has_next(&self) -> bool {
        self.p_off < self.current_count
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: PackedLongValues,
{
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.has_next() {
            None
        } else {
            let result = self.current_values[self.p_off];
            self.p_off += 1;

            if self.p_off == self.current_count {
                self.v_off += 1;
                self.p_off = 0;
                self.fill_block();
            }

            Some(result)
        }
    }
}

const INITIAL_PAGE_COUNT: usize = 16;

/// A builder for a [PackedLongValues] instance.
#[derive(Debug)]
pub struct Builder {
    pub(crate) page_shift: u32,
    pub(crate) page_mask: u32,
    pub(crate) acceptable_overhead_ratio: f32,
    pub(crate) values: Vec<Box<dyn Reader>>,
    pub(crate) pending: Vec<i64>,
    pub(crate) size: usize,
}

impl Builder {
    pub fn new(page_size: u32, acceptable_overhead_ratio: f32) -> IoResult<Self> {
        let page_shift = check_block_size(page_size, MIN_PAGE_SIZE, MAX_PAGE_SIZE)?;
        let page_mask = page_shift - 1;
        let values = Vec::with_capacity(INITIAL_PAGE_COUNT);
        let pending = Vec::with_capacity(page_size as usize);

        Ok(Self {
            page_shift,
            page_mask,
            acceptable_overhead_ratio,
            values,
            pending,
            size: 0,
        })
    }

    /// Build a [BasicPackedLongValues] instance that contains values that have been added to this builder.
    pub fn build(self) -> BasicPackedLongValues {
        self.finish();
        BasicPackedLongValues::new(self.page_shift, self.page_mask, self.values, self.size)
    }

    /// Add a new element to this builder.
    pub fn add(&mut self, l: i64) {
        if self.pending.len() == self.pending.capacity() {
            self.pack_one();
        }

        self.pending.push(l);
        self.size += 1;
    }

    pub(crate) fn finish(&mut self) {
        self.pack_one();
    }

    pub(crate) fn pack_one(&mut self) {
        self.pack(&self.pending, self.acceptable_overhead_ratio);
        // Reset pending buffer.
        self.pending.clear();
    }

    pub(crate) fn pack(&mut self, values: &[i64], acceptable_overhead_ratio: f32) {
        assert!(values.len() > 0);

        // Compute max delta
        let mut min_value = values[0];
        let mut max_value = min_value;
        for value in values[1..].iter() {
            min_value = min(min_value, *value);
            max_value = max(max_value, *value);
        }

        // Build a new packed reader
        if min_value == 0 && max_value == 0 {
            self.values.push(Box::new(NullReader::new(values.len() as u32)));
        } else {
            let bits_required = if min_value < 0 { 64 } else { bits_required(max_value) };
            let mutable = get_mutable(values.len() as u32, bits_required, acceptable_overhead_ratio);
            let mut i = 0;
            while i < values.len() {
                i += mutable.set_range(i, &values[i..]);
            }

            self.values.push(mutable.into_reader());
        }
    }
}
