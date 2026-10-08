use std::{fmt, num::ParseIntError, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

/// Version of the `Presolve` Application Contract schema.
///
/// Contract versions evolve independently from product releases and from the
/// `Presolve` wire protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContractVersion {
    major: u16,
    minor: u16,
}

impl ContractVersion {
    /// Creates an Application Contract version.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Returns the contract major version.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the contract minor version.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }
}

/// Application Contract version implemented by this release.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion::new(0, 1);

/// Error returned when a contract version cannot be parsed.
#[derive(Debug)]
pub enum ContractVersionParseError {
    /// The version is not in `<major>.<minor>` form.
    InvalidFormat,

    /// One of the version components is not an unsigned integer.
    InvalidComponent(ParseIntError),
}

impl fmt::Display for ContractVersionParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => {
                write!(formatter, "contract version must use `<major>.<minor>`")
            }
            Self::InvalidComponent(error) => {
                write!(formatter, "invalid contract version component: {error}")
            }
        }
    }
}

impl std::error::Error for ContractVersionParseError {}

impl From<ParseIntError> for ContractVersionParseError {
    fn from(error: ParseIntError) -> Self {
        Self::InvalidComponent(error)
    }
}

impl fmt::Display for ContractVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for ContractVersion {
    type Err = ContractVersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut components = value.split('.');

        let Some(major) = components.next() else {
            return Err(ContractVersionParseError::InvalidFormat);
        };

        let Some(minor) = components.next() else {
            return Err(ContractVersionParseError::InvalidFormat);
        };

        if components.next().is_some() || major.is_empty() || minor.is_empty() {
            return Err(ContractVersionParseError::InvalidFormat);
        }

        Ok(Self::new(major.parse()?, minor.parse()?))
    }
}

impl Serialize for ContractVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ContractVersion {
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
    fn contract_version_round_trips() {
        let version = ContractVersion::new(0, 1);

        let encoded = version.to_string();
        let decoded: ContractVersion = encoded.parse().expect("contract version should parse");

        assert_eq!(decoded, version);
    }

    #[test]
    fn contract_version_rejects_three_components() {
        assert!("0.1.2".parse::<ContractVersion>().is_err());
    }

    #[test]
    fn contract_version_serializes_as_string() {
        let json = serde_json::to_string(&ContractVersion::new(0, 1))
            .expect("contract version should serialize");

        assert_eq!(json, "\"0.1\"");
    }
}
