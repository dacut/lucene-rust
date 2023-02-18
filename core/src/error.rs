use {
    std::error::Error,
    std::fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug)]
pub enum LuceneError {
    CorruptIndex(String),
    InvalidVersionString(String),
    UnsupportedVersion(String),
}

impl Display for LuceneError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::CorruptIndex(message) => write!(f, "Corrupt index: {message}"),
            Self::InvalidVersionString(version) => write!(f, "Invalid version string: {version}"),
            Self::UnsupportedVersion(version) => write!(f, "Unsupported version: {version}"),
        }
    }
}

impl Error for LuceneError {}

pub type BoxError = Box<dyn Error + Send + Sync + 'static>;

pub type BoxResult<T> = Result<T, BoxError>;
