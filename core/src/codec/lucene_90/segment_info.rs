use {
    crate::{
        codec::SegmentInfoFormat,
        index::{IndexHeader, SegmentInfo},
        io::{Crc32Reader, Directory, EncodingReadExt},
        search::{get_sort_field_provider, Sort},
        BoxResult, Id, LuceneError, Version,
    },
    async_trait::async_trait,
    tokio::io::{AsyncRead, AsyncReadExt},
};

const CODEC_NAME: &str = "Lucene90SegmentInfo";
const VERSION_START: u32 = 0;
const VERSION_CURRENT: u32 = 0;

#[derive(Debug)]
/// Lucene 9.0 segment info (`.si`) file format
pub struct Lucene90SegmentInfoFormat {}

impl Lucene90SegmentInfoFormat {
    /// Create a new instance of [Lucene90SegmentInfoFormat]
    pub fn new() -> Self {
        Self {}
    }

    async fn read_segment_info_from<R: AsyncRead + Unpin>(
        &self,
        r: &mut Crc32Reader<R>,
        segment_name: &str,
        segment_id: Id,
    ) -> BoxResult<SegmentInfo> {
        IndexHeader::read_from(r, CODEC_NAME, VERSION_START, VERSION_CURRENT, Some(segment_id), "").await?;
        let version = Version::read_from_i32_le(r).await?;
        let has_min_version = r.read_u8().await?;
        let min_version = match has_min_version {
            0 => None,
            1 => Some(Version::read_from_i32_le(r).await?),
            _ => {
                return Err(LuceneError::CorruptIndex(format!(
                    "Invalid has_min_version value found in segment index: {has_min_version}"
                ))
                .into())
            }
        };

        let doc_count = r.read_i32_le().await?;
        if doc_count < 0 {
            return Err(LuceneError::CorruptIndex(format!(
                "Invalid doc_count value found in segment index: {doc_count}"
            ))
            .into());
        }
        let doc_count = doc_count as u32;
        let is_compound_file = r.read_u8().await? == 1;
        let diagnostics = r.read_string_map().await?;
        let files = r.read_string_set().await?;
        let attributes = r.read_string_map().await?;

        let num_sort_fields = r.read_vi32().await?;
        if num_sort_fields < 0 {
            return Err(LuceneError::CorruptIndex(format!(
                "Invalid num_sort_fields value found in segment index: {num_sort_fields}"
            ))
            .into());
        }

        let index_sort = if num_sort_fields == 0 {
            None
        } else {
            let mut sort_fields = Vec::with_capacity(num_sort_fields as usize);
            for _ in 0..num_sort_fields {
                let provider_name = r.read_string().await?;
                sort_fields.push(get_sort_field_provider(&provider_name)?.read_sort_field(r).await?);
            }
            Some(Sort::from_fields(sort_fields)?)
        };

        Ok(SegmentInfo {
            version,
            min_version,
            name: segment_name.to_string(),
            max_doc: doc_count,
            is_compound_file,
            diagnostics,
            id: segment_id,
            attributes,
            index_sort,
            files,
        })
    }
}

impl Default for Lucene90SegmentInfoFormat {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl SegmentInfoFormat for Lucene90SegmentInfoFormat {
    async fn read_segment_info(
        &self,
        directory: &mut dyn Directory,
        segment_name: &str,
        segment_id: Id,
    ) -> BoxResult<SegmentInfo> {
        let mut segment_file_name = String::with_capacity(segment_name.len() + 3);
        segment_file_name.push_str(segment_name);
        segment_file_name.push_str(".si");
        let fd = directory.open(&segment_file_name).await?;
        self.read_segment_info_from(&mut Crc32Reader::new(fd), segment_name, segment_id).await
    }
}
