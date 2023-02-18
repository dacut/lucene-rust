#![warn(clippy::all)]

pub mod io;
mod error;
mod version;

pub use error::*;
pub use version::*;
