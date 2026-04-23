//! Version type for protocol compatibility checking.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum VersionError {
    #[error("invalid version format: expected major.minor.patch")]
    InvalidFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    pub const CURRENT: Version = Version {
        major: 0,
        minor: 1,
        patch: 0,
    };

    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub const fn is_compatible(&self, other: Version) -> bool {
        self.major == other.major
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl std::str::FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(VersionError::InvalidFormat);
        }
        let major: u16 = parts[0]
            .parse()
            .map_err(|_| VersionError::InvalidFormat)?;
        let minor: u16 = parts[1]
            .parse()
            .map_err(|_| VersionError::InvalidFormat)?;
        let patch: u16 = parts[2]
            .parse()
            .map_err(|_| VersionError::InvalidFormat)?;
        Ok(Version::new(major, minor, patch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_current() {
        assert_eq!(Version::CURRENT.major, 0);
        assert_eq!(Version::CURRENT.minor, 1);
        assert_eq!(Version::CURRENT.patch, 0);
    }

    #[test]
    fn test_version_compatible() {
        let v1 = Version::new(0, 1, 0);
        let v2 = Version::new(0, 1, 5);
        let v3 = Version::new(1, 0, 0);

        assert!(v1.is_compatible(v2));
        assert!(!v1.is_compatible(v3));
    }

    #[test]
    fn test_version_display() {
        let v = Version::new(1, 2, 3);
        assert_eq!(format!("{}", v), "1.2.3");
    }

    #[test]
    fn test_version_from_str() {
        let v: Version = "0.1.0".parse().unwrap();
        assert_eq!(v.major, 0);
        assert_eq!(v.minor, 1);
        assert_eq!(v.patch, 0);
    }
}
