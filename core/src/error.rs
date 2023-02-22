use {
    crate::{index::MAX_DOCS, io::CODEC_MAGIC},
    std::{
        error::Error,
        fmt::{Display, Formatter, Result as FmtResult},
    },
};

#[derive(Debug)]
pub enum LuceneError {
    CorruptIndex(String),
    IncorrectCodecName(Vec<u8>, String),
    InvalidCodecName(String),
    InvalidCodecHeaderMagic([u8; 4]),
    InvalidSortField(String),
    InvalidVersionString(String),
    InvalidVersionStreamData(i32, i32, i32),
    MissingSortDirectives,
    TooManyDocs(u64),
    UnknownCodec(String),
    UnknownSortFieldProvider(String),
    UnknownSortFieldType(String),
    UnsupportedCodecVersion(String, u32, u32, u32),
    UnsupportedLuceneVersion(String),
}

impl Display for LuceneError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::CorruptIndex(message) => write!(f, "Corrupt index: {message}"),
            Self::IncorrectCodecName(actual, expected) => {
                if let Ok(actual) = String::from_utf8(actual.clone()) {
                    write!(f, "Incorrect codec name: got {actual:?}, expected {expected:?}")
                } else {
                    write!(f, "Incorrect codec name: got {actual:#x?}, expected {expected:?}")
                }
            }
            Self::InvalidCodecHeaderMagic(actual) => {
                write!(f, "Invalid codec header: got {actual:#x?}, expected {CODEC_MAGIC:#x?}")
            }
            Self::InvalidCodecName(codec_name) => {
                write!(f, "Invalid codec name: {codec_name:?} is not a valid ASCII string under 128 bytes")
            }
            Self::InvalidSortField(message) => write!(f, "Invalid sort field: {message}"),
            Self::InvalidVersionString(version) => write!(f, "Invalid version string: {version}"),
            Self::InvalidVersionStreamData(major, minor, bugfix) => {
                write!(f, "Invalid version data in stream: {major}.{minor}.{bugfix}")
            }
            Self::MissingSortDirectives => write!(f, "Missing sort directives"),
            Self::TooManyDocs(actual) => write!(f, "Too many docs: {actual} exceeds MAX_DOCS value of {MAX_DOCS}"),
            Self::UnknownCodec(name) => write!(f, "Unknown codec: {name}"),
            Self::UnknownSortFieldProvider(name) => write!(f, "Unknown sort directive provider: {name}"),
            Self::UnknownSortFieldType(name) => write!(f, "Unknown sort field type: {name}"),
            Self::UnsupportedCodecVersion(name, actual, min, max) => write!(
                f,
                "Codec version mismatch: {name} version {actual} is not supported (must be between {min} and {max}"
            ),
            Self::UnsupportedLuceneVersion(version) => write!(f, "Unsupported Lucene version: {version}"),
        }
    }
}

impl Error for LuceneError {}

pub type BoxError = Box<dyn Error + Send + Sync + 'static>;

pub type BoxResult<T> = Result<T, BoxError>;
