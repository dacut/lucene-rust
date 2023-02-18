/// Returns the next highest power of two, or the current value if it's already a power of two or zero.
pub trait NextHighestPowerOfTwo {
    fn next_highest_power_of_two(self) -> Self;
}

impl NextHighestPowerOfTwo for u8 {
    fn next_highest_power_of_two(self) -> u8 {
        let mut v = self;
        v -= 1;
        v |= v >> 1;
        v |= v >> 2;
        v |= v >> 4;
        v += 1;
        v
    }
}

impl NextHighestPowerOfTwo for u16 {
    fn next_highest_power_of_two(self) -> u16 {
        let mut v = self;
        v -= 1;
        v |= v >> 1;
        v |= v >> 2;
        v |= v >> 4;
        v |= v >> 8;
        v += 1;
        v
    }
}

impl NextHighestPowerOfTwo for u32 {
    fn next_highest_power_of_two(self) -> u32 {
        let mut v = self;
        v -= 1;
        v |= v >> 1;
        v |= v >> 2;
        v |= v >> 4;
        v |= v >> 8;
        v |= v >> 16;
        v += 1;
        v
    }
}

impl NextHighestPowerOfTwo for u64 {
    fn next_highest_power_of_two(self) -> u64 {
        let mut v = self;
        v -= 1;
        v |= v >> 1;
        v |= v >> 2;
        v |= v >> 4;
        v |= v >> 8;
        v |= v >> 16;
        v |= v >> 32;
        v += 1;
        v
    }
}

impl NextHighestPowerOfTwo for usize {
    fn next_highest_power_of_two(self) -> usize {
        let mut v = self;
        v -= 1;
        v |= v >> 1;
        v |= v >> 2;
        v |= v >> 4;
        v |= v >> 8;
        v |= v >> 16;
        v |= v >> 32;
        v += 1;
        v
    }
}

// magic numbers for bit interleaving
const MAGIC0: u64 = 0x5555555555555555;
const MAGIC1: u64 = 0x3333333333333333;
const MAGIC2: u64 = 0x0F0F0F0F0F0F0F0F;
const MAGIC3: u64 = 0x00FF00FF00FF00FF;
const MAGIC4: u64 = 0x0000FFFF0000FFFF;
const MAGIC5: u64 = 0x00000000FFFFFFFF;
const MAGIC6: u64 = 0xAAAAAAAAAAAAAAAA;

// shift values for bit interleaving
const SHIFT0: u64 = 1;
const SHIFT1: u64 = 2;
const SHIFT2: u64 = 4;
const SHIFT3: u64 = 8;
const SHIFT4: u64 = 16;

/// Interleaves the first 32 bits of each long value
///
/// Adapted from: http://graphics.stanford.edu/~seander/bithacks.html#InterleaveBMN
pub fn interleave(even: u32, odd: u32) -> u64 {
    let mut v1 = 0x00000000FFFFFFFF & even as u64;
    let mut v2 = 0x00000000FFFFFFFF & odd as u64;
    v1 = (v1 | (v1 << SHIFT4)) & MAGIC4;
    v1 = (v1 | (v1 << SHIFT3)) & MAGIC3;
    v1 = (v1 | (v1 << SHIFT2)) & MAGIC2;
    v1 = (v1 | (v1 << SHIFT1)) & MAGIC1;
    v1 = (v1 | (v1 << SHIFT0)) & MAGIC0;
    v2 = (v2 | (v2 << SHIFT4)) & MAGIC4;
    v2 = (v2 | (v2 << SHIFT3)) & MAGIC3;
    v2 = (v2 | (v2 << SHIFT2)) & MAGIC2;
    v2 = (v2 | (v2 << SHIFT1)) & MAGIC1;
    v2 = (v2 | (v2 << SHIFT0)) & MAGIC0;

    (v2 << 1) | v1
}

/// Extract just the even-bits value as a long from the bit-interleaved value
pub fn deinterleave(mut b: u64) -> u64 {
    b &= MAGIC0;
    b = (b ^ (b >> SHIFT0)) & MAGIC1;
    b = (b ^ (b >> SHIFT1)) & MAGIC2;
    b = (b ^ (b >> SHIFT2)) & MAGIC3;
    b = (b ^ (b >> SHIFT3)) & MAGIC4;
    b = (b ^ (b >> SHIFT4)) & MAGIC5;
    b
}

/// flip flops odd with even bits
pub fn flip_flop(b: u64) -> u64 {
    ((b & MAGIC6) >> 1) | ((b & MAGIC0) << 1)
}

/// [Zig-zag](https://developers.google.com/protocol-buffers/docs/encoding#types) encode
/// the provided `i64`. Assuming the input is a signed long whose absolute value can be stored on
/// `n` bits, the returned value will be an unsigned long that can be stored on `n+1` bits.
pub trait ZigZag {
    fn zig_zag_encode(self) -> Self;
    fn zig_zag_decode(self) -> Self;
}

impl ZigZag for i32 {
    fn zig_zag_encode(self) -> i32 {
        (self >> 31) ^ (self << 1)
    }
    
    fn zig_zag_decode(self) -> i32 {
        (self as u32 >> 1) as i32 ^ -(self & 1)
    }
}

impl ZigZag for u32 {
    fn zig_zag_encode(self) -> u32 {
        (self >> 31) ^ (self << 1)
    }
    
    fn zig_zag_decode(self) -> u32 {
        (self >> 1) ^ (-((self & 1) as i32)) as u32
    }
}

impl ZigZag for i64 {
    fn zig_zag_encode(self) -> i64 {
        (self >> 63) ^ (self << 1)
    }
    
    fn zig_zag_decode(self) -> i64 {
        (self as u64 >> 1) as i64 ^ -(self & 1)
    }
}

impl ZigZag for u64 {
    fn zig_zag_encode(self) -> u64 {
        (self >> 63) ^ (self << 1)
    }
    
    fn zig_zag_decode(self) -> u64 {
        (self >> 1) ^ (-((self & 1) as i64)) as u64
    }
}