//! Bit mixing utilities. The purpose of these methods is to evenly distribute key space over int32
//! range.
//!
//! Forked from com.carrotsearch.hppc.BitMixer
//!
//! github: https://github.com/carrotsearch/hppc release: 0.9.0

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub const fn mix_u8(key: u8) -> u32 {
    (key as u32).wrapping_mul(PHI_C32)
}

pub const fn mix_i16(key: i16) -> u32 {
    mix_phi_u16(key as u16)
}

pub const fn mix_char(key: char) -> u32 {
    mix_phi_char(key)
}

pub const fn mix_i32(key: i32) -> u32 {
    mix32_u32(key as u32)
}

pub const fn mix_u32(key: u32) -> u32 {
    mix32_u32(key)
}

pub const fn mix_i64(key: i64) -> u32 {
    mix64_u64(key as u64) as u32
}

pub const fn mix_u64(key: u64) -> u32 {
    mix64_u64(key) as u32
}

pub const fn mix_f32(key: f32) -> u32 {
    mix32_u32(key.to_bits())
}

pub const fn mix_f64(key: f64) -> u32 {
    mix64_u64(key.to_bits()) as u32
}

pub fn mix_hashable<T: Hash>(key: &T) -> u32 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    mix64_u64(hasher.finish()) as u32
}

/// MH3's plain finalization step.
pub const fn mix32_u32(k: u32) -> u32 {
    let k = (k ^ (k >> 16)).wrapping_mul(0x85ebca6bu32);
    let k = (k ^ (k >> 13)).wrapping_mul(0xc2b2ae35u32);
    k ^ (k >> 16)
}

/// Computes David Stafford variant 9 of 64bit mix function (MH3 finalization step, with different
/// shifts and constants).
///
/// Variant 9 is picked because it contains two 32-bit shifts which could be possibly optimized
/// into better machine code.
///
/// See: http://zimbry.blogspot.com/2011/09/better-bit-mixing-improving-on.html
pub const fn mix64_u64(z: u64) -> u64 {
    let z = (z ^ (z >> 32)).wrapping_mul(0x4cd6944c5cc20b6d);
    let z = (z ^ (z >> 29)).wrapping_mul(0xfc12c5b19d3259e9);
    z ^ (z >> 32)
}

/// Golden ratio bit mixers.
pub const PHI_C32: u32 = 0x9e3779b9;
pub const PHI_C64: u64 = 0x9e3779b97f4a7c15;

pub const fn mix_phi_u8(k: u8) -> u32 {
    let h = (k as u32).wrapping_mul(PHI_C32);
    h ^ (h >> 16)
}

pub const fn mix_phi_char(k: char) -> u32 {
    let h = (k as u32).wrapping_mul(PHI_C32);
    h ^ (h >> 16)
}

pub const fn mix_phi_u16(k: u16) -> u32 {
    let h = (k as u32).wrapping_mul(PHI_C32);
    h ^ (h >> 16)
}

pub const fn mix_phi_u32(k: u32) -> u32 {
    let h = k.wrapping_mul(PHI_C32);
    h ^ (h >> 16)
}

pub const fn mix_phi_f32(k: f32) -> u32 {
    let h = k.to_bits().wrapping_mul(PHI_C32);
    h ^ (h >> 16)
}

pub const fn mix_phi_f64(k: f64) -> u64 {
    let h = k.to_bits().wrapping_mul(PHI_C64);
    h ^ (h >> 32)
}

pub const fn mix_phi_u64(k: u64) -> u64 {
    let h = k.wrapping_mul(PHI_C64);
    h ^ (h >> 32)
}

pub fn mix_phi_hashable<T: Hash>(k: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    k.hash(&mut hasher);
    mix_phi_u64(hasher.finish())
}
