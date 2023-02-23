use {
    async_trait::async_trait,
    chrono::{DateTime, Utc},
    std::{fmt::Debug, io::Result as IoResult, pin::Pin, time::SystemTime},
    tokio::io::{AsyncRead, AsyncWrite},
};

/// A `Directory` is an abstraction for providing a file-like view of a Lucene index. A `Directory` contains only
/// files and no subdirectories.
///
/// This is not necessarily a filesystem directory. The underlying storage can be elsewhere, such as a database or
/// other networked resource, or in memory.
#[async_trait(?Send)]
pub trait Directory: Debug {
    /// Returns a listing of the files in this directory.
    async fn read_dir(&self) -> IoResult<Vec<String>>;

    /// Creates a new file for writing.
    ///
    /// If the file already exists, it will be overwritten.
    async fn create(&mut self, file_name: &str) -> IoResult<Pin<Box<dyn AsyncWrite>>>;

    /// Opens an existing file for reading.
    async fn open(&mut self, file_name: &str) -> IoResult<Pin<Box<dyn AsyncRead>>>;

    /// Removes the file with the given name.
    async fn remove(&mut self, file_name: &str) -> IoResult<()>;

    /// Renames the file with the given name to the new name.
    ///
    /// This is not guaranteed to be atomic. In particular, the [Directory::read_dir] method may return both the old
    /// and new names during the rename.
    async fn rename(&mut self, old_file_name: &str, new_file_name: &str) -> IoResult<()>;
}

/// A file timestamp, which can be either a [SystemTime] or [DateTime].
///
/// Local files typically have the [SystemTime] representation, while remote files will have a [DateTime]
/// representation. We avoid unnecessary conversions by storing the timestamp in the most efficient format.
#[derive(Clone, Copy, Debug)]
pub enum FileTimestamp {
    /// Timestamp represented as a [SystemTime].
    SystemTime(SystemTime),

    /// Timestamp represented as a [DateTime] in the UTC time zone.
    DateTime(DateTime<Utc>),
}

impl From<FileTimestamp> for DateTime<Utc> {
    fn from(timestamp: FileTimestamp) -> Self {
        match timestamp {
            FileTimestamp::SystemTime(system_time) => DateTime::from(system_time),
            FileTimestamp::DateTime(date_time) => date_time,
        }
    }
}

impl PartialEq for FileTimestamp {
    fn eq(&self, other: &Self) -> bool {
        let self_dt: DateTime<Utc> = (*self).into();
        let other_dt: DateTime<Utc> = (*other).into();
        self_dt == other_dt
    }
}

impl Eq for FileTimestamp {}

impl PartialOrd for FileTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_dt: DateTime<Utc> = (*self).into();
        let other_dt: DateTime<Utc> = (*other).into();
        self_dt.partial_cmp(&other_dt)
    }
}

impl Ord for FileTimestamp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_dt: DateTime<Utc> = (*self).into();
        let other_dt: DateTime<Utc> = (*other).into();
        self_dt.cmp(&other_dt)
    }
}
