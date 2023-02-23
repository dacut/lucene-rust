mod lucene_90;
mod lucene_95;
mod segment_info;
pub use {lucene_90::*, lucene_95::*, segment_info::*};

use {
    crate::{
        codec::{Lucene95Codec, SegmentInfoFormat},
        io::{EncodingReadExt, EncodingWriteExt},
        BoxResult, LuceneError,
    },
    std::{fmt::Debug, io::Result as IoResult},
    tokio::io::{AsyncRead, AsyncReadExt},
};

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

/// Constant to identify the start of a codec header.
pub const CODEC_MAGIC: [u8; 4] = [0x3f, 0xd7, 0x6c, 0x17];

/// Constant to identify the start of a codec footer -- bit inversion of [CODEC_MAGIC].
pub const FOOTER_MAGIC: [u8; 4] = [0xc0, 0x28, 0x93, 0xe8];

/// A basic Codec header that has undefined contents between the magic bytes/name/version and the suffix.
#[derive(Debug)]
pub struct CodecHeader {
    codec: String,
    version: u32,
}

impl CodecHeader {
    #[inline]
    /// The name of the codec used to encode the data.
    pub fn codec(&self) -> &str {
        &self.codec
    }

    #[inline]
    /// The version of the codec used to encode the data.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Create a new codec header from the given codec name and version.
    ///
    /// This returns an error if the codec name is too long or contains invalid characters.
    pub fn new(codec: &str, version: u32) -> Result<Self, LuceneError> {
        if codec.len() > 127 {
            return Err(LuceneError::InvalidCodecName(codec.to_string()));
        }

        if !codec.is_ascii() {
            return Err(LuceneError::InvalidCodecName(codec.to_string()));
        }

        Ok(Self {
            codec: codec.to_string(),
            version,
        })
    }

    /// Reads and verifies that the codec header has the correct magic bytes, the specified codec name, and that the version falls
    /// within the specified range.
    pub async fn read<R: AsyncRead + Unpin>(
        r: &mut R,
        codec: &str,
        min_version: u32,
        max_version: u32,
    ) -> BoxResult<Self> {
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic).await?;

        if magic != CODEC_MAGIC {
            return Err(LuceneError::InvalidCodecHeaderMagic(magic).into());
        }

        let actual_codec = r.read_string().await?;
        if actual_codec != codec {
            return Err(LuceneError::IncorrectCodecName(actual_codec.into_bytes(), codec.to_string()).into());
        }

        let version = r.read_u32().await?;
        if version < min_version || version > max_version {
            return Err(
                LuceneError::UnsupportedCodecVersion(codec.to_string(), version, min_version, max_version).into()
            );
        }

        Ok(Self {
            codec: codec.to_string(),
            version,
        })
    }

    /// Reads and verifies the suffix of an index header.
    pub async fn read_index_header_suffix<R: EncodingReadExt + Unpin>(
        &self,
        r: &mut R,
        expected: &str,
    ) -> BoxResult<()> {
        let suffix = r.read_short_string().await?;
        if suffix != expected {
            return Err(LuceneError::CorruptIndex(format!(
                "Codec header suffix contained invalid codec name: got {suffix:?}, expected {expected:?}"
            ))
            .into());
        }

        Ok(())
    }

    /// Writes a codec header, which records both a string to identify the file and a version number.
    ///
    /// CodecHeader --> Magic + CodecName + Version
    ///
    /// * Magic (4 bytes): This identifies the start of the header and is always [CODEC_MAGIC].
    /// * CodecName ([EncodingWriteExt::write_string]): This is a string to identify this file. This must be 127 bytes or less and in ASCII.
    /// * Version (BE u32): Records the version of the file.
    pub async fn write<W: EncodingWriteExt + Unpin>(&self, w: &mut W) -> IoResult<()> {
        w.write_all(&CODEC_MAGIC).await?;
        w.write_string(&self.codec).await?;
        w.write_u32(self.version).await?;
        Ok(())
    }
}
