use {
    crate::{search::Sort, Id, Version},
    std::collections::{HashMap, HashSet},
};

#[derive(Debug)]
pub struct SegmentInfo {
    pub(crate) name: String,
    pub(crate) id: Id,
    pub(crate) max_doc: u32,
    pub(crate) attributes: HashMap<String, String>,
    pub(crate) diagnostics: HashMap<String, String>,
    pub(crate) files: HashSet<String>,
    pub(crate) version: Version,
    pub(crate) min_version: Option<Version>,
    pub(crate) is_compound_file: bool,
    pub(crate) index_sort: Option<Sort>,
}

impl SegmentInfo {
    #[inline]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn get_id(&self) -> Id {
        self.id
    }

    #[inline]
    pub fn get_max_doc(&self) -> u32 {
        self.max_doc
    }

    #[inline]
    pub fn get_attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    #[inline]
    pub fn get_diagnostics(&self) -> &HashMap<String, String> {
        &self.diagnostics
    }

    #[inline]
    pub fn get_files(&self) -> &HashSet<String> {
        &self.files
    }

    #[inline]
    pub fn get_version(&self) -> Version {
        self.version
    }

    #[inline]
    pub fn get_min_version(&self) -> Option<Version> {
        self.min_version
    }

    #[inline]
    pub fn is_compound_file(&self) -> bool {
        self.is_compound_file
    }

    #[inline]
    pub fn get_index_sort(&self) -> Option<&Sort> {
        self.index_sort.as_ref()
    }
}

#[derive(Debug)]
pub struct SegmentCommitInfo {
    pub(crate) info: SegmentInfo,

    /// Id that uniquely identifies this segment commit.
    pub(crate) id: Option<Id>,

    /// How many deleted docs in the segment
    pub(crate) del_count: u32,

    /// How many soft-deleted docs in the segment that are not also hard-deleted
    pub(crate) soft_del_count: u32,

    /// Generation number of the live docs file (None if there are no deletes yet):
    pub(crate) del_gen: Option<u64>,

    /// Normally 1+del_gen, unless an error was returned on the last attempt to write:
    pub(crate) next_write_del_gen: u64,

    /// Generation number of the FieldInfos (None if there are no updates)
    pub(crate) field_infos_gen: Option<u64>,

    /// Normally 1+field_infos_gen, unless an error was returned on the last attempt to write
    pub(crate) next_write_field_infos_gen: u64,

    /// Generation number of the DocValues (None if there are no updates)
    pub(crate) doc_values_gen: Option<u64>,

    /// Normally 1+doc_values_gen, unless an error was returned on the last attempt to write
    pub(crate) next_write_doc_values_gen: u64,

    /// Track the per-field DocValues update files
    doc_values_update_files: HashMap<i32, HashSet<String>>,

    field_infos_files: HashSet<String>,
}

impl SegmentCommitInfo {
    pub fn new(info: SegmentInfo, del_count: u32, soft_del_count: u32, del_gen: Option<u64>, field_infos_gen: Option<u64>, doc_values_gen: Option<u64>, id: Option<Id>) -> Self {
        let next_write_del_gen = del_gen.unwrap_or(0) + 1;
        let next_write_field_infos_gen = field_infos_gen.unwrap_or(0) + 1;
        let next_write_doc_values_gen = doc_values_gen.unwrap_or(0) + 1;

        Self {
            info,
            id,
            del_count,
            soft_del_count,
            del_gen,
            next_write_del_gen,
            field_infos_gen,
            next_write_field_infos_gen,
            doc_values_gen,
            next_write_doc_values_gen,
            doc_values_update_files: HashMap::new(),
            field_infos_files: HashSet::new(),
        }
    }

    #[inline]
    pub fn get_segment_info(&self) -> &SegmentInfo {
        &self.info
    }

    #[inline]
    pub fn get_id(&self) -> Option<Id> {
        self.id
    }

    #[inline]
    pub fn get_del_count(&self) -> u32 {
        self.del_count
    }

    #[inline]
    pub fn get_soft_del_count(&self) -> u32 {
        self.soft_del_count
    }

    #[inline]
    pub fn get_del_gen(&self) -> Option<u64> {
        self.del_gen
    }

    #[inline]
    pub fn get_next_write_del_gen(&self) -> u64 {
        self.next_write_del_gen
    }

    #[inline]
    pub fn get_field_infos_gen(&self) -> Option<u64> {
        self.field_infos_gen
    }

    #[inline]
    pub fn get_next_write_field_infos_gen(&self) -> u64 {
        self.next_write_field_infos_gen
    }

    #[inline]
    pub fn get_doc_values_gen(&self) -> Option<u64> {
        self.doc_values_gen
    }

    #[inline]
    pub fn get_next_write_doc_values_gen(&self) -> u64 {
        self.next_write_doc_values_gen
    }

    #[inline]
    pub fn get_doc_values_update_files(&self) -> &HashMap<i32, HashSet<String>> {
        &self.doc_values_update_files
    }

    #[inline]
    pub fn get_field_infos_files(&self) -> &HashSet<String> {
        &self.field_infos_files
    }

    #[inline]
    pub fn get_version(&self) -> Version {
        self.info.get_version()
    }

    #[inline]
    pub fn get_min_version(&self) -> Option<Version> {
        self.info.get_min_version()
    }

    pub fn set_field_infos_files(&mut self, files: HashSet<String>) {
        self.field_infos_files = files;
    }

    pub fn set_doc_values_update_files(&mut self, files: HashMap<i32, HashSet<String>>) {
        self.doc_values_update_files = files;
    }
}
