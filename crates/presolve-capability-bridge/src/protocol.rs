use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Initial Presolve container capability transport protocol.
pub const CAPABILITY_BRIDGE_PROTOCOL: &str = "presolve-http-json/0.1";

/// HTTP endpoint path for capability calls.
pub const CAPABILITY_BRIDGE_PATH: &str = "/v1/call";

/// Container environment variable containing the bridge endpoint.
pub const CAPABILITY_ENDPOINT_ENV: &str = "PRESOLVE_CAPABILITY_ENDPOINT";

/// Container environment variable containing the per-deployment bearer token.
pub const CAPABILITY_TOKEN_ENV: &str = "PRESOLVE_CAPABILITY_TOKEN";

/// Container environment variable identifying the bridge protocol.
pub const CAPABILITY_PROTOCOL_ENV: &str = "PRESOLVE_CAPABILITY_PROTOCOL";

/// One transport-level invocation of a semantic capability operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityBridgeRequest {
    interface: String,
    operation: String,
    input: Value,
}

impl CapabilityBridgeRequest {
    /// Creates a capability invocation request.
    #[must_use]
    pub fn new(interface: impl Into<String>, operation: impl Into<String>, input: Value) -> Self {
        Self {
            interface: interface.into(),
            operation: operation.into(),
            input,
        }
    }

    /// Returns the semantic capability interface.
    #[must_use]
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Returns the semantic operation identifier.
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.operation
    }

    /// Returns operation-specific JSON input.
    #[must_use]
    pub const fn input(&self) -> &Value {
        &self.input
    }
}

/// Response envelope for one capability bridge call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum CapabilityBridgeResponse {
    /// Operation completed successfully.
    Ok {
        /// Operation-specific JSON output.
        output: Value,
    },

    /// The transport or provider rejected the operation.
    Error {
        /// Stable transport/provider error category.
        code: String,

        /// Human-readable diagnostic message.
        message: String,
    },
}

impl CapabilityBridgeResponse {
    /// Creates a successful response.
    #[must_use]
    pub const fn ok(output: Value) -> Self {
        Self::Ok { output }
    }

    /// Creates an error response.
    #[must_use]
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn protocol_request_round_trips() {
        let request =
            CapabilityBridgeRequest::new("presolve:kv/store", "get", json!({ "key": "example" }));

        let bytes = serde_json::to_vec(&request).expect("request should serialize");
        let decoded: CapabilityBridgeRequest =
            serde_json::from_slice(&bytes).expect("request should deserialize");

        assert_eq!(decoded, request);
    }

    #[test]
    fn response_shape_is_explicit() {
        let response = CapabilityBridgeResponse::error("invalid-key", "empty key");
        let value = serde_json::to_value(response).expect("response should serialize");

        assert_eq!(value["status"], "error");
        assert_eq!(value["code"], "invalid-key");
    }
}
