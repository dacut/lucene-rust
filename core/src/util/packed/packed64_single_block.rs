use {
    crate::util::packed::{
        bulk_operation::{new_decoder, new_encoder},
        packed_ints::{Format, Mutable, Reader, unsigned_bits_required},
    },
    std::cmp::min,
};

pub const MAX_SUPPORTED_BITS_PER_VALUE: u32 = 32;
const SUPPORTED_BITS_PER_VALUE: [u32; 14] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 16, 21, 32];

pub fn is_supported(bits_per_value: u32) -> bool {
    return SUPPORTED_BITS_PER_VALUE.contains(&bits_per_value);
}

fn required_capacity(value_count: u32, values_per_block: u32) -> u32 {
    value_count / values_per_block + if value_count % values_per_block == 0 { 0 } else { 1 }
}

pub trait Packed64SingleBlock: Mutable {}

pub fn new_packed64_single_block(value_count: u32, bits_per_value: u32) -> Option<Box<dyn Packed64SingleBlock>> {
    match bits_per_value {
        1 => Some(Box::new(Packed64SingleBlockImpl::<1>::new(value_count))),
        2 => Some(Box::new(Packed64SingleBlockImpl::<2>::new(value_count))),
        3 => Some(Box::new(Packed64SingleBlockImpl::<3>::new(value_count))),
        4 => Some(Box::new(Packed64SingleBlockImpl::<4>::new(value_count))),
        5 => Some(Box::new(Packed64SingleBlockImpl::<5>::new(value_count))),
        6 => Some(Box::new(Packed64SingleBlockImpl::<6>::new(value_count))),
        7 => Some(Box::new(Packed64SingleBlockImpl::<7>::new(value_count))),
        8 => Some(Box::new(Packed64SingleBlockImpl::<8>::new(value_count))),
        9 => Some(Box::new(Packed64SingleBlockImpl::<9>::new(value_count))),
        10 => Some(Box::new(Packed64SingleBlockImpl::<10>::new(value_count))),
        12 => Some(Box::new(Packed64SingleBlockImpl::<12>::new(value_count))),
        16 => Some(Box::new(Packed64SingleBlockImpl::<16>::new(value_count))),
        21 => Some(Box::new(Packed64SingleBlockImpl::<21>::new(value_count))),
        32 => Some(Box::new(Packed64SingleBlockImpl::<32>::new(value_count))),
        _ => None,
    }
}

pub fn new_mutable(value_count: u32, bits_per_value: u32) -> Option<Box<dyn Mutable>> {
    match bits_per_value {
        1 => Some(Box::new(Packed64SingleBlockImpl::<1>::new(value_count))),
        2 => Some(Box::new(Packed64SingleBlockImpl::<2>::new(value_count))),
        3 => Some(Box::new(Packed64SingleBlockImpl::<3>::new(value_count))),
        4 => Some(Box::new(Packed64SingleBlockImpl::<4>::new(value_count))),
        5 => Some(Box::new(Packed64SingleBlockImpl::<5>::new(value_count))),
        6 => Some(Box::new(Packed64SingleBlockImpl::<6>::new(value_count))),
        7 => Some(Box::new(Packed64SingleBlockImpl::<7>::new(value_count))),
        8 => Some(Box::new(Packed64SingleBlockImpl::<8>::new(value_count))),
        9 => Some(Box::new(Packed64SingleBlockImpl::<9>::new(value_count))),
        10 => Some(Box::new(Packed64SingleBlockImpl::<10>::new(value_count))),
        12 => Some(Box::new(Packed64SingleBlockImpl::<12>::new(value_count))),
        16 => Some(Box::new(Packed64SingleBlockImpl::<16>::new(value_count))),
        21 => Some(Box::new(Packed64SingleBlockImpl::<21>::new(value_count))),
        32 => Some(Box::new(Packed64SingleBlockImpl::<32>::new(value_count))),
        _ => None,
    }
}

pub fn new_reader(value_count: u32, bits_per_value: u32) -> Option<Box<dyn Reader>> {
    match bits_per_value {
        1 => Some(Box::new(Packed64SingleBlockImpl::<1>::new(value_count))),
        2 => Some(Box::new(Packed64SingleBlockImpl::<2>::new(value_count))),
        3 => Some(Box::new(Packed64SingleBlockImpl::<3>::new(value_count))),
        4 => Some(Box::new(Packed64SingleBlockImpl::<4>::new(value_count))),
        5 => Some(Box::new(Packed64SingleBlockImpl::<5>::new(value_count))),
        6 => Some(Box::new(Packed64SingleBlockImpl::<6>::new(value_count))),
        7 => Some(Box::new(Packed64SingleBlockImpl::<7>::new(value_count))),
        8 => Some(Box::new(Packed64SingleBlockImpl::<8>::new(value_count))),
        9 => Some(Box::new(Packed64SingleBlockImpl::<9>::new(value_count))),
        10 => Some(Box::new(Packed64SingleBlockImpl::<10>::new(value_count))),
        12 => Some(Box::new(Packed64SingleBlockImpl::<12>::new(value_count))),
        16 => Some(Box::new(Packed64SingleBlockImpl::<16>::new(value_count))),
        21 => Some(Box::new(Packed64SingleBlockImpl::<21>::new(value_count))),
        32 => Some(Box::new(Packed64SingleBlockImpl::<32>::new(value_count))),
        _ => None,
    }
}

/// This struct is similar to [Packed64] except that it trades space for speed by ensuring that
/// a single block needs to be read/written in order to read/write a value.
#[derive(Debug)]
pub struct Packed64SingleBlockImpl<const B: u32> {
    blocks: Vec<u64>,

    // From PackedInts.MutableImpl
    value_count: u32,

    // Elided: bits_per_value
}

/// Defines the [Mutable] methods for a Packed64SingleBlock implementation.
macro_rules! packed64sb_mutable_methods {
    ($bits_per_value:expr) => {
        fn clear(&mut self) {
            self.blocks.fill(0);
        }

        fn get_bits_per_value(&self) -> u32 {
            $bits_per_value
        }

        fn set_range(&mut self, index: usize, arr: &[i64]) -> usize {
            assert!(index < self.value_count as usize);
            let mut len = min(arr.len(), self.value_count as usize - index);
            let mut off = 0;
            let original_index = index;

            // go to the next block boundary.
            let values_per_block = 64 / $bits_per_value;
            let offset_in_block = index as u32 % values_per_block;

            if offset_in_block != 0 {
                for i in offset_in_block..values_per_block {
                    if len == 0 {
                        break;
                    }

                    self.set(index, arr[off]);
                    index += 1;
                    off += 1;
                    len -= 1;
                }

                if len == 0 {
                    return index - original_index;
                }
            }

            // bulk set
            assert_eq!(index % values_per_block as usize, 0);
            #[allow(deprecated)]
            let op = new_encoder(Format::PackedSingleBlock, $bits_per_value);
            assert_eq!(op.long_block_count(), 1);
            assert_eq!(op.long_value_count(), values_per_block);
            let block_index = index as u32 / values_per_block;
            let n_blocks = (index + len) as u32 / values_per_block - block_index;

            op.encode_i64_to_u64(&arr[off..], &mut self.blocks[block_index as usize..], n_blocks).unwrap();
            let diff = n_blocks * values_per_block;
            index += diff as usize;
            len -= diff as usize;

            if index > original_index {
                // stay at the block boundary
                index - original_index
            } else {
                // no progress so far; already at a block boundary but no full block to set.
                assert_eq!(index, original_index);

                // MutableImpl.set_range impl
                for i in 0..len {
                    self.set(index + i, arr[i]);
                }

                len
            }
        }

        fn fill(&mut self, mut from_index: usize, to_index: usize, val: i64) {
            assert!(from_index <= to_index);
            assert!(unsigned_bits_required(val) <= $bits_per_value);

            let values_per_block = 64 / $bits_per_value;
            if to_index - from_index <= values_per_block << 1 {
                // there needs to be at least one full block to set for the block approach to be worth trying.
                // MutableImpl.fill impl
                for i in from_index..to_index {
                    self.set(i, val);
                }
                return
            }

            // set values naively until the next block start
            let from_offset_in_block = from_index % values_per_block;
            if from_offset_in_block != 0 {
                for i in from_offset_in_block..values_per_block {
                    self.set(from_index, val);
                    from_index += 1;
                }

                assert_eq!(from_index % values_per_block as usize, 0);
            }

            // bulk set of the inner blocks.
            let from_block = from_index / values_per_block as usize;
            let to_block = to_index / values_per_block as usize;
            assert_eq!(from_block * values_per_block as usize, from_index);

            let mut block_value = 0;
            for i in 0..values_per_block {
                block_value = block_value | (val << (i * $bits_per_value)) as u64;
            }

            self.blocks[from_block..to_block].fill(block_value);

            // fill the gap
            for i in values_per_block * to_block..to_index {
                self.set(i, val);
            }
        }

        fn into_reader(self: Box<Self>) -> Box<dyn Reader> {
            self
        }
    };
}

/// Defines the [Reader] methods for a Packed64SingleBlock implementation.
macro_rules! packed64sb_reader_methods {
    ($bits_per_value:expr) => {
        fn size(&self) -> usize {
            self.value_count as usize
        }

        fn get_range(&self, mut index: usize, arr: &mut [i64]) -> usize {
            let value_count = self.value_count;
            assert!(index < value_count as usize);
            let len = min(arr.len(), value_count as usize - index);
            let original_index = index;

            // go to the next block boundary.
            let values_per_block = 64 / $bits_per_value;
            let offset_in_block = index % values_per_block as usize;
            let mut off = 0;

            if offset_in_block != 0 {
                for i in offset_in_block..values_per_block as usize {
                    if len == 0 {
                        break;
                    }

                    arr[off] = self.get(index);
                    off += 1;
                    index += 1;
                    len -= 1;
                }

                if len == 0 {
                    return index - original_index;
                }
            }

            // bulk get
            assert_eq!(index % values_per_block as usize, 0);
            #[allow(deprecated)]
            let decoder = new_decoder(Format::PackedSingleBlock, $bits_per_value);
            assert_eq!(decoder.long_block_count(), 1);
            assert_eq!(decoder.long_value_count(), values_per_block);

            let block_index = index as u32 / values_per_block;
            let n_blocks = (index + len) as u32 / values_per_block - block_index;
            decoder.decode_u64_to_i64(&self.blocks[block_index as usize..], &mut arr[off..], n_blocks).unwrap();
            let diff = n_blocks * values_per_block;
            index += diff as usize;
            len -= diff as usize;

            if index > original_index {
                // stay at the block boundary
                index - original_index
            } else {
                // no progress so far; already at a block boundary but no full block to get.
                assert_eq!(index, original_index);

                // Reader.get_range impl
                let to_get = min(self.value_count as usize - index, arr.len());
                for i in 0..to_get {
                    arr[i] = self.get(index + i);
                }

                to_get
            }
        }
    };
}

macro_rules! packed64sb_new {
    ($bits_per_value:expr) => {
        impl Packed64SingleBlockImpl<$bits_per_value> {
            fn new(value_count: u32) -> Self {
                assert!(is_supported($bits_per_value));
                let values_per_block = 64 / $bits_per_value;
                Self {
                    blocks: vec![0; required_capacity(value_count, $bits_per_value) as usize],
                    value_count,
                }
            }
        }
    };
}

/// Defines a Packed64SingleBlock implementation using the specified shift and masking constants.
macro_rules! packed64sb_shift_mask {
    ($bits_per_value:expr, $index_shift:expr, $index_mask:expr, $bit_shift:expr, $bit_mask:expr) => {
        packed64sb_new!($bits_per_value);

        impl Reader for Packed64SingleBlockImpl<$bits_per_value> {
            packed64sb_reader_methods!($bits_per_value);

            fn get(&self, index: usize) -> i64 {
                let o = index >> $index_shift;
                let b = index & $index_mask;
                let shift = b << $bit_shift;
                (self.blocks[o] >> shift) as i64 & $bit_mask
            }
        }

        impl Mutable for Packed64SingleBlockImpl<$bits_per_value> {
            packed64sb_mutable_methods!($bits_per_value);

            fn set(&mut self, index: usize, value: i64) {
                let o = index >> $index_shift;
                let b = index & $index_mask;
                let shift = b << $bit_shift;
                self.blocks[o] = (self.blocks[o] & !($bit_mask << shift)) | ((value as u64) << shift);
            }
        }

        impl Packed64SingleBlock for Packed64SingleBlockImpl<$bits_per_value> {}
    }
}

/// Defines a Packed64SingleBlock implementation using the specified div, mul, and mask constants.
macro_rules! packed64sb_div_mul_mask {
    ($bits_per_value:expr, $index_div:expr, $bit_mul:expr, $bit_mask:expr) => {
        packed64sb_new!($bits_per_value);

        impl Reader for Packed64SingleBlockImpl<$bits_per_value> {
            packed64sb_reader_methods!($bits_per_value);

            fn get(&self, index: usize) -> i64 {
                let o = index / $index_div;
                let b = index % $index_div;
                let shift = b * $bit_mul;
                (self.blocks[o] >> shift) as i64 & $bit_mask
            }
        }

        impl Mutable for Packed64SingleBlockImpl<$bits_per_value> {
            packed64sb_mutable_methods!($bits_per_value);

            fn set(&mut self, index: usize, value: i64) {
                let o = index / $index_div;
                let b = index % $index_div;
                let shift = b * $bit_mul;
                self.blocks[o] = (self.blocks[o] & !($bit_mask << shift)) | ((value as u64) << shift);
            }
        }

        impl Packed64SingleBlock for Packed64SingleBlockImpl<$bits_per_value> {}
    }
}

packed64sb_shift_mask!(1, 6, 63, 0, 1);
packed64sb_shift_mask!(2, 5, 31, 1, 3);
packed64sb_div_mul_mask!(3, 21, 3, 7);
packed64sb_shift_mask!(4, 4, 15, 2, 15);
packed64sb_div_mul_mask!(5, 12, 3, 31);
packed64sb_div_mul_mask!(6, 10, 6, 63);
packed64sb_div_mul_mask!(7, 9, 7, 127);
packed64sb_shift_mask!(8, 3, 7, 3, 255);
packed64sb_div_mul_mask!(9, 7, 9, 511);
packed64sb_div_mul_mask!(10, 6, 10, 1023);
packed64sb_div_mul_mask!(12, 5, 12, 4095);
packed64sb_shift_mask!(16, 2, 3, 4, 65535);
packed64sb_div_mul_mask!(21, 3, 21, 2097151);
packed64sb_shift_mask!(32, 1, 1, 5, 4294967295);
