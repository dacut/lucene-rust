use {
    crate::util::packed::{
        packed_ints::{Decoder, Encoder, Format},
        bulk_operation_packed,
        bulk_operation_packed_single_block,
    },
    std::fmt::Debug,
};


pub trait BulkOperation : Debug + Decoder + Encoder {
    /// The minimum number of long blocks to encode in a single iteration, when using long encoding.
    fn long_block_count(&self) -> u32;

    /// The number of values that can be stored in [Decoder::long_block_count] long blocks.
    fn long_value_count(&self) -> u32;

    /// The minimum number of byte blocks to encode in a single iteration, when using byte encoding.
    fn byte_block_count(&self) -> u32;

    /// The number of values that can be stored in [Decoder::byte_block_count] byte blocks.
    fn byte_value_count(&self) -> u32;
    
    /// For every number of bits per value, there is a minimum number of blocks (b) / values (v) you
    /// need to write in order to reach the next block boundary:
    /// 
    /// * 16 bits per value -> b=2, v=1
    /// * 24 bits per value -> b=3, v=1
    /// * 50 bits per value -> b=25, v=4
    /// * 63 bits per value -> b=63, v=8
    /// * ...
    /// 
    /// A bulk read consists in copying `iterations` * `v` values that are contained in 
    /// `iterations` * `b` blocks into a `&mut [u64]` (higher values of `iterations`
    /// are likely to yield a better throughput): this requires n * (b + 8v) bytes of memory.
    /// 
    /// This method computes `iterations` as `ramBudget / (b + 8v)` (since an i64 is 8 bytes).
    fn compute_iterations(&self, value_count: u32, ram_budget: usize) -> u32 {
        let byte_value_count = <Self as Decoder>::byte_value_count(self);
        let iterations = (ram_budget / (<Self as Decoder>::byte_block_count(self) as usize + 8 * byte_value_count as usize)) as u32;

        if iterations == 0 {
            // at least 1
            1
        } else if (iterations - 1) * byte_value_count >= value_count {
            // don't allocate for more than the size of the reader
            let iterations = value_count / byte_value_count;
            if iterations % byte_value_count == 0 {
                iterations
            } else {
                iterations + 1
            }        
        } else {
            iterations
        }
    }
}

pub fn new_bulk_operation(format: Format, bits_per_value: u32) -> Box<dyn BulkOperation> {
    match format {
        Format::Packed => match bulk_operation_packed::new(bits_per_value) {
            Some(bulk_operation) => bulk_operation,
            None => panic!("Unsupported bits_per_value: {bits_per_value}"),
        }
        #[allow(deprecated)]
        Format::PackedSingleBlock => match bulk_operation_packed_single_block::new(bits_per_value) {
            Some(bulk_operation) => bulk_operation,
            None => panic!("Unsupported bits_per_value: {bits_per_value}"),
        }
    }
}

pub fn new_decoder(format: Format, bits_per_value: u32) -> Box<dyn Decoder> {
    match format {
        Format::Packed => match bulk_operation_packed::new_decoder(bits_per_value) {
            Some(decoder)=> decoder,
            None => panic!("Unsupported bits_per_value: {bits_per_value}"),
        }
        #[allow(deprecated)]
        Format::PackedSingleBlock => match bulk_operation_packed_single_block::new_decoder(bits_per_value) {
            Some(decoder) => decoder,
            None => panic!("Unsupported bits_per_value: {bits_per_value}"),
        }
    }
}

pub fn new_encoder(format: Format, bits_per_value: u32) -> Box<dyn Encoder> {
    match format {
        Format::Packed => match bulk_operation_packed::new_encoder(bits_per_value) {
            Some(encoder)=> encoder,
            None => panic!("Unsupported bits_per_value: {bits_per_value}"),
        }
        #[allow(deprecated)]
        Format::PackedSingleBlock => match bulk_operation_packed_single_block::new_encoder(bits_per_value) {
            Some(encoder) => encoder,
            None => panic!("Unsupported bits_per_value: {bits_per_value}"),
        }
    }
}
