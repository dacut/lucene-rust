use {
    crate::{io::EncodingReadExt, BoxError, LuceneError},
    log::error,
    std::{
        fmt::{Display, Formatter, Result as FmtResult},
        str::FromStr,
    },
    tokio::io::{AsyncRead, AsyncReadExt},
};

/// Version numbers of Lucene. This is used to ensure compatibility across different releases.
/// This is kept in-sync with the Java version of Lucene.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Version {
    major: u8,
    minor: u8,
    bugfix: u8,
    prerelease: u8,
}

impl Version {
    /// Create a new `Version` structure from the given major, minor, and bugfix numbers.
    pub const fn new(major: u8, minor: u8, bugfix: u8) -> Self {
        Self {
            major,
            minor,
            bugfix,
            prerelease: 0,
        }
    }

    /// Returns the major version number.
    #[inline]
    pub fn major(&self) -> u8 {
        self.major
    }

    /// Returns the minor version number.
    #[inline]
    pub fn minor(&self) -> u8 {
        self.minor
    }

    /// Returns the bugfix version number.
    #[inline]
    pub fn bugfix(&self) -> u8 {
        self.bugfix
    }

    /// Returns the prerelease version number.
    #[inline]
    pub fn prerelease(&self) -> u8 {
        self.prerelease
    }

    /// Read a version from a stream as three vi32 values.
    pub async fn read_from_vi32<R: AsyncRead + Unpin>(r: &mut R) -> Result<Self, BoxError> {
        let major = r.read_vi32().await?;
        let minor = r.read_vi32().await?;
        let bugfix = r.read_vi32().await?;

        if major < 0
            || major > u8::MAX as i32
            || minor < 0
            || minor > u8::MAX as i32
            || bugfix < 0
            || bugfix > u8::MAX as i32
        {
            Err(LuceneError::InvalidVersionStreamData(major, minor, bugfix).into())
        } else {
            Ok(Self {
                major: major as u8,
                minor: minor as u8,
                bugfix: bugfix as u8,
                prerelease: 0,
            })
        }
    }

    /// Read a version from a stream as three i32 little-endian values.
    pub async fn read_from_i32_le<R: AsyncRead + Unpin>(r: &mut R) -> Result<Self, BoxError> {
        let major = r.read_i32_le().await?;
        let minor = r.read_i32_le().await?;
        let bugfix = r.read_i32_le().await?;

        if major < 0
            || major > u8::MAX as i32
            || minor < 0
            || minor > u8::MAX as i32
            || bugfix < 0
            || bugfix > u8::MAX as i32
        {
            Err(LuceneError::InvalidVersionStreamData(major, minor, bugfix).into())
        } else {
            Ok(Self {
                major: major as u8,
                minor: minor as u8,
                bugfix: bugfix as u8,
                prerelease: 0,
            })
        }
    }
}

impl From<Version> for u32 {
    fn from(version: Version) -> Self {
        (version.major as u32) << 24
            | (version.minor as u32) << 16
            | (version.bugfix as u32) << 8
            | version.prerelease as u32
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        if self.prerelease > 0 {
            write!(f, "{}.{}.{}.{}", self.major, self.minor, self.bugfix, self.prerelease)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.bugfix)
        }
    }
}

impl FromStr for Version {
    type Err = LuceneError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');
        let major = parts.next().ok_or(LuceneError::InvalidVersionString(s.to_string()))?;
        let minor = parts.next().ok_or(LuceneError::InvalidVersionString(s.to_string()))?;
        let bugfix = parts.next().ok_or(LuceneError::InvalidVersionString(s.to_string()))?;
        let prerelease = parts.next();

        let major = major.parse::<u8>().map_err(|_| LuceneError::InvalidVersionString(s.to_string()))?;
        let minor = minor.parse::<u8>().map_err(|_| LuceneError::InvalidVersionString(s.to_string()))?;
        let bugfix = bugfix.parse::<u8>().map_err(|_| LuceneError::InvalidVersionString(s.to_string()))?;
        let prerelease = if let Some(prerelease) = prerelease {
            prerelease.parse::<u8>().map_err(|_| LuceneError::InvalidVersionString(s.to_string()))?
        } else {
            0
        };

        if prerelease > 2 {
            error!("Invalid prerelease {prerelease} in version string {s}");
            return Err(LuceneError::InvalidVersionString(s.to_string()));
        }

        if prerelease != 0 && (minor != 0 || bugfix != 0) {
            error!("Prerelease cannot be non-zero when minor or bugfix is non-zero in version string {s}");
            return Err(LuceneError::InvalidVersionString(s.to_string()));
        }

        Ok(Self {
            major,
            minor,
            bugfix,
            prerelease,
        })
    }
}

/// Match settings and bugs in Lucene's 9.0.0 release.
pub const LUCENE_9_0_0: Version = Version::new(9, 0, 0);

/// Match settings and bugs in Lucene's 9.1.0 release.
pub const LUCENE_9_1_0: Version = Version::new(9, 1, 0);

/// Match settings and bugs in Lucene's 9.2.0 release.
pub const LUCENE_9_2_0: Version = Version::new(9, 2, 0);

/// Match settings and bugs in Lucene's 9.3.0 release.
pub const LUCENE_9_3_0: Version = Version::new(9, 3, 0);

/// Match settings and bugs in Lucene's 9.4.0 release.
pub const LUCENE_9_4_0: Version = Version::new(9, 4, 0);

/// Match settings and bugs in Lucene's 9.4.1 release.
pub const LUCENE_9_4_1: Version = Version::new(9, 4, 1);

/// Match settings and bugs in Lucene's 9.4.2 release.
pub const LUCENE_9_4_2: Version = Version::new(9, 4, 2);

/// Match settings and bugs in Lucene's 9.5.0 release.
pub const LUCENE_9_5_0: Version = Version::new(9, 5, 0);

/// The current version of Lucene.
pub const LATEST: Version = LUCENE_9_5_0;

/// The minimul supported version of an index.
pub const MIN_SUPPORTED: Version = Version::new(8, 0, 0);
