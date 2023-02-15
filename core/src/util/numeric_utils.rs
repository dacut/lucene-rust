/// Converts an `f64` value to a sortable signed `i64`. The value is
/// converted by getting their IEEE 754 floating-point "double format" bit layout and
/// then some bits are swapped, to be able to compare the result as i64. By this the precision is
/// not reduced, but the value can easily used as an i64. The sort order places [f64::NAN] greater than
/// positive infinity.
pub fn double_to_sortable_long(value: f64) -> i64 {
    sortable_double_bits(f64::to_bits(value) as i64)
}

/// Converts IEEE 754 representation of a double to sortable order (or back to the original)
pub fn sortable_double_bits(bits: i64) -> i64 {
    bits ^ (bits >> 63) & 0x7fffffffffffffff
}

/// Converts an `f32` value to a sortable signed `i32`. The value is
/// converted by getting their IEEE 754 floating-point "float format" bit layout and then
/// some bits are swapped, to be able to compare the result as i32. By this the precision is not
/// reduced, but the value can easily used as an i32. The sort order places [f32::NAN] greater than
/// positive infinity.
pub fn float_to_sortable_int(value: f32) -> i32 {
    sortable_float_bits(f32::to_bits(value) as i32)
}

/// Converts IEEE 754 representation of a float to sortable order (or back to the original)
pub fn sortable_float_bits(bits: i32) -> i32 {
    bits ^ (bits >> 31) & 0x7ffffffff
}

