use {
    crate::{BoxResult, LuceneError},
    log::error,
    std::{
        ffi::OsString,
        fs::{metadata, DirEntry},
        io::Result as IoResult,
    },
};

/// Index segment file name prefix.
pub const INDEX_SEGMENT_FILE_NAME_PREFIX: &str = "segments";

/// Suffix for Lucene < 4.0 index segment file names.
pub const PRE_40_INDEX_SEGMENT_FILE_NAME_SUFFIX: &str = ".gen";

/// Pending index segment file name prefix.
pub const PENDING_INDEX_SEGMENT_FILE_NAME_PREFIX: &str = "pending_segments";

/// Get the latest index segment file and its generation of the most recent commit.
pub fn get_latest_segment_index_filename_and_generation<T: Iterator<Item = IoResult<DirEntry>>>(
    files: T,
) -> BoxResult<Option<(OsString, u64)>> {
    let mut result = None;

    for entry_result in files {
        let entry = entry_result?;
        let file_name_os = entry.file_name();
        let Some(file_name) = file_name_os.to_str() else {
            error!("Failed to convert file name {:?} to string", file_name_os);
            continue;
        };

        // Ignore files whose name doesn't start with "segments".
        let Some(suffix) = file_name.strip_prefix(INDEX_SEGMENT_FILE_NAME_PREFIX) else {
            continue;
        };

        // Not in Java: make sure this is a regular file.
        // Don't use entry.file_type here; it doesn't follow symlinks.
        let entry_metadata = match metadata(entry.path()) {
            Ok(md) => md,
            Err(e) => {
                error!("Failed to get metadata for file {:?}: {}", entry.path(), e);
                continue;
            }
        };

        if !entry_metadata.is_file() {
            error!("File {:?} is not a regular file", entry.path());
            continue;
        }

        if suffix == PRE_40_INDEX_SEGMENT_FILE_NAME_SUFFIX {
            return Err(LuceneError::UnsupportedVersion(format!(
                "Index segment file {:?} is unsupported version from pre-4.0",
                entry.path()
            ))
            .into());
        }

        let this_generation = if suffix.is_empty() {
            0
        } else {
            let Ok(generation) = suffix[1..].parse::<u64>() else {
                error!("Failed to parse generation from file name {:?}", file_name);
                continue;
            };
            generation
        };

        result = match result {
            None => Some((file_name_os, this_generation)),
            Some((cur_highest_file_name, cur_highest_generation)) => {
                if this_generation > cur_highest_generation {
                    Some((file_name_os, this_generation))
                } else {
                    Some((cur_highest_file_name, cur_highest_generation))
                }
            }
        };
    }

    Ok(result)
}
