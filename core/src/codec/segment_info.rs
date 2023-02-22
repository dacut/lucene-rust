use {
    crate::{BoxResult, Id, index::SegmentInfo},
    std::{fmt::Debug, path::Path},
};

/// Controls the format of the SegmentInfo (segment metadata file).
pub trait SegmentInfoFormat: Debug {
    /// Read segment info from given a directory, segment name, and segment id.
    fn read_segment_info(&self, directory: &Path, segment_name: &str, segment_id: Id) -> BoxResult<SegmentInfo>;
}