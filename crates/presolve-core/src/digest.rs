use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use sha2::{Digest as _, Sha256};
use thiserror::Error;

/// Number of bytes in a SHA-256 digest.
pub const SHA256_LENGTH: usize = 32;

/// Prefix used by textual `Presolve` artifact digests.
pub const SHA256_PREFIX: &str = "sha256:";

/// SHA-256 content digest.
///
/// Artifact identity in `Presolve` is content-addressed. Two artifacts with
/// identical bytes therefore have identical digests regardless of where they
/// were built, stored, or deployed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArtifactDigest([u8; SHA256_LENGTH]);

impl ArtifactDigest {
    /// Computes the digest of the supplied content.
    #[must_use]
    pub fn from_content(content: &[u8]) -> Self {
        let digest = Sha256::digest(content);
        let mut bytes = [0_u8; SHA256_LENGTH];

        bytes.copy_from_slice(&digest);

        Self(bytes)
    }

    /// Constructs a digest from an already-computed SHA-256 value.
    #[must_use]
    pub const fn from_sha256_bytes(bytes: [u8; SHA256_LENGTH]) -> Self {
        Self(bytes)
    }

    /// Returns the raw SHA-256 bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; SHA256_LENGTH] {
        &self.0
    }
}

/// Error returned when a textual artifact digest cannot be parsed.
#[derive(Debug, Error)]
pub enum ArtifactDigestParseError {
    /// The algorithm prefix is missing or unsupported.
    #[error("artifact digest must begin with `{SHA256_PREFIX}`")]
    InvalidPrefix,

    /// The hexadecimal payload is malformed.
    #[error("artifact digest contains invalid hexadecimal data: {0}")]
    InvalidHex(#[from] hex::FromHexError),

    /// The digest is not the correct number of bytes for SHA-256.
    #[error("SHA-256 digest must contain {expected} bytes, found {actual}")]
    InvalidLength { expected: usize, actual: usize },
}

impl fmt::Display for ArtifactDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{SHA256_PREFIX}{}", hex::encode(self.0))
    }
}

impl FromStr for ArtifactDigest {
    type Err = ArtifactDigestParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let encoded = value
            .strip_prefix(SHA256_PREFIX)
            .ok_or(ArtifactDigestParseError::InvalidPrefix)?;

        let decoded = hex::decode(encoded)?;

        let bytes: [u8; SHA256_LENGTH] = decoded.try_into().map_err(|value: Vec<u8>| {
            ArtifactDigestParseError::InvalidLength {
                expected: SHA256_LENGTH,
                actual: value.len(),
            }
        })?;

        Ok(Self(bytes))
    }
}

impl Serialize for ArtifactDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ArtifactDigest {
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

    const PRESOLVE_SHA256: &str =
        "sha256:c563861335134ec35db456c15c8ad8ea31125251ee6c4292cac41ae1a2e0f73c";

    #[test]
    fn digest_matches_known_sha256() {
        let digest = ArtifactDigest::from_content(b"presolve");

        assert_eq!(digest.to_string(), PRESOLVE_SHA256);
    }

    #[test]
    fn digest_round_trips_through_text() {
        let digest = ArtifactDigest::from_content(b"portable application");

        let encoded = digest.to_string();
        let decoded: ArtifactDigest = encoded.parse().expect("digest should parse");

        assert_eq!(decoded, digest);
    }

    #[test]
    fn digest_round_trips_through_json() {
        let digest = ArtifactDigest::from_content(b"application artifact");

        let json = serde_json::to_string(&digest).expect("digest should serialize");
        let decoded: ArtifactDigest =
            serde_json::from_str(&json).expect("digest should deserialize");

        assert_eq!(decoded, digest);
    }

    #[test]
    fn digest_rejects_unknown_algorithm() {
        let result = "md5:c563861335134ec35db456c15c8ad8ea31125251ee6c4292cac41ae1a2e0f73c"
            .parse::<ArtifactDigest>();

        assert!(matches!(
            result,
            Err(ArtifactDigestParseError::InvalidPrefix)
        ));
    }

    #[test]
    fn digest_rejects_wrong_length() {
        let result = "sha256:abcd".parse::<ArtifactDigest>();

        assert!(matches!(
            result,
            Err(ArtifactDigestParseError::InvalidLength { .. })
        ));
    }
}
