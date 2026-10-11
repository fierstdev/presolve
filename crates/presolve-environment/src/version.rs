use std::{fmt, num::ParseIntError, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

/// Version of the Presolve Environment Specification schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EnvironmentSpecificationVersion {
    major: u16,
    minor: u16,
}

impl EnvironmentSpecificationVersion {
    /// Creates an Environment Specification schema version.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Returns the schema major version.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the schema minor version.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }
}

/// Environment Specification version implemented by this release.
pub const ENVIRONMENT_SPECIFICATION_VERSION: EnvironmentSpecificationVersion =
    EnvironmentSpecificationVersion::new(0, 1);

/// Error returned when an Environment Specification version cannot be parsed.
#[derive(Debug)]
pub enum EnvironmentSpecificationVersionParseError {
    /// The version is not in `<major>.<minor>` form.
    InvalidFormat,

    /// One of the version components is not an unsigned integer.
    InvalidComponent(ParseIntError),
}

impl fmt::Display for EnvironmentSpecificationVersionParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => {
                formatter.write_str("environment specification version must use `<major>.<minor>`")
            }
            Self::InvalidComponent(error) => {
                write!(
                    formatter,
                    "invalid environment specification version component: {error}"
                )
            }
        }
    }
}

impl std::error::Error for EnvironmentSpecificationVersionParseError {}

impl From<ParseIntError> for EnvironmentSpecificationVersionParseError {
    fn from(error: ParseIntError) -> Self {
        Self::InvalidComponent(error)
    }
}

impl fmt::Display for EnvironmentSpecificationVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for EnvironmentSpecificationVersion {
    type Err = EnvironmentSpecificationVersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut components = value.split('.');

        let Some(major) = components.next() else {
            return Err(EnvironmentSpecificationVersionParseError::InvalidFormat);
        };

        let Some(minor) = components.next() else {
            return Err(EnvironmentSpecificationVersionParseError::InvalidFormat);
        };

        if components.next().is_some() || major.is_empty() || minor.is_empty() {
            return Err(EnvironmentSpecificationVersionParseError::InvalidFormat);
        }

        Ok(Self::new(major.parse()?, minor.parse()?))
    }
}

impl Serialize for EnvironmentSpecificationVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for EnvironmentSpecificationVersion {
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
        let version = EnvironmentSpecificationVersion::new(0, 1);
        let encoded = version.to_string();
        let decoded: EnvironmentSpecificationVersion =
            encoded.parse().expect("version should parse");

        assert_eq!(decoded, version);
    }

    #[test]
    fn version_rejects_three_components() {
        assert!("0.1.2".parse::<EnvironmentSpecificationVersion>().is_err());
    }

    #[test]
    fn version_serializes_as_string() {
        let json = serde_json::to_string(&ENVIRONMENT_SPECIFICATION_VERSION)
            .expect("version should serialize");

        assert_eq!(json, "\"0.1\"");
    }
}
