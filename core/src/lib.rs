//! Core Lucene functionality.

#![warn(clippy::all)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(missing_docs)]

mod error;
mod id;
mod version;

/// Codec related types and functionality.
pub mod codec;

/// Lucene index-on-disk types and functionality.
pub mod fs;

/// Generic Lucene I/O types.
pub mod io;

/// Lucene index (database) types.
pub mod index;

/// Lucene search types.
pub mod search;

pub use {error::*, id::*, io::*, version::*};
