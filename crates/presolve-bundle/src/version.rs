use std::{fmt, num::ParseIntError, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use thiserror::Error;

/// Version of the `.presolved` bundle format.
///
/// Bundle format compatibility evolves independently from the product,
/// Application Contract, and wire protocol versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BundleVersion {
    major: u16,
    minor: u16,
}

impl BundleVersion {
    /// Creates a bundle format version.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Returns the major bundle format version.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the minor bundle format version.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }
}

/// Initial `.presolved` bundle format.
pub const BUNDLE_VERSION: BundleVersion = BundleVersion::new(0, 1);

/// Error returned when a bundle format version cannot be parsed.
#[derive(Debug, Error)]
pub enum BundleVersionParseError {
    /// The version does not contain exactly two numeric components.
    #[error("bundle version must use `<major>.<minor>`")]
    InvalidFormat,

    /// One version component is not an unsigned integer.
    #[error("invalid bundle version component: {0}")]
    InvalidComponent(#[from] ParseIntError),
}

impl fmt::Display for BundleVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for BundleVersion {
    type Err = BundleVersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut components = value.split('.');

        let Some(major) = components.next() else {
            return Err(BundleVersionParseError::InvalidFormat);
        };

        let Some(minor) = components.next() else {
            return Err(BundleVersionParseError::InvalidFormat);
        };

        if components.next().is_some() || major.is_empty() || minor.is_empty() {
            return Err(BundleVersionParseError::InvalidFormat);
        }

        Ok(Self::new(major.parse()?, minor.parse()?))
    }
}

impl Serialize for BundleVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for BundleVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_round_trips() {
        let encoded = BUNDLE_VERSION.to_string();
        let decoded: BundleVersion = encoded.parse().expect("version should parse");

        assert_eq!(decoded, BUNDLE_VERSION);
    }

    #[test]
    fn version_serializes_as_string() {
        let json = serde_json::to_string(&BUNDLE_VERSION).expect("version should serialize");

        assert_eq!(json, "\"0.1\"");
    }
}
