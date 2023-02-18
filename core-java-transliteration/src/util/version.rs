use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

/// Match settings and bugs in Lucene's 9.0.0 release.
pub const LUCENE_9_0_0: Version = Version::new(9, 0, 0, 0);

/// Match settings and bugs in Lucene's 9.1.0 release.
pub const LUCENE_9_1_0: Version = Version::new(9, 1, 0, 0);

/// Match settings and bugs in Lucene's 9.2.0 release.
pub const LUCENE_9_2_0: Version = Version::new(9, 2, 0, 0);

/// Match settings and bugs in Lucene's 9.3.0 release.
pub const LUCENE_9_3_0: Version = Version::new(9, 3, 0, 0);

/// Match settings and bugs in Lucene's 9.4.0 release.
pub const LUCENE_9_4_0: Version = Version::new(9, 4, 0, 0);

/// Match settings and bugs in Lucene's 9.4.1 release.
pub const LUCENE_9_4_1: Version = Version::new(9, 4, 1, 0);

/// Match settings and bugs in Lucene's 9.4.2 release.
pub const LUCENE_9_4_2: Version = Version::new(9, 4, 2, 0);

/// Match settings and bugs in Lucene's 10.0.0 release.
pub const LUCENE_10_0_0: Version = Version::new(10, 0, 0, 0);

/// # Warning
/// If you use this setting, and then upgrade to a newer release of Lucene, sizable
/// changes may happen. If backwards compatibility is important then you should instead explicitly
/// specify an actual version.
///
/// If you use this constant then you may need to _re-index all of your documents_ when
/// upgrading Lucene, as the way text is indexed may have changed. Additionally, you may need to
/// _re-test your entire application_ to ensure it behaves as expected, as some defaults may
/// have changed and may break functionality in your application.
pub const LATEST: Version = LUCENE_10_0_0;

/// Constant for the minimal supported major version of an index. This version is defined by the
/// version that initially created the index.
pub const MIN_SUPPORTED_MAJOR: u8 = LATEST.get_major() - 1;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Version {
    /// Major version, the difference between stable and trunk
    major: u8,

    ///mMinor version, incremented within the stable branch
    minor: u8,

    /// Bugfix number, incremented on release branches
    bugfix: u8,

    /// Prerelease version, currently 0 (alpha), 1 (beta), or 2 (final)
    prerelease: u8,
}

impl Version {
    pub const fn new(major: u8, minor: u8, bugfix: u8, prerelease: u8) -> Self {
        if prerelease > 2 {
            panic!("Invalid prerelease version: {}", prerelease);
        }

        if prerelease != 0 && (minor != 0 || bugfix != 0) {
            panic!("Prerelease version only supported with major release (got prerelease: {prerelease}, minor: {minor}, bugfix: {bugfix})");
        }

        Self {
            major,
            minor,
            bugfix,
            prerelease,
        }
    }

    #[inline]
    pub const fn get_major(&self) -> u8 {
        self.major
    }

    #[inline]
    pub const fn get_minor(&self) -> u8 {
        self.minor
    }

    #[inline]
    pub const fn get_bugfix(&self) -> u8 {
        self.bugfix
    }

    #[inline]
    pub const fn get_prerelease(&self) -> u8 {
        self.prerelease
    }

    fn parse_lucene_version_suffix(suffix: &str) -> Result<Self, VersionParseError> {
        if suffix.len() == 0 {
            return Err(VersionParseError::from_lucene_suffix(suffix));
        }
    
        let parts = suffix.split('_').collect::<Vec<_>>();
        if parts.len() == 1 {
            // Should be a version in the form LUCENE_XY -> X.Y.0.0
            let maj_min = parts[0];
            if maj_min.len() != 2 {
                return Err(VersionParseError::from_lucene_suffix(suffix));
            }

            let major = maj_min[0..1].parse::<u8>().map_err(|_| VersionParseError::from_lucene_suffix(suffix))?;

            let minor = maj_min[1..2].parse::<u8>().map_err(|_| VersionParseError::from_lucene_suffix(suffix))?;

            return Ok(Self::new(major, minor, 0, 0));
        }

        if parts.len() > 3 {
            return Err(VersionParseError::from_lucene_suffix(suffix));
        }

        let major = parts[0].parse::<u8>().map_err(|_| VersionParseError::from_lucene_suffix(suffix))?;

        let minor = parts[1].parse::<u8>().map_err(|_| VersionParseError::from_lucene_suffix(suffix))?;

        let bugfix = if parts.len() > 2 {
            parts[2].parse::<u8>().map_err(|_| VersionParseError::from_lucene_suffix(suffix))?
        } else {
            0
        };

        Ok(Self::new(major, minor, bugfix, 0))
    }
}

impl FromStr for Version {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, VersionParseError> {
        if s == "LATEST" {
            return Ok(LATEST);
        }

        // Check for LUCENE_* version strings.
        if let Some(suffix) = s.strip_prefix("LUCENE_") {
            return Version::parse_lucene_version_suffix(suffix);
        }

        let parts = s.split('.').collect::<Vec<_>>();
        if parts.len() < 3 || parts.len() > 4 {
            return Err(VersionParseError {
                version: s.to_string(),
            });
        }

        let major = parts[0].parse::<u8>().map_err(|_| VersionParseError {
            version: s.to_string(),
        })?;
        let minor = parts[1].parse::<u8>().map_err(|_| VersionParseError {
            version: s.to_string(),
        })?;
        let bugfix = parts[2].parse::<u8>().map_err(|_| VersionParseError {
            version: s.to_string(),
        })?;
        let prerelease = if parts.len() == 4 {
            parts[3].parse::<u8>().map_err(|_| VersionParseError {
                version: s.to_string(),
            })?
        } else {
            0
        };

        if prerelease > 2 {
            return Err(VersionParseError {
                version: s.to_string(),
            });
        }

        Ok(Self::new(major, minor, bugfix, prerelease))
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.prerelease == 0 {
            write!(f, "{}.{}.{}", self.major, self.minor, self.bugfix)
        } else {
            write!(f, "{}.{}.{}.{}", self.major, self.minor, self.bugfix, self.prerelease)
        }
    }
}

#[derive(Debug)]
pub struct VersionParseError {
    pub version: String,
}

impl VersionParseError {
    fn from_lucene_suffix(suffix: &str) -> Self {
        Self {
            version: format!("LUCENE_{}", suffix),
        }
    }
}

impl Display for VersionParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Version is not in form major.minor.bugfix[.prerelease] format:: {}", self.version)
    }
}

impl Error for VersionParseError {}
