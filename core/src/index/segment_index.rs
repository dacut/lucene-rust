use {
    crate::{
        codec::get_codec,
        index::{IndexHeader, SegmentCommitInfo, MAX_DOCS},
        io::{Crc32Reader, Directory, EncodingReadExt},
        BoxResult, Id, LuceneError, Version,
    },
    log::{debug, error},
    std::collections::HashMap,
    tokio::io::AsyncReadExt,
};

/// Index segment file name prefix.
pub const INDEX_SEGMENT_FILE_NAME_PREFIX: &str = "segments";

/// Suffix for Lucene < 4.0 index segment file names.
pub const PRE_40_INDEX_SEGMENT_FILE_NAME_SUFFIX: &str = ".gen";

/// Pending index segment file name prefix.
pub const PENDING_INDEX_SEGMENT_FILE_NAME_PREFIX: &str = "pending_segments";

/// The segment index version at the time when 8.0 was released.
pub const SEGMENT_INDEX_VERSION_7_4: u32 = 9;

/// The segment index version that recorded SegmentCommitInfo IDs.
pub const SEGMENT_INDEX_VERSION_8_6: u32 = 10;

const SEGMENT_INDEX_VERSION_CURRENT: u32 = SEGMENT_INDEX_VERSION_8_6;

/// The name of the segment index codec.
pub const SEGMENT_CODEC_NAME: &str = "segments";

/// Information about a Lucene index. This is in the `segments_N` file.
///
/// In the Lucene Java implementation, this is called `SegmentInfos`.
#[derive(Debug)]
pub struct SegmentIndex {
    /// Used to name new segments.
    counter: u64,

    /// Counts how often the index has been changed.
    version: u64,

    /// generation of the "segments_N" for the next commit
    generation: u64,

    /// generation of the "segments_N" file we last successfully read
    /// or wrote; this is normally the same as generation except if
    /// there was an IOException that had interrupted a commit
    last_generation: u64,

    /// Opaque Map (String -> String) that user can specify during IndexWriter::commit().
    user_data: HashMap<String, String>,

    segments: Vec<SegmentCommitInfo>,

    /// Id for this commit; only written starting with Lucene 5.0
    id: Id,

    /// Version of the oldest segment in the index, or null if there are no segments.
    lucene_version: Version,

    /// The Lucene version major that was used to create the index.
    index_created_version_major: u8,
}

impl SegmentIndex {
    /// Returns the id of the segment index.
    #[inline]
    pub fn get_id(&self) -> Id {
        self.id
    }

    /// Returns the Lucene version used to create the index.
    #[inline]
    pub fn get_lucene_version(&self) -> Version {
        self.lucene_version
    }

    /// Returns the Lucene version major that was used to create the index.
    #[inline]
    pub fn get_index_created_version_major(&self) -> u8 {
        self.index_created_version_major
    }

    /// Returns the generation of the "segments_N" file for the next commit.
    #[inline]
    pub fn get_generation(&self) -> u64 {
        self.generation
    }

    /// Returns the generation of the "segments_N" file we last successfully read or wrote.
    #[inline]
    pub fn get_last_generation(&self) -> u64 {
        self.last_generation
    }

    /// Counts how often the index has been changed.
    #[inline]
    pub fn get_version(&self) -> u64 {
        self.version
    }

    /// Used to name new segments.
    #[inline]
    pub fn get_counter(&self) -> u64 {
        self.counter
    }

    /// Opaque user data that is associated with the index.
    #[inline]
    pub fn get_user_data(&self) -> &HashMap<String, String> {
        &self.user_data
    }

    /// Returns the segments of the index.
    #[inline]
    pub fn get_segments(&self) -> &[SegmentCommitInfo] {
        &self.segments
    }

    /// Open a segment index from the given directory.
    pub async fn open<D: Directory>(directory: &mut D) -> BoxResult<Self> {
        let dir_entries = directory.read_dir().await?;
        let Some((segment_index_file_name, generation)) = get_latest_segment_index_file_name_and_generation(&dir_entries)? else {
            return Err(LuceneError::CorruptIndex(format!("No segment index file found in directory: {directory:?}")).into());
        };

        let segment_index_file = directory.open(&segment_index_file_name).await?;
        let mut segment_index_reader = Crc32Reader::new(segment_index_file);
        Self::read_from(directory, &mut segment_index_reader, generation).await
    }

    /// Read the segment index from the given reader.
    pub async fn read_from<D: Directory, R: EncodingReadExt>(
        directory: &mut D,
        r: &mut Crc32Reader<R>,
        generation: u64,
    ) -> BoxResult<Self> {
        // From SegmentInfos#readCommit(Directory, ChecksumIndexInput, long, int)
        let gen_str = generation_to_string(generation);
        let index_header = IndexHeader::read_from(
            r,
            SEGMENT_CODEC_NAME,
            SEGMENT_INDEX_VERSION_7_4,
            SEGMENT_INDEX_VERSION_CURRENT,
            None,
            &gen_str,
        )
        .await?;
        let format = index_header.version();

        let lucene_version = Version::read_from_vi32(r).await?;
        debug!("SegmentIndex has Lucene version {lucene_version}");

        let index_created_version_major = r.read_vi32().await?;
        debug!("SegmentIndex has index created version major {index_created_version_major}");

        if (lucene_version.major() as i32) < index_created_version_major {
            return Err(LuceneError::CorruptIndex(format!("Segment index has version {index_created_version_major} but is greater than the Lucene version that created it: {lucene_version}")).into());
        }

        let index_created_version_major: u8 = index_created_version_major.try_into().map_err(|_| {
            LuceneError::CorruptIndex(format!(
                "Index created version {index_created_version_major} is too large to fit in a u8"
            ))
        })?;

        // From SegmentInfos#parseSegmentInfos(Directory, DataInput, SegmentInfos, int)

        let version = r.read_i64().await?;
        assert!(version >= 0);
        let version = version as u64;

        let counter = r.read_vi64().await?;
        assert!(counter >= 0);
        let counter = counter as u64;

        let num_segments = r.read_i32().await?;
        debug!("SegmentIndex has {num_segments} segments; version={version}, counter={counter}");

        if num_segments < 0 {
            return Err(LuceneError::CorruptIndex(format!(
                "Segment index has negative number of segments: {num_segments}"
            ))
            .into());
        }

        let min_segment_lucene_version = if num_segments > 0 {
            Some(Version::read_from_vi32(r).await?)
        } else {
            None
        };

        let mut total_docs = 0;
        let mut segments = Vec::with_capacity(num_segments as usize);

        for seg in 0..num_segments as usize {
            let seg_name = r.read_string().await?;
            let seg_id = Id::read_from(r).await?;
            let codec_name = r.read_string().await?;

            debug!("Segment {seg} has name {seg_name}, id {seg_id}, using codec {codec_name}");

            let codec = get_codec(&codec_name)?;
            let segment_info_format = codec.segment_info_format();
            let segment_info = segment_info_format.read_segment_info(directory, &seg_name, seg_id).await?;

            let max_doc = segment_info.get_max_doc();
            total_docs += max_doc;

            let del_gen = r.read_i64().await?;
            let del_count = r.read_i32().await?;
            let field_infos_gen = r.read_i64().await?;
            let dv_gen = r.read_i64().await?;
            let soft_del_count = r.read_i32().await?;

            debug!("Segment {seg_name} has max_doc={max_doc}, del_gen={del_gen}, del_count={del_count}, field_infos_gen={field_infos_gen}, dv_gen={dv_gen}, soft_del_count={soft_del_count}");

            // Make del_gen more Rust friendly.
            let del_gen = if del_gen < 0 {
                None
            } else {
                Some(del_gen as u64)
            };

            // Ensure del_count is valid and Rust friendly.
            if del_count < 0 || del_count as u32 > max_doc {
                return Err(LuceneError::CorruptIndex(format!(
                    "Segment index has deletion count {del_count} greater than max docs {}",
                    segment_info.get_max_doc()
                ))
                .into());
            }
            let del_count = del_count as u32;

            // Make field_infos_gen more Rust friendly.
            let field_infos_gen = if field_infos_gen < 0 {
                None
            } else {
                Some(field_infos_gen as u64)
            };

            // Make dv_gen more Rust friendly.
            let dv_gen = if dv_gen < 0 {
                None
            } else {
                Some(dv_gen as u64)
            };

            // Ensure soft_del_count is valid and Rust friendly.
            if soft_del_count < 0 || soft_del_count as u32 > max_doc {
                return Err(LuceneError::CorruptIndex(format!(
                    "Segment index has soft deletion count {soft_del_count} greater than max docs {}",
                    segment_info.get_max_doc()
                ))
                .into());
            }
            let soft_del_count = soft_del_count as u32;

            // Make sure we don't have more deleted documents than the total number of documents.
            if soft_del_count + del_count > max_doc {
                return Err(LuceneError::CorruptIndex(format!(
                    "Segment index has invalid total deletion count {} greater than max docs {}",
                    soft_del_count + del_count,
                    segment_info.get_max_doc()
                ))
                .into());
            }

            let sci_id = if format > SEGMENT_INDEX_VERSION_7_4 {
                match r.read_u8().await? {
                    1 => Some(Id::read_from(r).await?),
                    0 => None,
                    other => {
                        return Err(LuceneError::CorruptIndex(format!(
                            "Segment index has SegmentCommitInfo marker: {other}"
                        ))
                        .into())
                    }
                }
            } else {
                None
            };

            let mut si_per_commit = SegmentCommitInfo::new(
                segment_info,
                del_count,
                soft_del_count,
                del_gen,
                field_infos_gen,
                dv_gen,
                sci_id,
            );

            si_per_commit.set_field_infos_files(r.read_string_set().await?);
            let n_dv_fields = r.read_i32().await?;
            if n_dv_fields > 0 {
                let mut dv_fields = HashMap::new();
                for _ in 0..n_dv_fields {
                    let key = r.read_i32().await?;
                    let values = r.read_string_set().await?;
                    dv_fields.insert(key, values);
                }

                si_per_commit.set_doc_values_update_files(dv_fields);
            }

            let segment_version = si_per_commit.get_version();

            // We guarantee that min_segment_lucene_version is not None because num_segments > 0
            if segment_version < min_segment_lucene_version.unwrap() {
                return Err(LuceneError::CorruptIndex(format!(
                    "Segment index has segment version {segment_version} less than min segment version {}",
                    min_segment_lucene_version.unwrap()
                ))
                .into());
            }

            if index_created_version_major >= 7 && segment_version.major() < index_created_version_major {
                return Err(LuceneError::CorruptIndex(format!(
                    "Segment index has segment version {segment_version} less than index created version {index_created_version_major}")).into());
            }

            if index_created_version_major >= 7 && si_per_commit.get_min_version().is_none() {
                return Err(LuceneError::CorruptIndex(format!(
                    "Segment infos must record a min version when created with index major version {index_created_version_major}")).into());
            }
            segments.push(si_per_commit);
        }

        let user_data = r.read_string_map().await?;

        let segment_index = Self {
            id: index_header.id(),
            lucene_version,
            index_created_version_major,
            generation,
            last_generation: generation,
            version,
            counter,
            user_data,
            segments,
        };

        if total_docs > MAX_DOCS {
            return Err(LuceneError::TooManyDocs(total_docs as u64).into());
        }

        Ok(segment_index)
    }
}

/// Get the latest index segment file and its generation of the most recent commit.
pub fn get_latest_segment_index_file_name_and_generation<T: AsRef<str>>(
    files: &[T],
) -> BoxResult<Option<(String, u64)>> {
    let mut result = None;

    for file_name in files {
        let file_name = file_name.as_ref();

        // Ignore files whose name doesn't start with "segments".
        let Some(suffix) = file_name.strip_prefix(INDEX_SEGMENT_FILE_NAME_PREFIX) else {
            debug!("File {file_name:?} doesn't start with {INDEX_SEGMENT_FILE_NAME_PREFIX:?}, skipping");
            continue;
        };

        if suffix == PRE_40_INDEX_SEGMENT_FILE_NAME_SUFFIX {
            return Err(LuceneError::UnsupportedLuceneVersion(format!(
                "Index segment file {:?} is unsupported version from pre-4.0",
                file_name
            ))
            .into());
        }

        let this_generation = if suffix.is_empty() {
            debug!("File {file_name:?} has no generation suffix, using 0");
            0
        } else {
            let Ok(generation) = suffix[1..].parse::<u64>() else {
                error!("Failed to parse generation from file name {:?}", file_name);
                continue;
            };
            debug!("File {file_name:?} has generation {generation}");
            generation
        };

        result = match result {
            None => {
                debug!("No previous result; setting to {file_name:?} with generation {this_generation}");
                Some((file_name.to_string(), this_generation))
            }
            Some((cur_highest_file_name, cur_highest_generation)) => {
                if this_generation > cur_highest_generation {
                    debug!("New generation {this_generation} is higher than current highest generation {cur_highest_generation}; setting to {file_name:?} with generation {this_generation}");
                    Some((file_name.to_string(), this_generation))
                } else {
                    debug!("New generation {this_generation} is lower than current highest generation {cur_highest_generation}; keeping {cur_highest_file_name:?} with generation {cur_highest_generation}");
                    Some((cur_highest_file_name, cur_highest_generation))
                }
            }
        };
    }

    debug!("Found latest segment index file name and generation: {result:?}");
    Ok(result)
}

/// Convert a generation to its string representation (in base-36)
pub fn generation_to_string(mut gen: u64) -> String {
    let mut result = Vec::with_capacity(10);

    loop {
        let digit = gen % 36;
        gen /= 36;
        result.push(char::from_digit(digit as u32, 36).unwrap());

        if gen == 0 {
            break;
        }
    }

    result.iter().rev().collect()
}
