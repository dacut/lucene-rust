/// Rust equivalent for `java.lang.Number`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[allow(non_camel_case_types)]
pub enum Number {
    i8(i8),
    i16(i16),
    i32(i32),
    i64(i64),
    i128(i128),
    isize(isize),
    u8(u8),
    u16(u16),
    u32(u32),
    u64(u64),
    u128(u128),
    usize(usize),
    f32(f32),
    f64(f64),
}

impl Eq for Number {}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Number::i8(a), Number::i8(b)) => a.cmp(b),
            (Number::i16(a), Number::i16(b)) => a.cmp(b),
            (Number::i32(a), Number::i32(b)) => a.cmp(b),
            (Number::i64(a), Number::i64(b)) => a.cmp(b),
            (Number::i128(a), Number::i128(b)) => a.cmp(b),
            (Number::isize(a), Number::isize(b)) => a.cmp(b),
            (Number::u8(a), Number::u8(b)) => a.cmp(b),
            (Number::u16(a), Number::u16(b)) => a.cmp(b),
            (Number::u32(a), Number::u32(b)) => a.cmp(b),
            (Number::u64(a), Number::u64(b)) => a.cmp(b),
            (Number::u128(a), Number::u128(b)) => a.cmp(b),
            (Number::usize(a), Number::usize(b)) => a.cmp(b),
            (Number::f32(a), Number::f32(b)) => a.partial_cmp(b).unwrap(),
            (Number::f64(a), Number::f64(b)) => a.partial_cmp(b).unwrap(),
            _ => panic!("Cannot compare different types"),
        }
    }
}