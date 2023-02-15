use {
    crate::util::packed::{
        bulk_operation::BulkOperation,
        packed_ints::{bits_required, unsigned_bits_required, Decoder, Encoder},
    },
    std::io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
};

pub fn new(bits_per_value: u32) -> Option<Box<dyn BulkOperation>> {
    match bits_per_value {
        1 => Some(Box::new(BulkOperationPacked::<1>::new())),
        2 => Some(Box::new(BulkOperationPacked::<2>::new())),
        3 => Some(Box::new(BulkOperationPacked::<3>::new())),
        4 => Some(Box::new(BulkOperationPacked::<4>::new())),
        5 => Some(Box::new(BulkOperationPacked::<5>::new())),
        6 => Some(Box::new(BulkOperationPacked::<6>::new())),
        7 => Some(Box::new(BulkOperationPacked::<7>::new())),
        8 => Some(Box::new(BulkOperationPacked::<8>::new())),
        9 => Some(Box::new(BulkOperationPacked::<9>::new())),
        10 => Some(Box::new(BulkOperationPacked::<10>::new())),
        11 => Some(Box::new(BulkOperationPacked::<11>::new())),
        12 => Some(Box::new(BulkOperationPacked::<12>::new())),
        13 => Some(Box::new(BulkOperationPacked::<13>::new())),
        14 => Some(Box::new(BulkOperationPacked::<14>::new())),
        15 => Some(Box::new(BulkOperationPacked::<15>::new())),
        16 => Some(Box::new(BulkOperationPacked::<16>::new())),
        17 => Some(Box::new(BulkOperationPacked::<17>::new())),
        18 => Some(Box::new(BulkOperationPacked::<18>::new())),
        19 => Some(Box::new(BulkOperationPacked::<19>::new())),
        20 => Some(Box::new(BulkOperationPacked::<20>::new())),
        21 => Some(Box::new(BulkOperationPacked::<21>::new())),
        22 => Some(Box::new(BulkOperationPacked::<22>::new())),
        23 => Some(Box::new(BulkOperationPacked::<23>::new())),
        24 => Some(Box::new(BulkOperationPacked::<24>::new())),
        25 => Some(Box::new(BulkOperationPacked::<25>::new())),
        26 => Some(Box::new(BulkOperationPacked::<26>::new())),
        27 => Some(Box::new(BulkOperationPacked::<27>::new())),
        28 => Some(Box::new(BulkOperationPacked::<28>::new())),
        29 => Some(Box::new(BulkOperationPacked::<29>::new())),
        30 => Some(Box::new(BulkOperationPacked::<30>::new())),
        31 => Some(Box::new(BulkOperationPacked::<31>::new())),
        32 => Some(Box::new(BulkOperationPacked::<32>::new())),
        33 => Some(Box::new(BulkOperationPacked::<33>::new())),
        34 => Some(Box::new(BulkOperationPacked::<34>::new())),
        35 => Some(Box::new(BulkOperationPacked::<35>::new())),
        36 => Some(Box::new(BulkOperationPacked::<36>::new())),
        37 => Some(Box::new(BulkOperationPacked::<37>::new())),
        38 => Some(Box::new(BulkOperationPacked::<38>::new())),
        39 => Some(Box::new(BulkOperationPacked::<39>::new())),
        40 => Some(Box::new(BulkOperationPacked::<40>::new())),
        41 => Some(Box::new(BulkOperationPacked::<41>::new())),
        42 => Some(Box::new(BulkOperationPacked::<42>::new())),
        43 => Some(Box::new(BulkOperationPacked::<43>::new())),
        44 => Some(Box::new(BulkOperationPacked::<44>::new())),
        45 => Some(Box::new(BulkOperationPacked::<45>::new())),
        46 => Some(Box::new(BulkOperationPacked::<46>::new())),
        47 => Some(Box::new(BulkOperationPacked::<47>::new())),
        48 => Some(Box::new(BulkOperationPacked::<48>::new())),
        49 => Some(Box::new(BulkOperationPacked::<49>::new())),
        50 => Some(Box::new(BulkOperationPacked::<50>::new())),
        51 => Some(Box::new(BulkOperationPacked::<51>::new())),
        52 => Some(Box::new(BulkOperationPacked::<52>::new())),
        53 => Some(Box::new(BulkOperationPacked::<53>::new())),
        54 => Some(Box::new(BulkOperationPacked::<54>::new())),
        55 => Some(Box::new(BulkOperationPacked::<55>::new())),
        56 => Some(Box::new(BulkOperationPacked::<56>::new())),
        57 => Some(Box::new(BulkOperationPacked::<57>::new())),
        58 => Some(Box::new(BulkOperationPacked::<58>::new())),
        59 => Some(Box::new(BulkOperationPacked::<59>::new())),
        60 => Some(Box::new(BulkOperationPacked::<60>::new())),
        61 => Some(Box::new(BulkOperationPacked::<61>::new())),
        62 => Some(Box::new(BulkOperationPacked::<62>::new())),
        63 => Some(Box::new(BulkOperationPacked::<63>::new())),
        64 => Some(Box::new(BulkOperationPacked::<64>::new())),
        _ => None,
    }
}

pub fn new_decoder(bits_per_value: u32) -> Option<Box<dyn Decoder>> {
    match bits_per_value {
        1 => Some(Box::new(BulkOperationPacked::<1>::new())),
        2 => Some(Box::new(BulkOperationPacked::<2>::new())),
        3 => Some(Box::new(BulkOperationPacked::<3>::new())),
        4 => Some(Box::new(BulkOperationPacked::<4>::new())),
        5 => Some(Box::new(BulkOperationPacked::<5>::new())),
        6 => Some(Box::new(BulkOperationPacked::<6>::new())),
        7 => Some(Box::new(BulkOperationPacked::<7>::new())),
        8 => Some(Box::new(BulkOperationPacked::<8>::new())),
        9 => Some(Box::new(BulkOperationPacked::<9>::new())),
        10 => Some(Box::new(BulkOperationPacked::<10>::new())),
        11 => Some(Box::new(BulkOperationPacked::<11>::new())),
        12 => Some(Box::new(BulkOperationPacked::<12>::new())),
        13 => Some(Box::new(BulkOperationPacked::<13>::new())),
        14 => Some(Box::new(BulkOperationPacked::<14>::new())),
        15 => Some(Box::new(BulkOperationPacked::<15>::new())),
        16 => Some(Box::new(BulkOperationPacked::<16>::new())),
        17 => Some(Box::new(BulkOperationPacked::<17>::new())),
        18 => Some(Box::new(BulkOperationPacked::<18>::new())),
        19 => Some(Box::new(BulkOperationPacked::<19>::new())),
        20 => Some(Box::new(BulkOperationPacked::<20>::new())),
        21 => Some(Box::new(BulkOperationPacked::<21>::new())),
        22 => Some(Box::new(BulkOperationPacked::<22>::new())),
        23 => Some(Box::new(BulkOperationPacked::<23>::new())),
        24 => Some(Box::new(BulkOperationPacked::<24>::new())),
        25 => Some(Box::new(BulkOperationPacked::<25>::new())),
        26 => Some(Box::new(BulkOperationPacked::<26>::new())),
        27 => Some(Box::new(BulkOperationPacked::<27>::new())),
        28 => Some(Box::new(BulkOperationPacked::<28>::new())),
        29 => Some(Box::new(BulkOperationPacked::<29>::new())),
        30 => Some(Box::new(BulkOperationPacked::<30>::new())),
        31 => Some(Box::new(BulkOperationPacked::<31>::new())),
        32 => Some(Box::new(BulkOperationPacked::<32>::new())),
        33 => Some(Box::new(BulkOperationPacked::<33>::new())),
        34 => Some(Box::new(BulkOperationPacked::<34>::new())),
        35 => Some(Box::new(BulkOperationPacked::<35>::new())),
        36 => Some(Box::new(BulkOperationPacked::<36>::new())),
        37 => Some(Box::new(BulkOperationPacked::<37>::new())),
        38 => Some(Box::new(BulkOperationPacked::<38>::new())),
        39 => Some(Box::new(BulkOperationPacked::<39>::new())),
        40 => Some(Box::new(BulkOperationPacked::<40>::new())),
        41 => Some(Box::new(BulkOperationPacked::<41>::new())),
        42 => Some(Box::new(BulkOperationPacked::<42>::new())),
        43 => Some(Box::new(BulkOperationPacked::<43>::new())),
        44 => Some(Box::new(BulkOperationPacked::<44>::new())),
        45 => Some(Box::new(BulkOperationPacked::<45>::new())),
        46 => Some(Box::new(BulkOperationPacked::<46>::new())),
        47 => Some(Box::new(BulkOperationPacked::<47>::new())),
        48 => Some(Box::new(BulkOperationPacked::<48>::new())),
        49 => Some(Box::new(BulkOperationPacked::<49>::new())),
        50 => Some(Box::new(BulkOperationPacked::<50>::new())),
        51 => Some(Box::new(BulkOperationPacked::<51>::new())),
        52 => Some(Box::new(BulkOperationPacked::<52>::new())),
        53 => Some(Box::new(BulkOperationPacked::<53>::new())),
        54 => Some(Box::new(BulkOperationPacked::<54>::new())),
        55 => Some(Box::new(BulkOperationPacked::<55>::new())),
        56 => Some(Box::new(BulkOperationPacked::<56>::new())),
        57 => Some(Box::new(BulkOperationPacked::<57>::new())),
        58 => Some(Box::new(BulkOperationPacked::<58>::new())),
        59 => Some(Box::new(BulkOperationPacked::<59>::new())),
        60 => Some(Box::new(BulkOperationPacked::<60>::new())),
        61 => Some(Box::new(BulkOperationPacked::<61>::new())),
        62 => Some(Box::new(BulkOperationPacked::<62>::new())),
        63 => Some(Box::new(BulkOperationPacked::<63>::new())),
        64 => Some(Box::new(BulkOperationPacked::<64>::new())),
        _ => None,
    }
}

pub fn new_encoder(bits_per_value: u32) -> Option<Box<dyn Encoder>> {
    match bits_per_value {
        1 => Some(Box::new(BulkOperationPacked::<1>::new())),
        2 => Some(Box::new(BulkOperationPacked::<2>::new())),
        3 => Some(Box::new(BulkOperationPacked::<3>::new())),
        4 => Some(Box::new(BulkOperationPacked::<4>::new())),
        5 => Some(Box::new(BulkOperationPacked::<5>::new())),
        6 => Some(Box::new(BulkOperationPacked::<6>::new())),
        7 => Some(Box::new(BulkOperationPacked::<7>::new())),
        8 => Some(Box::new(BulkOperationPacked::<8>::new())),
        9 => Some(Box::new(BulkOperationPacked::<9>::new())),
        10 => Some(Box::new(BulkOperationPacked::<10>::new())),
        11 => Some(Box::new(BulkOperationPacked::<11>::new())),
        12 => Some(Box::new(BulkOperationPacked::<12>::new())),
        13 => Some(Box::new(BulkOperationPacked::<13>::new())),
        14 => Some(Box::new(BulkOperationPacked::<14>::new())),
        15 => Some(Box::new(BulkOperationPacked::<15>::new())),
        16 => Some(Box::new(BulkOperationPacked::<16>::new())),
        17 => Some(Box::new(BulkOperationPacked::<17>::new())),
        18 => Some(Box::new(BulkOperationPacked::<18>::new())),
        19 => Some(Box::new(BulkOperationPacked::<19>::new())),
        20 => Some(Box::new(BulkOperationPacked::<20>::new())),
        21 => Some(Box::new(BulkOperationPacked::<21>::new())),
        22 => Some(Box::new(BulkOperationPacked::<22>::new())),
        23 => Some(Box::new(BulkOperationPacked::<23>::new())),
        24 => Some(Box::new(BulkOperationPacked::<24>::new())),
        25 => Some(Box::new(BulkOperationPacked::<25>::new())),
        26 => Some(Box::new(BulkOperationPacked::<26>::new())),
        27 => Some(Box::new(BulkOperationPacked::<27>::new())),
        28 => Some(Box::new(BulkOperationPacked::<28>::new())),
        29 => Some(Box::new(BulkOperationPacked::<29>::new())),
        30 => Some(Box::new(BulkOperationPacked::<30>::new())),
        31 => Some(Box::new(BulkOperationPacked::<31>::new())),
        32 => Some(Box::new(BulkOperationPacked::<32>::new())),
        33 => Some(Box::new(BulkOperationPacked::<33>::new())),
        34 => Some(Box::new(BulkOperationPacked::<34>::new())),
        35 => Some(Box::new(BulkOperationPacked::<35>::new())),
        36 => Some(Box::new(BulkOperationPacked::<36>::new())),
        37 => Some(Box::new(BulkOperationPacked::<37>::new())),
        38 => Some(Box::new(BulkOperationPacked::<38>::new())),
        39 => Some(Box::new(BulkOperationPacked::<39>::new())),
        40 => Some(Box::new(BulkOperationPacked::<40>::new())),
        41 => Some(Box::new(BulkOperationPacked::<41>::new())),
        42 => Some(Box::new(BulkOperationPacked::<42>::new())),
        43 => Some(Box::new(BulkOperationPacked::<43>::new())),
        44 => Some(Box::new(BulkOperationPacked::<44>::new())),
        45 => Some(Box::new(BulkOperationPacked::<45>::new())),
        46 => Some(Box::new(BulkOperationPacked::<46>::new())),
        47 => Some(Box::new(BulkOperationPacked::<47>::new())),
        48 => Some(Box::new(BulkOperationPacked::<48>::new())),
        49 => Some(Box::new(BulkOperationPacked::<49>::new())),
        50 => Some(Box::new(BulkOperationPacked::<50>::new())),
        51 => Some(Box::new(BulkOperationPacked::<51>::new())),
        52 => Some(Box::new(BulkOperationPacked::<52>::new())),
        53 => Some(Box::new(BulkOperationPacked::<53>::new())),
        54 => Some(Box::new(BulkOperationPacked::<54>::new())),
        55 => Some(Box::new(BulkOperationPacked::<55>::new())),
        56 => Some(Box::new(BulkOperationPacked::<56>::new())),
        57 => Some(Box::new(BulkOperationPacked::<57>::new())),
        58 => Some(Box::new(BulkOperationPacked::<58>::new())),
        59 => Some(Box::new(BulkOperationPacked::<59>::new())),
        60 => Some(Box::new(BulkOperationPacked::<60>::new())),
        61 => Some(Box::new(BulkOperationPacked::<61>::new())),
        62 => Some(Box::new(BulkOperationPacked::<62>::new())),
        63 => Some(Box::new(BulkOperationPacked::<63>::new())),
        64 => Some(Box::new(BulkOperationPacked::<64>::new())),
        _ => None,
    }
}

#[derive(Debug)]
pub struct BulkOperationPacked<const B: u32> {
    long_block_count: u32,
    long_value_count: u32,
    byte_block_count: u32,
    byte_value_count: u32,
    mask: u64,
    int_mask: u32,
}

impl<const B: u32> BulkOperationPacked<B> {
    pub fn new() -> Self {
        let mut blocks = B;
        while blocks & 1 == 0 {
            blocks >>= 1;
        }

        let long_block_count = blocks;
        let long_value_count = 64 * long_block_count / B;
        let mut byte_block_count = 8 * long_block_count;
        let mut byte_value_count = long_value_count;

        while (byte_block_count & 1) == 0 && (byte_value_count & 1) == 0 {
            byte_block_count >>= 1;
            byte_value_count >>= 1;
        }

        let mask = if B == 64 { !0 } else { (1 << B) - 1 };

        assert_eq!(long_value_count * B, 64 * long_block_count);

        Self {
            long_block_count,
            long_value_count,
            byte_block_count,
            byte_value_count,
            mask,
            int_mask: mask as u32,
        }
    }
}

/// Defines the basic methods for a packed bulk operation.
///
/// # Example:
/// ```ignore
/// impl Decoder for BulkOperationPacked<16> {
///    bulk_operation_packed_basic_methods!();
///    /// additional methods...
/// }
/// ```
macro_rules! bulk_operation_packed_basic_methods {
    () => {
        fn long_block_count(&self) -> u32 {
            self.long_block_count
        }

        fn long_value_count(&self) -> u32 {
            self.long_value_count
        }

        fn byte_block_count(&self) -> u32 {
            self.byte_block_count
        }

        fn byte_value_count(&self) -> u32 {
            self.byte_value_count
        }
    };
}

/// Defines the decoder for a packed bulk operation of a given `bits_per_value` using the standard implementation.
///
/// # Usage:
/// `bulk_operation_packed_default_encode!(bits_per_value);`
macro_rules! bulk_operation_packed_default_decode {
    ($bits_per_value:expr) => {
        impl Decoder for BulkOperationPacked<$bits_per_value> {
            bulk_operation_packed_basic_methods!();

            fn decode_u64_to_i64(&mut self, blocks: &[u64], values: &mut [i64], iterations: u32) -> IoResult<()> {
                let mut bits_left: i64 = 64;
                let mut blocks_offset = 0;
                let mut values_offset = 0;
                for i in 0..self.long_value_count * iterations {
                    bits_left -= $bits_per_value;
                    if bits_left < 0 {
                        values[values_offset] =
                            (((blocks[blocks_offset] & ((1 << ($bits_per_value + bits_left)) - 1)) << -bits_left)
                                | (blocks[blocks_offset + 1] >> (64 + bits_left))) as i64;
                        blocks_offset += 1;
                        values_offset += 1;
                        bits_left += 64;
                    } else {
                        values[values_offset] = ((blocks[blocks_offset] >> bits_left) & self.mask) as i64;
                        values_offset += 1;
                    }
                }

                Ok(())
            }

            fn decode_u8_to_i64(&mut self, blocks: &[u8], values: &mut [i64], iterations: u32) -> IoResult<()> {
                let mut next_value: i64 = 0;
                let mut bits_left = $bits_per_value;
                let mut blocks_offset = 0;
                let mut values_offset = 0;
                for i in 0..self.byte_block_count * iterations {
                    let bytes = (blocks[blocks_offset] & 0xff) as i64;
                    blocks_offset += 1;
                    if bits_left > 8 {
                        // just buffer
                        bits_left -= 8;
                        next_value |= bytes << bits_left;
                    } else {
                        // flush
                        let mut bits = 8 - bits_left;
                        values[values_offset] = next_value | (bytes >> bits);
                        values_offset += 1;
                        while bits > $bits_per_value {
                            bits -= $bits_per_value;
                            values[values_offset] = (bytes >> bits) & self.mask as i64;
                            values_offset += 1;
                        }

                        // then buffer
                        bits_left = $bits_per_value - bits;
                        next_value = (bytes & ((1 << bits) - 1)) << bits_left;
                    }
                }

                assert_eq!(bits_left, $bits_per_value);

                Ok(())
            }

            fn decode_u64_to_i32(&mut self, blocks: &[u64], values: &mut [i32], iterations: u32) -> IoResult<()> {
                if $bits_per_value > 32 {
                    return Err(IoError::new(
                        IoErrorKind::Unsupported,
                        "Cannot decode more than 32 bits into an [i32]",
                    ));
                }

                let mut bits_left: i64 = 64;
                let mut blocks_offset = 0;
                let mut values_offset = 0;
                for i in 0..self.long_value_count * iterations {
                    bits_left -= $bits_per_value;
                    if bits_left < 0 {
                        values[values_offset] =
                            (((blocks[blocks_offset] & ((1 << ($bits_per_value + bits_left)) - 1)) << -bits_left)
                                | (blocks[blocks_offset + 1] >> (64 + bits_left))) as i32;
                        blocks_offset += 1;
                        values_offset += 1;
                        bits_left += 64;
                    } else {
                        values[values_offset] = ((blocks[blocks_offset] >> bits_left) & self.mask) as i32;
                        values_offset += 1;
                    }
                }

                Ok(())
            }

            fn decode_u8_to_i32(&mut self, blocks: &[u8], values: &mut [i32], iterations: u32) -> IoResult<()> {
                let mut next_value = 0;
                let mut bits_left = $bits_per_value;
                let mut blocks_offset = 0;
                let mut values_offset = 0;
                for i in 0..self.byte_block_count * iterations {
                    let bytes = (blocks[blocks_offset] & 0xff) as i32;
                    blocks_offset += 1;

                    if bits_left > 8 {
                        // just buffer
                        bits_left -= 8;
                        next_value |= bytes << bits_left;
                    } else {
                        // flush
                        let mut bits = 8 - bits_left;
                        values[values_offset] = next_value | (bytes >> bits);
                        values_offset += 1;
                        while bits > $bits_per_value {
                            bits -= $bits_per_value;
                            values[values_offset] = (bytes >> bits) & self.mask as i32;
                            values_offset += 1;
                        }

                        // then buffer
                        bits_left = $bits_per_value - bits;
                        next_value = (bytes & ((1 << bits) - 1)) << bits_left;
                    }
                }

                assert_eq!(bits_left, $bits_per_value);
                Ok(())
            }
        }
    };
}

/// Defines the encoder for a packed bulk operation of a given `bits_per_value` using the standard implementation.
///
/// # Usage:
/// `bulk_operation_packed_default_encode!(bits_per_value);`
macro_rules! bulk_operation_packed_default_encode {
    ($bits_per_value:expr) => {
        impl Encoder for BulkOperationPacked<$bits_per_value> {
            bulk_operation_packed_basic_methods!();

            fn encode_i64_to_u64(&mut self, values: &[i64], blocks: &mut [u64], iterations: u32) -> IoResult<()> {
                let mut next_block: u64 = 0;
                let mut bits_left = 64;
                let mut blocks_offset = 0;
                let mut values_offset = 0;
                for i in 0..self.long_value_count * iterations {
                    bits_left -= $bits_per_value;
                    if bits_left > 0 {
                        next_block |= (values[values_offset] as u64) << bits_left;
                        values_offset += 1;
                    } else if bits_left == 0 {
                        next_block |= values[values_offset] as u64;
                        values_offset += 1;
                        blocks[blocks_offset] = next_block;
                        blocks_offset += 1;
                        next_block = 0;
                        bits_left = 64;
                    } else {
                        // bits_left < 0
                        next_block |= values[values_offset] as u64 >> -bits_left;
                        blocks[blocks_offset] = next_block;
                        blocks_offset += 1;
                        next_block = ((values[values_offset] & ((1 << -bits_left) - 1)) as u64) << (64 + bits_left);
                        values_offset += 1;
                        bits_left += 64;
                    }
                }
                Ok(())
            }

            fn encode_i32_to_u64(&mut self, values: &[i32], blocks: &mut [u64], iterations: u32) -> IoResult<()> {
                let mut next_block: u64 = 0;
                let mut bits_left = 64;
                let mut blocks_offset = 0;
                let mut values_offset = 0;
                for i in 0..self.long_value_count * iterations {
                    bits_left -= $bits_per_value;
                    if bits_left > 0 {
                        next_block |= ((values[values_offset] & 0xffffffff) as u64) << bits_left;
                        values_offset += 1;
                    } else if bits_left == 0 {
                        next_block |= ((values[values_offset] & 0xffffffff) as u64) << bits_left;
                        values_offset += 1;
                        blocks[blocks_offset] = next_block;
                        blocks_offset += 1;
                        next_block = 0;
                        bits_left = 64;
                    } else {
                        // bits_left < 0
                        next_block |= (values[values_offset] & 0xffffffff) as u64 >> -bits_left;
                        blocks[blocks_offset] = next_block;
                        blocks_offset += 1;
                        next_block = ((values[values_offset] & ((1 << -bits_left) - 1)) as u64) << (64 + bits_left);
                        values_offset += 1;
                        bits_left += 64;
                    }
                }
                Ok(())
            }

            fn encode_i64_to_u8(&mut self, values: &[i64], blocks: &mut [u8], iterations: u32) -> IoResult<()> {
                let mut next_block: u8 = 0;
                let mut bits_left = 8;
                let mut blocks_offset = 0;
                let mut values_offset = 0;
                for i in 0..self.byte_value_count * iterations {
                    let v = values[values_offset];
                    values_offset += 1;
                    assert!(unsigned_bits_required(v) <= $bits_per_value);

                    if $bits_per_value < bits_left {
                        // just buffer
                        next_block |= (v << (bits_left - $bits_per_value)) as u8;
                        bits_left -= $bits_per_value;
                    } else {
                        // flush as many blocks as possible
                        let mut bits = $bits_per_value - bits_left;
                        blocks[blocks_offset] = next_block | (v >> bits) as u8;
                        blocks_offset += 1;
                        while bits >= 8 {
                            bits -= 8;
                            blocks[blocks_offset] = (v >> bits) as u8;
                            blocks_offset += 1;
                        }

                        // then buffer
                        bits_left = 8 - bits;
                        next_block = ((v & ((1 << bits) - 1)) as u8) << bits_left;
                    }
                }

                assert_eq!(bits_left, 8);

                Ok(())
            }

            fn encode_i32_to_u8(&mut self, values: &[i32], blocks: &mut [u8], iterations: u32) -> IoResult<()> {
                let mut next_block: u8 = 0;
                let mut bits_left = 8;
                let mut blocks_offset = 0;
                let mut values_offset = 0;
                for i in 0..self.byte_value_count * iterations {
                    let v = values[values_offset];
                    values_offset += 1;
                    assert!(bits_required(v as i64 & 0xffffffff) <= $bits_per_value);
                    if $bits_per_value < bits_left {
                        // just buffer
                        next_block |= (v << (bits_left - $bits_per_value)) as u8;
                        bits_left -= $bits_per_value;
                    } else {
                        // flush as many blocks as possible
                        let mut bits = $bits_per_value - bits_left;
                        blocks[blocks_offset] = next_block | (v >> bits) as u8;
                        blocks_offset += 1;
                        while bits >= 8 {
                            bits -= 8;
                            blocks[blocks_offset] = (v >> bits) as u8;
                            blocks_offset += 1;
                        }

                        // then buffer
                        bits_left = 8 - bits;
                        next_block = ((v & ((1 << bits) - 1)) as u8) << bits_left;
                    }
                }

                assert_eq!(bits_left, 8);
                Ok(())
            }
        }
    };
}

/// Defines both the encoder and decoder for a packed bulk operation of a given `bits_per_value` using the
/// standard implementation.
///
/// # Usage:
/// `bulk_operation_packed_default!(bits_per_value);`
macro_rules! bulk_operation_packed_default {
    ($bits_per_value:expr) => {
        bulk_operation_packed_default_decode!($bits_per_value);
        bulk_operation_packed_default_encode!($bits_per_value);
        impl BulkOperation for BulkOperationPacked<$bits_per_value> {
            bulk_operation_packed_basic_methods!();
        }
    };
}

macro_rules! unpack {
    ($values:ident, $values_offset:ident, $result:ty, $block:ident, $mask:literal, $shift:expr) => {
        $values[$values_offset] = (($block >> $shift) & $mask) as $result; $values_offset += 1;
    };

    ($values:ident, $values_offset:ident, $result:ty, $block:ident, $mask:literal, $shift:expr, $($more_shifts:expr),+) => {
        unpack!($values, $values_offset, $result, $block, $mask, $shift);
        unpack!($values, $values_offset, $result, $block, $mask, $($more_shifts),+);
    };

    ($result:ty, $prev_block:ident, $prev_mask:literal, $prev_shift:expr, $block:ident, $shift:expr) => {
        values[values_offset] = ((($prev_block << $prev_shift) & prev_mask) | ($block >> $shift)) as $result; values_offset += 1;
    };
}

bulk_operation_packed_default!(25);
bulk_operation_packed_default!(26);
bulk_operation_packed_default!(27);
bulk_operation_packed_default!(28);
bulk_operation_packed_default!(29);
bulk_operation_packed_default!(30);
bulk_operation_packed_default!(31);
bulk_operation_packed_default!(32);
bulk_operation_packed_default!(33);
bulk_operation_packed_default!(34);
bulk_operation_packed_default!(35);
bulk_operation_packed_default!(36);
bulk_operation_packed_default!(37);
bulk_operation_packed_default!(38);
bulk_operation_packed_default!(39);
bulk_operation_packed_default!(40);
bulk_operation_packed_default!(41);
bulk_operation_packed_default!(42);
bulk_operation_packed_default!(43);
bulk_operation_packed_default!(44);
bulk_operation_packed_default!(45);
bulk_operation_packed_default!(46);
bulk_operation_packed_default!(47);
bulk_operation_packed_default!(48);
bulk_operation_packed_default!(49);
bulk_operation_packed_default!(50);
bulk_operation_packed_default!(51);
bulk_operation_packed_default!(52);
bulk_operation_packed_default!(53);
bulk_operation_packed_default!(54);
bulk_operation_packed_default!(55);
bulk_operation_packed_default!(56);
bulk_operation_packed_default!(57);
bulk_operation_packed_default!(58);
bulk_operation_packed_default!(59);
bulk_operation_packed_default!(60);
bulk_operation_packed_default!(61);
bulk_operation_packed_default!(62);
bulk_operation_packed_default!(63);
bulk_operation_packed_default!(64);

impl Decoder for BulkOperationPacked<1> {
    bulk_operation_packed_basic_methods!();

    fn decode_u64_to_i32(&mut self, blocks: &[u64], values: &mut [i32], iterations: u32) -> IoResult<()> {
        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for i in 0..iterations {
            let block = blocks[blocks_offset];
            blocks_offset += 1;
            for shift in (0..=63).rev() {
                unpack!(values, values_offset, i32, block, 1, shift);
            }
        }

        Ok(())
    }

    fn decode_u8_to_i32(&mut self, blocks: &[u8], values: &mut [i32], iterations: u32) -> IoResult<()> {
        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for j in 0..iterations {
            let block = blocks[blocks_offset];
            blocks_offset += 1;
            unpack!(values, values_offset, i32, block, 1, 7, 6, 5, 4, 3, 2, 1, 0);
        }

        Ok(())
    }

    fn decode_u8_to_i64(&mut self, blocks: &[u8], values: &mut [i64], iterations: u32) -> IoResult<()> {
        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for i in 0..iterations {
            let block = blocks[blocks_offset];
            blocks_offset += 1;
            for shift in (0..=63).rev() {
                unpack!(values, values_offset, i64, block, 1, shift);
            }
        }

        Ok(())
    }

    fn decode_u64_to_i64(&mut self, blocks: &[u64], values: &mut [i64], iterations: u32) -> IoResult<()> {
        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for j in 0..iterations {
            let block = blocks[blocks_offset];
            blocks_offset += 1;
            unpack!(values, values_offset, i64, block, 1, 7, 6, 5, 4, 3, 2, 1, 0);
        }

        Ok(())
    }
}

bulk_operation_packed_default_encode!(1);
impl BulkOperation for BulkOperationPacked<1> {
    bulk_operation_packed_basic_methods!();
}

impl Decoder for BulkOperationPacked<2> {
    bulk_operation_packed_basic_methods!();

    fn decode_u64_to_i32(&mut self, blocks: &[u64], values: &mut [i32], iterations: u32) -> IoResult<()> {
        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for i in 0..iterations {
            let block = blocks[blocks_offset];
            blocks_offset += 1;
            for shift in (0..=62).rev() {
                unpack!(values, values_offset, i32, block, 3, shift);
            }
        }

        Ok(())
    }

    fn decode_u8_to_i32(&mut self, blocks: &[u8], values: &mut [i32], iterations: u32) -> IoResult<()> {
        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for j in 0..iterations {
            let block = blocks[blocks_offset];
            blocks_offset += 1;
            unpack!(values, values_offset, i32, block, 3, 6, 4, 2, 0);
        }

        Ok(())
    }

    fn decode_u64_to_i64(&mut self, blocks: &[u64], values: &mut [i64], iterations: u32) -> IoResult<()> {
        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for i in 0..iterations {
            let block = blocks[blocks_offset];
            blocks_offset += 1;
            for shift in (0..=62).rev().step_by(2) {
                unpack!(values, values_offset, i64, block, 3, shift);
            }
        }

        Ok(())
    }

    fn decode_u8_to_i64(&mut self, blocks: &[u8], values: &mut [i64], iterations: u32) -> IoResult<()> {
        let mut blocks_offset = 0;
        let mut values_offset = 0;
        for j in 0..iterations {
            let block = blocks[blocks_offset];
            blocks_offset += 1;
            unpack!(values, values_offset, i64, block, 3, 6, 4, 2, 0);
        }

        Ok(())
    }
}

bulk_operation_packed_default_encode!(2);
impl BulkOperation for BulkOperationPacked<2> {
    bulk_operation_packed_basic_methods!();
}

include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_3.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_4.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_5.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_6.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_7.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_8.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_9.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_10.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_11.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_12.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_13.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_14.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_15.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_16.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_17.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_18.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_19.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_20.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_21.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_22.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_23.rs"));
include!(concat!(env!("OUT_DIR"), "/bulk_operation_packed_24.rs"));
