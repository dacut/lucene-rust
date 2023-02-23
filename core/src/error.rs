use {
    crate::{codec::CODEC_MAGIC, index::MAX_DOCS},
    std::{
        error::Error,
        fmt::{Display, Formatter, Result as FmtResult},
    },
};

/// Errors that can occur in Lucene.
#[derive(Debug)]
pub enum LuceneError {
    /// The index is corrupt.
    CorruptIndex(String),

    /// The codec name in the index is incorrect and was expected to be something else.
    IncorrectCodecName(Vec<u8> /* name */, String /* expected */),

    /// A codec name was invalid (not a valid ASCII string under 128 bytes).
    InvalidCodecName(String),

    /// The codec header magic bytes were incorrect.
    InvalidCodecHeaderMagic([u8; 4]),

    /// A sort field specification was invalid.
    InvalidSortField(String /* message */),

    /// A version string was invalid.
    InvalidVersionString(String),

    /// A version number in a stream was invalid.
    InvalidVersionStreamData(i32, i32, i32),

    /// A sort field was missing.
    MissingSortDirectives,

    /// Too many documents (beyond [crate::index::MAX_DOCS]) were encountered.
    TooManyDocs(u64 /* actual */),

    /// A codec was unknown.
    UnknownCodec(String /* requested */),

    /// A sort field provider was unknown.
    UnknownSortFieldProvider(String),

    /// A sort field type was unknown.
    UnknownSortFieldType(String),

    /// A given codec version is unsupported.
    UnsupportedCodecVersion(String, u32, u32, u32),

    /// The Lucene version of the data is unsupported.
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

/// A type alias for any kind of error. The error is boxed and must be `Send`, `Sync`, and `'static`.
pub type BoxError = Box<dyn Error + Send + Sync + 'static>;

/// A type alias for a `Result` with a [BoxError].
pub type BoxResult<T> = Result<T, BoxError>;
