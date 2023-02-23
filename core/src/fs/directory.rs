use {
    crate::io::Directory,
    async_trait::async_trait,
    log::error,
    std::{
        convert::AsRef,
        io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
        path::{Path, PathBuf},
        pin::Pin,
    },
    tokio::{
        fs::{create_dir_all, metadata, read_dir, remove_dir_all, remove_file, rename, OpenOptions},
        io::{AsyncRead, AsyncWrite},
    },
};

/// Implementation of a Lucene directory (database) that stores index files on te file system.
#[derive(Debug)]
pub struct FilesystemDirectory {
    path: PathBuf,
}

impl FilesystemDirectory {
    /// Returns the path of this directory.
    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Open a directory at the given path.
    ///
    /// This will return an error if the directory does not exist.
    pub async fn open<P: AsRef<Path>>(path: P) -> IoResult<Self> {
        let path = path.as_ref();
        let md = metadata(path).await?;
        if !md.is_dir() {
            return Err(IoError::new(IoErrorKind::Other, format!("{} is not a directory", path.display())));
        }

        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    /// Opens a directory at the given path, creating it if it does not exist.
    pub async fn open_or_create<P: AsRef<Path>>(path: P) -> IoResult<Self> {
        let path = path.as_ref();
        match metadata(path).await {
            Ok(md) => {
                if !md.is_dir() {
                    return Err(IoError::new(IoErrorKind::Other, format!("{} is not a directory", path.display())));
                }
                Ok(Self {
                    path: path.to_path_buf(),
                })
            }
            Err(e) => {
                if e.kind() == IoErrorKind::NotFound {
                    create_dir_all(path).await?;
                    Ok(Self {
                        path: path.to_path_buf(),
                    })
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Creates a new directory at the given path.
    ///
    /// If the directory exists, it is deleted and recreated.
    pub async fn create<P: AsRef<Path>>(path: P) -> IoResult<Self> {
        let path = path.as_ref();
        if path.exists() {
            remove_dir_all(path).await?;
        }
        create_dir_all(path).await?;
        Ok(Self {
            path: path.to_path_buf(),
        })
    }
}

const DEFAULT_CAPACITY: usize = 64;

#[async_trait(?Send)]
impl Directory for FilesystemDirectory {
    async fn read_dir(&self) -> IoResult<Vec<String>> {
        let mut result = Vec::with_capacity(DEFAULT_CAPACITY);

        let mut rd = read_dir(&self.path).await?;
        loop {
            let entry = rd.next_entry().await?;
            let Some(entry) = entry else { break };
            let md = entry.metadata().await?;

            // Only include files...
            if md.is_file() {
                // That we can decode as UTF-8...
                match entry.file_name().into_string() {
                    Ok(s) => {
                        // That aren't the '.' or '..' current dir or parent dir entries.
                        if &s != "." && &s != ".." {
                            result.push(s);
                        }
                    }
                    Err(e) => {
                        error!("Failed to decode file name as UTF-8: {e:?}");
                    }
                }
            }
        }

        Ok(result)
    }

    async fn create(&mut self, file_name: &str) -> IoResult<Pin<Box<dyn AsyncWrite>>> {
        let mut options = OpenOptions::new();
        options.write(true);
        options.truncate(true);
        options.create(true);
        let f = options.open(self.path.join(file_name)).await?;
        Ok(Box::pin(f))
    }

    async fn open(&mut self, file_name: &str) -> IoResult<Pin<Box<dyn AsyncRead>>> {
        let mut options = OpenOptions::new();
        options.read(true);
        let f = options.open(self.path.join(file_name)).await?;
        Ok(Box::pin(f))
    }

    async fn remove(&mut self, file_name: &str) -> IoResult<()> {
        remove_file(self.path.join(file_name)).await
    }

    async fn rename(&mut self, old_file_name: &str, new_file_name: &str) -> IoResult<()> {
        rename(self.path.join(old_file_name), self.path.join(new_file_name)).await
    }
}
