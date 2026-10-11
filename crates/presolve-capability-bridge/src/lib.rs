#![forbid(unsafe_code)]

//! Container/network transport projection for Presolve capability semantics.
//!
//! Capability contracts remain the canonical semantic layer. This crate defines
//! the initial HTTP/JSON bridge used when a workload cannot receive native WIT
//! host imports, such as an OCI container.

mod client;
mod protocol;

pub use client::{CapabilityBridgeClient, CapabilityBridgeClientError};
pub use protocol::{
    CAPABILITY_BRIDGE_PATH, CAPABILITY_BRIDGE_PROTOCOL, CAPABILITY_ENDPOINT_ENV,
    CAPABILITY_PROTOCOL_ENV, CAPABILITY_TOKEN_ENV, CapabilityBridgeRequest,
    CapabilityBridgeResponse,
};
