use {
    crate::util::packed::{
        bulk_operation::BulkOperation,
        packed_ints::{Decoder, Encoder},
    },
    std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
};

const BLOCK_COUNT: u32 = 1;

pub fn new(bits_per_value: u32) -> Option<Box<dyn BulkOperation>> {
    match bits_per_value {
        1 => Some(Box::new(BulkOperationPackedSingleBlock::<1>::new())),
        2 => Some(Box::new(BulkOperationPackedSingleBlock::<2>::new())),
        3 => Some(Box::new(BulkOperationPackedSingleBlock::<3>::new())),
        4 => Some(Box::new(BulkOperationPackedSingleBlock::<4>::new())),
        5 => Some(Box::new(BulkOperationPackedSingleBlock::<5>::new())),
        6 => Some(Box::new(BulkOperationPackedSingleBlock::<6>::new())),
        7 => Some(Box::new(BulkOperationPackedSingleBlock::<7>::new())),
        8 => Some(Box::new(BulkOperationPackedSingleBlock::<8>::new())),
        9 => Some(Box::new(BulkOperationPackedSingleBlock::<9>::new())),
        10 => Some(Box::new(BulkOperationPackedSingleBlock::<10>::new())),
        12 => Some(Box::new(BulkOperationPackedSingleBlock::<12>::new())),
        16 => Some(Box::new(BulkOperationPackedSingleBlock::<16>::new())),
        21 => Some(Box::new(BulkOperationPackedSingleBlock::<21>::new())),
        32 => Some(Box::new(BulkOperationPackedSingleBlock::<32>::new())),
        _ => None,
    }
}

pub fn new_decoder(bits_per_value: u32) -> Option<Box<dyn Decoder>> {
    match bits_per_value {
        1 => Some(Box::new(BulkOperationPackedSingleBlock::<1>::new())),
        2 => Some(Box::new(BulkOperationPackedSingleBlock::<2>::new())),
        3 => Some(Box::new(BulkOperationPackedSingleBlock::<3>::new())),
        4 => Some(Box::new(BulkOperationPackedSingleBlock::<4>::new())),
        5 => Some(Box::new(BulkOperationPackedSingleBlock::<5>::new())),
        6 => Some(Box::new(BulkOperationPackedSingleBlock::<6>::new())),
        7 => Some(Box::new(BulkOperationPackedSingleBlock::<7>::new())),
        8 => Some(Box::new(BulkOperationPackedSingleBlock::<8>::new())),
        9 => Some(Box::new(BulkOperationPackedSingleBlock::<9>::new())),
        10 => Some(Box::new(BulkOperationPackedSingleBlock::<10>::new())),
        12 => Some(Box::new(BulkOperationPackedSingleBlock::<12>::new())),
        16 => Some(Box::new(BulkOperationPackedSingleBlock::<16>::new())),
        21 => Some(Box::new(BulkOperationPackedSingleBlock::<21>::new())),
        32 => Some(Box::new(BulkOperationPackedSingleBlock::<32>::new())),
        _ => None,
    }
}

pub fn new_encoder(bits_per_value: u32) -> Option<Box<dyn Encoder>> {
    match bits_per_value {
        1 => Some(Box::new(BulkOperationPackedSingleBlock::<1>::new())),
        2 => Some(Box::new(BulkOperationPackedSingleBlock::<2>::new())),
        3 => Some(Box::new(BulkOperationPackedSingleBlock::<3>::new())),
        4 => Some(Box::new(BulkOperationPackedSingleBlock::<4>::new())),
        5 => Some(Box::new(BulkOperationPackedSingleBlock::<5>::new())),
        6 => Some(Box::new(BulkOperationPackedSingleBlock::<6>::new())),
        7 => Some(Box::new(BulkOperationPackedSingleBlock::<7>::new())),
        8 => Some(Box::new(BulkOperationPackedSingleBlock::<8>::new())),
        9 => Some(Box::new(BulkOperationPackedSingleBlock::<9>::new())),
        10 => Some(Box::new(BulkOperationPackedSingleBlock::<10>::new())),
        12 => Some(Box::new(BulkOperationPackedSingleBlock::<12>::new())),
        16 => Some(Box::new(BulkOperationPackedSingleBlock::<16>::new())),
        21 => Some(Box::new(BulkOperationPackedSingleBlock::<21>::new())),
        32 => Some(Box::new(BulkOperationPackedSingleBlock::<32>::new())),
        _ => None,
    }
}

/// Non-specialized [BulkOperation] for [Format::PackedSingleBlock]
#[derive(Debug)]
pub struct BulkOperationPackedSingleBlock<const B: u32> {
    value_count: u32,
    mask: u64,
}

impl<const B: u32> BulkOperationPackedSingleBlock<B> {
    pub fn new() -> Self {
        Self {
            value_count: 64 / B,
            mask: (1 << B) - 1,
        }
    }

    fn single_decode_i64(&self, block: u64, values: &mut [i64], mut values_offset: usize) -> usize{
        values[values_offset] = (block & self.mask) as i64;
        values_offset += 1;
        for j in 1..self.value_count {
            block >>= B;
            values[values_offset] = (block & self.mask) as i64;
            values_offset += 1;
        }

        values_offset
    }
    
    fn single_decode_i32(&self, block: u64, values: &mut [i32], mut values_offset: usize) -> usize {
        values[values_offset] = (block & self.mask) as i32;
        values_offset += 1;
        for j in 1..self.value_count {
            block >>= B;
            values[values_offset] = (block & self.mask) as i32;
            values_offset += 1;
        }

        values_offset
    }

    fn single_encode_i32(&self, values: &[i32], values_offset: usize) -> u64 {
        let mut block = values[values_offset] as u64;
        values_offset += 1;
        for j in 1..self.value_count {
            block |= (values[values_offset] as u64) << (j * B);
            values_offset += 1;
        }
        block
    }

    fn single_encode_i64(&self, values: &[i64], values_offset: usize) -> u64 {
        let mut block = values[values_offset] as u64;
        values_offset += 1;
        for j in 1..self.value_count {
            block |= (values[values_offset] as u64) << (j * B);
            values_offset += 1;
        }
        block
    }
}

fn read_long(blocks: &[u8], blocks_offset: &mut usize) -> u64 {
    (blocks[*blocks_offset] as u64) << 56 |
    (blocks[*blocks_offset + 1] as u64) << 48 |
    (blocks[*blocks_offset + 2] as u64) << 40 |
    (blocks[*blocks_offset + 3] as u64) << 32 |
    (blocks[*blocks_offset + 4] as u64) << 24 |
    (blocks[*blocks_offset + 5] as u64) << 16 |
    (blocks[*blocks_offset + 6] as u64) << 8 |
    blocks[*blocks_offset + 7] as u64
}

fn write_long(block: u64, blocks: &mut [u8]) -> usize {
    let mut j = 1;
    for i in 0..8 {
        blocks[i] = (block >> (64 - (j << 3))) as u8;
        j += 1;
    }
    8
}

impl<const B: u32> Decoder for BulkOperationPackedSingleBlock<B> {
    fn long_block_count(&self) -> u32 {
        BLOCK_COUNT
    }

    fn byte_block_count(&self) -> u32 {
        BLOCK_COUNT * 8
    }

    fn long_value_count(&self) -> u32 {
        self.value_count
    }

    fn byte_value_count(&self) -> u32 {
        self.value_count
    }

    fn decode_u64_to_i64(&mut self, blocks: &[u64], values: &mut [i64], iterations: u32) -> IoResult<()> {
        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for _ in 0..iterations {
            let block = blocks[blocks_offset];
            blocks_offset += 1;
            values_offset = self.single_decode_i64(block, values, values_offset);
        }

        Ok(())
    }

    fn decode_u8_to_i64(&mut self, blocks: &[u8], values: &mut [i64], iterations: u32) -> IoResult<()> {
        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for _ in 0..iterations {
            let block = read_long(blocks, &mut blocks_offset);
            blocks_offset += 8;
            values_offset = self.single_decode_i64(block, values, values_offset);
        }

        Ok(())
    }

    fn decode_u64_to_i32(&mut self, blocks: &[u64], values: &mut [i32], iterations: u32) -> IoResult<()> {
        if B > 32 {
            return Err(IoError::new(IoErrorKind::InvalidData, "Cannot decode more than 32-bits into i32"));
        }

        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for _ in 0..iterations {
            let block = blocks[blocks_offset];
            blocks_offset += 1;
            values_offset = self.single_decode_i32(block, values, values_offset);
        }

        Ok(())
    }

    fn decode_u8_to_i32(&mut self, blocks: &[u8], values: &mut [i32], iterations: u32) -> IoResult<()> {
        if B > 32 {
            return Err(IoError::new(IoErrorKind::InvalidData, "Cannot decode more than 32-bits into i32"));
        }

        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for _ in 0..iterations {
            let block = read_long(blocks, &mut blocks_offset);
            blocks_offset += 8;
            values_offset = self.single_decode_i32(block, values, values_offset);
        }

        Ok(())
    }
}

impl<const B: u32> Encoder for BulkOperationPackedSingleBlock<B> {
    fn long_block_count(&self) -> u32 {
        BLOCK_COUNT
    }

    fn byte_block_count(&self) -> u32 {
        BLOCK_COUNT * 8
    }

    fn long_value_count(&self) -> u32 {
        self.value_count
    }

    fn byte_value_count(&self) -> u32 {
        self.value_count
    }

    fn encode_i64_to_u64(&mut self, values: &[i64], blocks: &mut [u64], iterations: u32) -> IoResult<()> {
        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for _ in 0..iterations {
            blocks[blocks_offset] = self.single_encode_i64(values, values_offset);
            blocks_offset += 1;
            values_offset += self.value_count as usize;
        }

        Ok(())
    }

    fn encode_i32_to_u64(&mut self, values: &[i32], blocks: &mut [u64], iterations: u32) -> IoResult<()> {
        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for _ in 0..iterations {
            blocks[blocks_offset] = self.single_encode_i32(values, values_offset);
            blocks_offset += 1;
            values_offset += self.value_count as usize;
        }

        Ok(())
    }

    fn encode_i64_to_u8(&mut self, values: &[i64], blocks: &mut [u8], iterations: u32) -> IoResult<()> {
        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for _ in 0..iterations {
            let block = self.single_encode_i64(values, values_offset);
            values_offset += self.value_count as usize;
            write_long(block, &mut blocks[blocks_offset..]);
            blocks_offset += 8;
        }

        Ok(())
    }

    fn encode_i32_to_u8(&mut self, values: &[i32], blocks: &mut [u8], iterations: u32) -> IoResult<()> {
        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for _ in 0..iterations {
            let block = self.single_encode_i32(values, values_offset);
            values_offset += self.value_count as usize;
            write_long(block, &mut blocks[blocks_offset..]);
            blocks_offset += 8;
        }

        Ok(())
    }
}

impl<const B: u32> BulkOperation for BulkOperationPackedSingleBlock<B> {
    fn long_block_count(&self) -> u32 {
        BLOCK_COUNT
    }

    fn byte_block_count(&self) -> u32 {
        BLOCK_COUNT * 8
    }

    fn long_value_count(&self) -> u32 {
        self.value_count
    }

    fn byte_value_count(&self) -> u32 {
        self.value_count
    }
}