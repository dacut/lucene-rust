use {
    crate::{codec::CodecHeader, BoxResult, Id, LuceneError},
    tokio::io::AsyncRead,
};

/// A [CodecHeader] that has the magic bytes/name/version, followed by an id, followed by the suffix (name repeated).
#[derive(Debug)]
pub struct IndexHeader {
    codec_header: CodecHeader,
    id: Id,
}

impl IndexHeader {
    /// The name of the codec used to encode the data.
    #[inline]
    pub fn codec(&self) -> &str {
        self.codec_header.codec()
    }

    /// The version of the codec used to encode the data.
    #[inline]
    pub fn version(&self) -> u32 {
        self.codec_header.version()
    }

    /// The id of the index.
    #[inline]
    pub fn id(&self) -> Id {
        self.id
    }

    /// Reads and verifies that the index header has the correct magic bytes, the specified codec name, the version falls
    /// within the specified range, the id matches the specified id, and the suffix matches the codec name.
    pub async fn read_from<R: AsyncRead + Unpin>(
        r: &mut R,
        codec: &str,
        min_version: u32,
        max_version: u32,
        expected_id: Option<Id>,
        expected_suffix: &str,
    ) -> BoxResult<Self> {
        let codec_header = CodecHeader::read(r, codec, min_version, max_version).await?;
        let id = Id::read_from(r).await?;

        if let Some(expected_id) = expected_id {
            if id != expected_id {
                return Err(LuceneError::CorruptIndex(format!(
                    "Index header contained invalid id: got {id}, expected {expected_id}",
                ))
                .into());
            }
        }

        codec_header.read_index_header_suffix(r, expected_suffix).await?;

        Ok(Self {
            codec_header,
            id,
        })
    }
}
