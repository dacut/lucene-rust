/// Returns the minimum of two f32 values.
///
/// If either value is NaN, then NaN is returned.
pub fn f32_min(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        a
    } else if b.is_nan() {
        b
    } else if a < b {
        a
    } else {
        b
    }
}

/// Returns the maximum of two f32 values.
///
/// If either value is NaN, then NaN is returned.
pub fn f32_max(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        a
    } else if b.is_nan() {
        b
    } else if a > b {
        a
    } else {
        b
    }
}

/// Returns the minimum of two f64 values.
///
/// If either value is NaN, then NaN is returned.
pub fn f64_min(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        a
    } else if b.is_nan() {
        b
    } else if a < b {
        a
    } else {
        b
    }
}

/// Returns the maximum of two f64 values.
///
/// If either value is NaN, then NaN is returned.
pub fn f64_max(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        a
    } else if b.is_nan() {
        b
    } else if a > b {
        a
    } else {
        b
    }
}
