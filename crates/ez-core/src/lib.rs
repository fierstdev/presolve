#![forbid(unsafe_code)]

//! Fundamental types and constants shared across `EdgeZero`.
//!
//! This crate must remain lightweight. Higher-level `EdgeZero` crates may depend
//! on `edgezero-core`, but `edgezero-core` must not depend on runtime,
//! deployment, provider, or control-plane implementations.

/// Human-readable product name.
pub const PRODUCT_NAME: &str = "EdgeZero";

/// Version of the protocol understood by this `EdgeZero` build.
///
/// Product releases and protocol compatibility are deliberately separate.
/// `EdgeZero` may release new product versions without changing its wire or
/// application protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}

impl ProtocolVersion {
    /// Creates a protocol version.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

/// Initial development protocol.
///
/// This is not the `EdgeZero` Application Contract version. Protocol, contract,
/// bundle, capability, and product versions will each evolve independently.
pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::new(0, 1);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_name_is_stable() {
        assert_eq!(PRODUCT_NAME, "EdgeZero");
    }

    #[test]
    fn initial_protocol_version_is_zero_one() {
        assert_eq!(PROTOCOL_VERSION, ProtocolVersion { major: 0, minor: 1 });
    }
}
