use std::{fmt::Debug, ops::Index};

/// Abstraction over an array of longs.
pub trait LongValues: Index<usize, Output = i64> + Debug {}

/// An instance of LongValues that returns the provided value.
#[derive(Debug)]
pub struct Identity;

impl Index<usize> for Identity {
    type Output = i64;

    fn index(&self, index: usize) -> &Self::Output {
        &(index as i64)
    }
}

impl LongValues for Identity {}

/// An instance of LongValues that always returns 0.
#[derive(Debug)]
pub struct Zeroes;

impl Index<usize> for Zeroes {
    type Output = i64;

    fn index(&self, _index: usize) -> &Self::Output {
        &0
    }
}

impl LongValues for Zeroes {}
