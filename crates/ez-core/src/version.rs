use std::{fmt, num::ParseIntError, str::FromStr};

use semver::Version;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use thiserror::Error;

/// Human-readable product version compiled into this crate.
pub const PRODUCT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Version of the `EdgeZero` wire protocol.
///
/// Protocol compatibility evolves independently from the product release
/// version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProtocolVersion {
    major: u16,
    minor: u16,
}

impl ProtocolVersion {
    /// Creates a protocol version.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Returns the protocol major version.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the protocol minor version.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }
}

/// Initial development protocol version.
pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::new(0, 1);

/// Error returned when a protocol version cannot be parsed.
#[derive(Debug, Error)]
pub enum ProtocolVersionParseError {
    /// The protocol version does not have exactly two components.
    #[error("protocol version must use `<major>.<minor>`")]
    InvalidFormat,

    /// A protocol component is not an unsigned integer.
    #[error("invalid protocol version component: {0}")]
    InvalidComponent(#[from] ParseIntError),
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for ProtocolVersion {
    type Err = ProtocolVersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut components = value.split('.');

        let Some(major) = components.next() else {
            return Err(ProtocolVersionParseError::InvalidFormat);
        };

        let Some(minor) = components.next() else {
            return Err(ProtocolVersionParseError::InvalidFormat);
        };

        if components.next().is_some() || major.is_empty() || minor.is_empty() {
            return Err(ProtocolVersionParseError::InvalidFormat);
        }

        Ok(Self::new(major.parse()?, minor.parse()?))
    }
}

impl Serialize for ProtocolVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ProtocolVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

/// Semantic version of an `EdgeZero` product release.
///
/// This type is distinct from [`ProtocolVersion`]. Product releases may occur
/// without changing any interoperability protocol.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProductVersion(Version);

impl ProductVersion {
    /// Creates a product version from a semantic version.
    #[must_use]
    pub const fn from_semver(version: Version) -> Self {
        Self(version)
    }

    /// Returns the semantic version representation.
    #[must_use]
    pub const fn as_semver(&self) -> &Version {
        &self.0
    }
}

impl fmt::Display for ProductVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for ProductVersion {
    type Err = semver::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(Version::parse(value)?))
    }
}

impl Serialize for ProductVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ProductVersion {
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
    fn protocol_version_round_trips() {
        let version = ProtocolVersion::new(2, 7);

        let encoded = version.to_string();
        let decoded: ProtocolVersion = encoded.parse().expect("version should parse");

        assert_eq!(decoded, version);
    }

    #[test]
    fn protocol_version_requires_two_components() {
        assert!("1".parse::<ProtocolVersion>().is_err());
        assert!("1.2.3".parse::<ProtocolVersion>().is_err());
    }

    #[test]
    fn protocol_version_serializes_as_string() {
        let json =
            serde_json::to_string(&ProtocolVersion::new(1, 4)).expect("version should serialize");

        assert_eq!(json, "\"1.4\"");
    }

    #[test]
    fn product_version_supports_prereleases() {
        let version: ProductVersion = "1.0.0-beta.3"
            .parse()
            .expect("semantic version should parse");

        assert_eq!(version.to_string(), "1.0.0-beta.3");
    }

    #[test]
    fn compiled_product_version_is_semver() {
        let version = PRODUCT_VERSION
            .parse::<ProductVersion>()
            .expect("Cargo package version must be semantic");

        assert_eq!(version.to_string(), "0.0.1");
    }
}
