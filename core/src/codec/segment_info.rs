use {
    crate::{index::SegmentInfo, io::Directory, BoxResult, Id},
    async_trait::async_trait,
    std::fmt::Debug,
};

/// Controls the format of the SegmentInfo (segment metadata file).
#[async_trait(?Send)]
pub trait SegmentInfoFormat: Debug {
    /// Read segment info from given a directory, segment name, and segment id.
    async fn read_segment_info(
        &self,
        directory: &mut dyn Directory,
        segment_name: &str,
        segment_id: Id,
    ) -> BoxResult<SegmentInfo>;
}
