mod lucene_90;
mod lucene_95;
mod segment_info;
pub use {lucene_90::*, lucene_95::*, segment_info::*};

use {crate::{codec::{Lucene95Codec, SegmentInfoFormat}, LuceneError}, std::fmt::Debug};

/// Create a new instance of a codec given its name. If the codec is not known, `None` is returned.
///
/// Unlike the Lucene Java implemention, the Rust implementation does not have the ability to dynamically
/// load codecs. Codecs are hard-coded in the [get_codec] function.
///
/// FIXME: This function currently hard codes the available codecs. In the future, it should allow for dynamically
/// loading codecs.
///
/// FIXME: This function currently only handles the `"Lucene95" codec.
pub fn get_codec(name: &str) -> Result<Box<dyn Codec>, LuceneError> {
    match name {
        "Lucene95" => Ok(Box::new(Lucene95Codec::new())),
        _ => Err(LuceneError::UnknownCodec(name.to_string())),
    }
}

/// Encodes and decodes an inverted segment index.
pub trait Codec: Debug {
    /// Returns the Codec's name.
    fn get_name(&self) -> String;

    /// Encodes/decodes segment info file.
    fn segment_info_format(&self) -> Box<dyn SegmentInfoFormat>;
}
