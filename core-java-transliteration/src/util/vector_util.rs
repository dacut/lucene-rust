//! Utilities for computations with numeric arrays.
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// Returns the vector dot product of the two vectors.
///
/// # Panics
/// Panics if the two vectors have different lengths.
pub trait DotProduct {
    type Output;
    fn dot_product(&self, other: &Self) -> Self::Output;
}

impl DotProduct for [f32] {
    type Output = f32;

    fn dot_product(&self, b: &[f32]) -> f32 {
        assert_eq!(self.len(), b.len(), "vector dimensions differ: {} != {}", self.len(), b.len());

        let mut res = 0.0;

        // Just allow the LLVM optimizer to handle this.
        for i in 0..self.len() {
            res += self[i] * b[i];
        }

        res
    }
}

impl DotProduct for [u8] {
    type Output = f32;

    fn dot_product(&self, b: &[u8]) -> f32 {
        let mut total = 0;
        for i in 0..self.len() {
            total += (self[i] as i8) as i32 * (b[i] as i8) as i32;
        }

        total as f32
    }
}

/// Returns the cosine similarity between the two vectors.
///
/// # Panics
/// Panics if the two vectors have different lengths.
pub trait Cosine {
    type Output;
    fn cosine(&self, other: &Self) -> Self::Output;
}

impl Cosine for [f32] {
    type Output = f32;

    fn cosine(&self, v2: &[f32]) -> f32 {
        assert_eq!(self.len(), v2.len(), "vector dimensions differ: {} != {}", self.len(), v2.len());

        let mut sum = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;

        for i in 0..self.len() {
            let elem1 = self[i];
            let elem2 = v2[i];
            sum += elem1 * elem2;
            norm1 += elem1 * elem1;
            norm2 += elem2 * elem2;
        }

        sum / (norm1 * norm2).sqrt()
    }
}

/// Returns the cosine similarity between the two vectors.
///
/// # Panics
/// Panics if the two vectors have different lengths.
impl Cosine for [u8] {
    type Output = f32;
    fn cosine(&self, b: &[u8]) -> f32 {
        assert_eq!(self.len(), b.len(), "vector dimensions differ: {} != {}", self.len(), b.len());

        let mut sum = 0;
        let mut norm1 = 0;
        let mut norm2 = 0;

        for i in 0..self.len() {
            let elem1 = (self[i] as i8) as i64;
            let elem2 = (b[i] as i8) as i64;
            sum += elem1 * elem2;
            norm1 += elem1 * elem1;
            norm2 += elem2 * elem2;
        }

        sum as f32 / (norm1 as f64 * norm2 as f64).sqrt() as f32
    }
}

/// Returns the sum of squared differences of the two vectors.
///
/// # Panics
/// Panics if the two vectors have different lengths.
pub trait SquareDistance {
    type Output;

    fn square_distance(&self, other: &Self) -> Self::Output;
}

impl SquareDistance for [f32] {
    type Output = f32;
    fn square_distance(&self, v2: &[f32]) -> f32 {
        assert_eq!(self.len(), v2.len(), "vector dimensions differ: {} != {}", self.len(), v2.len());

        // Just let LLVM optimize this.
        let mut square_sum = 0.0;
        for i in 0..self.len() {
            let diff = self[i] - v2[i];
            square_sum += diff * diff;
        }

        square_sum
    }
}

impl SquareDistance for [u8] {
    type Output = f32;
    fn square_distance(&self, b: &[u8]) -> f32 {
        assert_eq!(self.len(), b.len(), "vector dimensions differ: {} != {}", self.len(), b.len());

        let mut square_sum = 0;
        for i in 0..self.len() {
            let diff = (self[i] as i8) as i32 - (b[i] as i8) as i32;
            square_sum += diff * diff;
        }

        square_sum as f32
    }
}

pub trait L2Normalize {
    type Output;

    /// Modifies the argument to be unit length, dividing by its l2-norm.
    fn l2_normalize(&self) -> Result<Self::Output, ZeroLengthVectorError>;
}

impl L2Normalize for [f32] {
    type Output = Vec<f32>;

    fn l2_normalize(&self) -> Result<Vec<f32>, ZeroLengthVectorError> {
        let mut square_sum = 0.0;
        for x in self {
            square_sum += x * x;
        }

        if square_sum == 0.0 {
            return Err(ZeroLengthVectorError);
        }

        let len = square_sum.sqrt();
        Ok(self.iter().map(|x| x / len).collect())
    }
}

#[derive(Debug)]
pub struct ZeroLengthVectorError;

impl Display for ZeroLengthVectorError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str("Zero length vector")
    }
}

impl Error for ZeroLengthVectorError {
    fn description(&self) -> &str {
        "Zero length vector"
    }
}

/// Adds the second argument to the first.
pub trait AddAssign {
    fn add_assign(&mut self, other: &Self);
}

impl AddAssign for [f32] {
    fn add_assign(&mut self, other: &[f32]) {
        assert_eq!(self.len(), other.len(), "vector dimensions differ: {} != {}", self.len(), other.len());

        for i in 0..self.len() {
            self[i] += other[i];
        }
    }
}

/// Dot product score, sclaed to be in [0.0, 1.0].
pub trait DotProductScore {
    type Output;
    fn dot_product_score(&self, other: &Self) -> Self::Output;
}

impl DotProductScore for [u8] {
    type Output = f32;
    fn dot_product_score(&self, other: &[u8]) -> f32 {
        // Divide by 2 * 2^145 (maximum absolute value of product of 2 signed bytes) * len
        let denom = (self.len() * (1 << 15)) as f32;
        0.5 + self.dot_product(other) / denom
    }
}

/// Convert a floating point vector to an array of i8s.
///
/// # Panics
/// Panics if any element is out of range for i8.
pub trait ToI8Vec {
    fn to_i8vec(&self) -> Vec<i8>;
}

impl ToI8Vec for [f32] {
    fn to_i8vec(&self) -> Vec<i8> {
        let result = Vec::with_capacity(self.len());
        for (i, x) in self.iter().enumerate() {
            if *x < -128.0 || *x > 127.0 {
                panic!("Value {} at index {} is out of range for i8", *x, i);
            }

            result.push(*x as i8);
        }

        result
    }
}
