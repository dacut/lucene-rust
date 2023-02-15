use {
    crate::util::packed::{
        bulk_operation::{new_decoder, new_encoder},
        packed_ints::{unsigned_bits_required, Format, Mutable, Reader, VERSION_CURRENT},
    },
    std::cmp::min,
};

const BLOCK_SIZE: u32 = 64; // 32 = int, 64 = long
const BLOCK_BITS: u32 = 6; // The #bits representing BLOCK_SIZE
const MOD_MASK: u32 = BLOCK_SIZE - 1; // x % BLOCK_SIZE

/// Space optimized random access capable array of values with a fixed number of bits/value. Values
/// are packed contiguously.
///
/// The implementation strives to perform as fast as possible under the constraint of contiguous
/// bits, by avoiding expensive operations. This comes at the cost of code clarity.
///
/// Technical details: This implementation is a refinement of a non-branching version. The
/// non-branching get and set methods meant that 2 or 4 atomics in the underlying array were always
/// accessed, even for the cases where only 1 or 2 were needed. Even with caching, this had a
/// detrimental effect on performance. Related to this issue, the old implementation used lookup
/// tables for shifts and masks, which also proved to be a bit slower than calculating the shifts and
/// masks on the fly. See https://issues.apache.org/jira/browse/LUCENE-4062 for details.
#[derive(Debug)]
pub struct Packed64 {
    /// Values are stores contiguously in the blocks array.
    blocks: Vec<u64>,

    /// A right-aligned mask of width bits_per_value used by [Packed64::get].
    mask_right: u64,

    /// Optimization: Saves one lookup in [Packed64::get].
    bpv_minus_block_size: u32,

    // From PackedInts.MutableImpl in Java. Sizes are u32 to match Java.
    value_count: u32,
    bits_per_value: u32,
}

impl Packed64 {
    /// Creates an array with the internal structures adjusted for the given limits and initialized to 0.
    ///
    /// # Parameters
    /// * `value_count`: the number of elements.
    /// * `bits_per_value`: the number of bits available for any given value.
    pub fn new(value_count: u32, bits_per_value: u32) -> Self {
        // PackedInts.MutableImpl
        assert!(
            bits_per_value > 0 && bits_per_value <= 64,
            "bits_per_value must be > 0 and <= 64 (got {})",
            bits_per_value
        );

        // Packed64
        let format = Format::Packed;
        let long_count = format.long_count(VERSION_CURRENT, value_count, bits_per_value);
        let blocks = vec![0; long_count];
        let mask_right = !0 << (BLOCK_SIZE - bits_per_value) >> (BLOCK_SIZE - bits_per_value);
        let bpv_minus_block_size = (bits_per_value - BLOCK_SIZE) as u32;

        Self {
            blocks,
            mask_right,
            bpv_minus_block_size,
            value_count,
            bits_per_value,
        }
    }
}

impl Reader for Packed64 {
    fn get(&self, index: usize) -> i64 {
        // The abstract index in a bit stream
        let major_bit_pos = index * self.bits_per_value as usize;

        // The index in the backing long-array.
        let element_pos = (major_bit_pos >> BLOCK_BITS) as usize;

        // The number of value-bits in the second long
        let end_bits = (major_bit_pos as i64 & MOD_MASK as i64) + self.bpv_minus_block_size as i64;

        let result = if end_bits <= 0 {
            // Single block
            (self.blocks[element_pos] >> -end_bits) & self.mask_right
        } else {
            // Two blocks
            ((self.blocks[element_pos] << end_bits) | (self.blocks[element_pos + 1] >> (BLOCK_SIZE as i64 - end_bits)))
                & self.mask_right
        };

        result as i64
    }

    fn get_range(&self, index: usize, arr: &mut [i64]) -> usize {
        assert!(index <= u32::MAX as usize);
        assert!(index < self.value_count as usize);
        let mut len = min(arr.len(), self.value_count as usize - index);
        let original_index = index;
        let decoder = new_decoder(Format::Packed, self.bits_per_value);
        let long_value_count = decoder.long_value_count() as usize;

        // go to the next block where the value does not span across two blocks
        let offset_in_blocks = index % long_value_count;
        if offset_in_blocks != 0 {
            let mut i = offset_in_blocks;
            let mut off = 0;
            while i < long_value_count && len > 0 {
                arr[off] = self.get(index);
                index += 1;
                len -= 1;
            }

            if len == 0 {
                return index - original_index;
            }
        }

        // bulk get
        assert_eq!(index % long_value_count, 0);
        let block_index = (index * self.bits_per_value as usize) >> BLOCK_BITS;
        assert_eq!(block_index & MOD_MASK as usize, 0);
        let iterations = len / long_value_count;
        assert!(iterations <= u32::MAX as usize);
        let iterations = iterations as u32;

        decoder.decode_u64_to_i64(&self.blocks[block_index..], arr, iterations);
        let n_values = iterations as usize * long_value_count;
        index += n_values as usize;
        len -= n_values as usize;

        if index > original_index {
            // stay at the block boundary
            index - original_index
        } else {
            // no progress so far; already at a block boundary but no full block to get.
            assert_eq!(index, original_index);
            Reader::get_range(self, index, arr)
        }
    }

    fn size(&self) -> usize {
        self.value_count as usize
    }
}

impl Mutable for Packed64 {
    fn get_bits_per_value(&self) -> u32 {
        self.bits_per_value
    }

    fn set(&mut self, index: usize, value: i64) {
        let value = value as u64;

        // The abstract index in a contiguous bit stream
        let major_bit_pos = index * self.bits_per_value as usize;

        // The index in the backing long-array
        let element_pos = major_bit_pos >> BLOCK_BITS; // / BLOCK_SIZE

        // The number of value-bits in the second long
        let end_bits = (major_bit_pos as i64 & (MOD_MASK as i64)) + (self.bpv_minus_block_size as i64);

        if end_bits <= 0 {
            // Single block
            self.blocks[element_pos] =
                self.blocks[element_pos] & !(self.mask_right << -end_bits) | (value << -end_bits);
        } else {
            // Two blocks
            self.blocks[element_pos] = self.blocks[element_pos] & !(self.mask_right >> end_bits) | (value >> end_bits);
            self.blocks[element_pos + 1] =
                self.blocks[element_pos + 1] & (!0 >> end_bits) | (value << (BLOCK_SIZE as i64 - end_bits));
        }
    }

    fn set_range(&mut self, index: usize, arr: &[i64]) -> usize {
        assert!(index < self.value_count as usize);
        let len = min(arr.len(), self.value_count as usize - index);
        let original_index = index;
        let encoder = new_encoder(Format::Packed, self.bits_per_value);
        let long_value_count = encoder.long_value_count() as usize;

        // go to the next block where the value does not span across two blocks
        let offset_in_blocks = index % long_value_count;
        if offset_in_blocks != 0 {
            let mut i = offset_in_blocks;
            let mut off = 0;
            while i < long_value_count && len > 0 {
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
        assert_eq!(index % encoder.long_value_count() as usize, 0);
        let block_index = (index * self.bits_per_value as usize) >> BLOCK_BITS;
        let iterations = len / long_value_count;
        assert!(iterations <= u32::MAX as usize);
        let iterations = iterations as u32;
        encoder.encode_i64_to_u64(arr, &mut self.blocks[block_index..], iterations);
        let n_values = iterations * encoder.long_value_count();
        index += n_values as usize;
        len -= n_values as usize;

        if index > original_index {
            // stay at the block boundary
            index - original_index
        } else {
            // no progress so far; already at a block boundary but no full block to set.
            assert_eq!(index, original_index);
            Mutable::set_range(self, index, arr)
        }
    }

    fn fill(&mut self, from_index: usize, to_index: usize, val: i64) {
        assert!(unsigned_bits_required(val) < self.get_bits_per_value());
        assert!(from_index <= to_index);
        assert!(to_index <= u32::MAX as usize);
        let mut from_index = from_index as u32;
        let to_index = to_index as u32;

        // minimum number of values that use an exact number of full blocks
        let n_aligned_values = 64 / gcd(64, self.bits_per_value);
        let span = to_index - from_index;
        if (span as u32) < 3 * n_aligned_values {
            // there needs to be at least 2 * n_aligned_values aligned values for the block approach to be worth trying.
            Mutable::fill(self, from_index as usize, to_index as usize, val);
            return;
        }

        // fill the first values naively until the next block starts.
        let from_index_mod_n_aligned_values = from_index % n_aligned_values;
        if from_index_mod_n_aligned_values != 0 {
            for i in from_index_mod_n_aligned_values..n_aligned_values {
                self.set(from_index as usize, val);
                from_index += 1;
            }
        }

        assert!(from_index % n_aligned_values == 0);

        // compute the long[] blocks for nAlignedValues consecutive values and
        // use them to set as many values as possible without applying any mask
        // or shift
        let n_aligned_blocks = (n_aligned_values * self.bits_per_value) >> 6;

        let values = Packed64::new(n_aligned_values, self.bits_per_value);
        for i in 0..n_aligned_values {
            values.set(i as usize, val);
        }
        
        let n_aligned_values_blocks = values.blocks;
        assert!(n_aligned_blocks as usize <= n_aligned_values_blocks.len());

        let start_block = (from_index * self.bits_per_value) >> 6;
        let end_block = (to_index * self.bits_per_value) >> 6;
        for block in start_block..end_block {
            let block_value = n_aligned_values_blocks.as_slice()[(block % n_aligned_blocks) as usize];
            self.blocks.as_mut_slice()[block as usize] = block_value;
        }

        // fill the gap
        for i in ((end_block << 6) / self.bits_per_value)..to_index {
            self.set(i as usize, val);
        }
    }

    fn into_reader(self: Box<Self>) -> Box<dyn Reader> {
        self
    }
}

fn gcd(a: u32, b: u32) -> u32 {
    if a < b {
        gcd(b, a)
    } else if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}
