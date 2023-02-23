use crate::codec::{Codec, Lucene90SegmentInfoFormat, SegmentInfoFormat};

#[derive(Debug)]
pub struct Lucene95Codec {}

impl Default for Lucene95Codec {
    fn default() -> Self {
        Self::new()
    }
}

impl Lucene95Codec {
    pub fn new() -> Self {
        Self {}
    }
}

impl Codec for Lucene95Codec {
    fn get_name(&self) -> String {
        "Lucene95".to_string()
    }

    fn segment_info_format(&self) -> Box<dyn SegmentInfoFormat> {
        Box::new(Lucene90SegmentInfoFormat::new())
    }
}
