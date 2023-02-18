use {
    crate::util::{
        long_values::LongValues,
        packed::{
            packed_ints::Reader,
            packed_long_values::{self, BasicPackedLongValues, Iter, PackedLongValues},
        },
    },
    std::{cmp::min, io::Result as IoResult, ops::Index},
};

#[derive(Debug)]
pub struct DeltaPackedLongValues {
    pub(crate) bplv: BasicPackedLongValues,
    pub(crate) mins: Vec<i64>,
}

impl DeltaPackedLongValues {
    pub(crate) fn new(
        page_shift: u32,
        page_mask: u32,
        values: Vec<Box<dyn Reader>>,
        mins: Vec<i64>,
        size: usize,
    ) -> Self {
        Self {
            bplv: BasicPackedLongValues::new(page_shift, page_mask, values, size),
            mins,
        }
    }
}

impl PackedLongValues for DeltaPackedLongValues {
    type Iter<'a> = Iter<'a, Self>;

    #[inline]
    fn size(&self) -> usize {
        self.bplv.size()
    }

    fn get(&self, block: u32, element: usize) -> i64 {
        self.mins.as_slice()[block as usize] + self.bplv.get(block, element) + self.mins[block as usize]
    }

    fn decode_block(&self, block: u32, dest: &mut [i64]) -> usize {
        let count = self.bplv.decode_block(block, dest);
        let min = self.mins.as_slice()[block as usize];
        for i in 0..count {
            dest[i] += min;
        }

        count
    }

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        Iter::new(self)
    }

    #[inline]
    fn values_slice(&self) -> &[Box<dyn Reader>] {
        self.bplv.values_slice()
    }

    #[inline]
    fn get_page_shift(&self) -> u32 {
        self.bplv.get_page_shift()
    }

    #[inline]
    fn get_page_mask(&self) -> u32 {
        self.bplv.get_page_mask()
    }
}

impl LongValues for DeltaPackedLongValues {}

impl Index<usize> for DeltaPackedLongValues {
    type Output = i64;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.size());
        let block = index >> self.bplv.page_shift;
        let element = index & self.bplv.page_mask as usize;
        &PackedLongValues::get(self, block as u32, element)
    }
}

#[derive(Debug)]
pub struct Builder {
    pub(crate) plvb: packed_long_values::Builder,
    pub(crate) mins: Vec<i64>,
}


impl Builder {
    pub fn new(page_size: u32, acceptable_overhead_ratio: f32) -> IoResult<Self> {
        let plvb = packed_long_values::Builder::new(page_size, acceptable_overhead_ratio)?;
        let mins = Vec::with_capacity(plvb.values.len());
        Ok(Self {
            plvb,
            mins,
        })
    }

    pub fn build(self) -> DeltaPackedLongValues {
        self.finish();
        DeltaPackedLongValues::new(self.plvb.page_shift, self.plvb.page_mask, self.plvb.values, self.mins, self.plvb.size)
    }

    /// Add a new element to this builder.
    #[inline]
    pub fn add(&mut self, l: i64) {
        self.plvb.add(l)
    }

    pub(crate) fn finish(&mut self) {
        self.pack_one();
    }

    pub(crate) fn pack_one(&mut self) {
        self.pack(&self.plvb.pending, self.plvb.acceptable_overhead_ratio);
        // Reset pending buffer.
        self.plvb.pending.clear();
    }

    pub(crate) fn pack(&mut self, values: &[i64], acceptable_overhead_ratio: f32) {
        assert!(!values.is_empty());
        let min_value = values[0];

        for value in values.iter().skip(1) {
            min_value = min(min_value, *value);
        }

        let min_erased = Vec::with_capacity(values.len());
        for value in values.iter() {
            min_erased.push(value - min_value);
        }

        self.plvb.pack(&min_erased, acceptable_overhead_ratio);
        self.mins.push(min_value);
    }
}