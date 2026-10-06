#![forbid(unsafe_code)]

//! Fundamental types and contracts shared across `EdgeZero`.
//!
//! `edgezero-core` contains the lowest-level vocabulary used by the rest of the
//! project. It deliberately contains no runtime, deployment, provider, or
//! control-plane implementation.

mod diagnostic;
mod digest;
mod id;
mod version;

pub use diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticCodeError, DiagnosticSeverity, MAX_DIAGNOSTIC_CODE,
    MIN_DIAGNOSTIC_CODE,
};
pub use digest::{ArtifactDigest, ArtifactDigestParseError, SHA256_LENGTH, SHA256_PREFIX};
pub use id::{ApplicationId, EnvironmentId, IdParseError, ProviderId};
pub use version::{
    PRODUCT_VERSION, PROTOCOL_VERSION, ProductVersion, ProtocolVersion, ProtocolVersionParseError,
};

/// Human-readable product name.
pub const PRODUCT_NAME: &str = "EdgeZero";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_name_is_stable() {
        assert_eq!(PRODUCT_NAME, "EdgeZero");
    }

    #[test]
    fn initial_protocol_version_is_zero_one() {
        assert_eq!(PROTOCOL_VERSION, ProtocolVersion::new(0, 1));
    }

    #[test]
    fn package_version_is_zero_zero_one() {
        assert_eq!(PRODUCT_VERSION, "0.0.1");
    }
}
