use {
    crate::util::{
        long_values::LongValues,
        packed::{
            delta_packed_long_values::{self, DeltaPackedLongValues},
            monotonic_block_packed_reader::expected,
            packed_ints::Reader,
            packed_long_values::{Iter, PackedLongValues},
        },
    },
    std::{io::Result as IoResult, ops::Index},
};

#[derive(Debug)]
pub struct MonotonicLongValues {
    dplv: DeltaPackedLongValues,
    averages: Vec<f32>,
}

impl MonotonicLongValues {
    pub(crate) fn new(
        page_shift: u32,
        page_mask: u32,
        values: Vec<Box<dyn Reader>>,
        mins: Vec<i64>,
        averages: Vec<f32>,
        size: usize,
    ) -> Self {
        Self {
            dplv: DeltaPackedLongValues::new(page_shift, page_mask, values, mins, size),
            averages,
        }
    }
}

impl PackedLongValues for MonotonicLongValues {
    type Iter<'a> = Iter<'a, Self> where Self: 'a;

    #[inline]
    fn size(&self) -> usize {
        self.dplv.bplv.size()
    }

    fn get(&self, block: u32, element: usize) -> i64 {
        expected(self.dplv.mins[block as usize], self.averages[block as usize], element)
            + self.dplv.bplv.values[block as usize].get(element)
    }

    fn decode_block(&self, block: u32, dest: &mut [i64]) -> usize {
        let count = self.dplv.decode_block(block, dest);
        let average = self.averages[block as usize];
        for i in 0..count {
            dest[i] += expected(0, average, i);
        }

        count
    }

    fn iter<'a>(&'a self) -> Iter<'a, Self> {
        Iter::new(self)
    }

    #[inline]
    fn values_slice(&self) -> &[Box<dyn Reader>] {
        self.dplv.values_slice()
    }

    #[inline]
    fn get_page_shift(&self) -> u32 {
        self.dplv.get_page_shift()
    }

    #[inline]
    fn get_page_mask(&self) -> u32 {
        self.dplv.get_page_mask()
    }
}

impl LongValues for MonotonicLongValues {}

impl Index<usize> for MonotonicLongValues {
    type Output = i64;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.size());
        let block = index >> self.dplv.bplv.page_shift;
        let element = index & self.dplv.bplv.page_mask as usize;
        &PackedLongValues::get(self, block as u32, element)
    }
}

#[derive(Debug)]
pub struct Builder {
    averages: Vec<f32>,
    dplvb: delta_packed_long_values::Builder,
}

impl Builder {
    pub fn new(page_size: u32, acceptable_overhead_ratio: f32) -> IoResult<Self> {
        let dplvb = delta_packed_long_values::Builder::new(page_size, acceptable_overhead_ratio)?;
        let averages = Vec::with_capacity(dplvb.plvb.values.len());

        Ok(Self {
            averages,
            dplvb,
        })
    }

    pub fn build(self) -> MonotonicLongValues {
        self.finish();
        MonotonicLongValues::new(
            self.dplvb.plvb.page_shift,
            self.dplvb.plvb.page_mask,
            self.dplvb.plvb.values,
            self.dplvb.mins,
            self.averages,
            self.dplvb.plvb.size,
        )
    }

    /// Add a new element to this builder.
    #[inline]
    pub fn add(&mut self, l: i64) {
        self.dplvb.add(l)
    }
    
    pub(crate) fn finish(&mut self) {
        self.pack_one();
    }

    fn pack_one(&mut self) {
        self.pack(&self.dplvb.plvb.pending, self.dplvb.plvb.acceptable_overhead_ratio);
        // Reset pending buffer.
        self.dplvb.plvb.pending.clear();
    }

    fn pack(&mut self, values: &[i64], acceptable_overhead_ratio: f32) {
        assert!(!values.is_empty());
        let average = if values.len() == 1 {
            0.0
        } else {
            (values[values.len() - 1] - values[0]) as f32 / (values.len() - 1) as f32
        };

        let mut averaged_values = Vec::with_capacity(values.len());
        for i in 0..values.len() {
            averaged_values.push(values[i] - expected(0, average, i));
        }

        self.dplvb.pack(&averaged_values, acceptable_overhead_ratio);
        self.averages.push(average);
    }
}
