use {
    byteorder::{ReadBytesExt, LE},
    crate::{
        codec::SegmentInfoFormat,
        io::{CodecReadExt, Crc32Reader},
        index::{IndexHeader, SegmentInfo},
        search::{Sort, get_sort_field_provider},
        BoxResult, Id, Version, LuceneError,
    },
    std::{
        fs::File,
        path::{Path, PathBuf},
    },
};

const CODEC_NAME: &str = "Lucene90SegmentInfo";
const VERSION_START: u32 = 0;
const VERSION_CURRENT: u32 = 0;

#[derive(Debug)]
pub struct Lucene90SegmentInfoFormat {}

impl Lucene90SegmentInfoFormat {
    pub fn new() -> Self {
        Self {}
    }

    fn read_segment_info_from<R: CodecReadExt>(&self, mut r: Crc32Reader<R>, segment_name: &str, segment_id: Id) -> BoxResult<SegmentInfo> {
        IndexHeader::read_from(&mut r, CODEC_NAME, VERSION_START, VERSION_CURRENT, Some(segment_id), "")?;
        let version = Version::read_from_i32_le(&mut r)?;
        let has_min_version = r.read_u8()?;
        let min_version = match has_min_version {
            0 => None,
            1 => Some(Version::read_from_i32_le(&mut r)?),
            _ => return Err(LuceneError::CorruptIndex(format!("Invalid has_min_version value found in segment index: {has_min_version}")).into()),
        };

        let doc_count = r.read_i32::<LE>()?;
        if doc_count < 0 {
            return Err(LuceneError::CorruptIndex(format!("Invalid doc_count value found in segment index: {doc_count}")).into());
        }
        let doc_count = doc_count as u32;
        let is_compound_file = r.read_u8()? == 1;
        let diagnostics = r.read_string_map()?;
        let files = r.read_string_set()?;
        let attributes = r.read_string_map()?;

        let num_sort_fields = r.read_vi32()?;
        if num_sort_fields < 0 {
            return Err(LuceneError::CorruptIndex(format!("Invalid num_sort_fields value found in segment index: {num_sort_fields}")).into());
        }

        let index_sort = if num_sort_fields == 0 {
            None
        } else {
            let mut sort_fields = Vec::with_capacity(num_sort_fields as usize);
            for _ in 0..num_sort_fields {
                let provider_name = r.read_string()?;
                sort_fields.push(get_sort_field_provider(&provider_name)?.read_sort_field(&mut r)?);
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

impl SegmentInfoFormat for Lucene90SegmentInfoFormat {
    fn read_segment_info(&self, directory: &Path, segment_name: &str, segment_id: Id) -> BoxResult<SegmentInfo> {
        let mut segment_file_name = PathBuf::with_capacity(segment_name.len() + 3);
        segment_file_name.set_file_name(segment_name);
        segment_file_name.set_extension("si");
        let segment_file_path = directory.join(segment_file_name);
        let segment_file = File::open(segment_file_path)?;

        self.read_segment_info_from(Crc32Reader::new(segment_file), segment_name, segment_id)
    }
}
