use std::fmt;

use serde::{Deserialize, Serialize};

/// Interaction semantics of an application-internal interface.
///
/// Interface kinds describe how workloads logically communicate. They do not
/// prescribe a network protocol, serialization format, process boundary, or
/// deployment transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterfaceKind {
    /// A caller sends a request and expects a response.
    RequestResponse,

    /// A producer emits information without requiring a direct response.
    Event,
}

impl fmt::Display for InterfaceKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestResponse => formatter.write_str("request_response"),
            Self::Event => formatter.write_str("event"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_response_serializes_as_snake_case() {
        let encoded = serde_json::to_string(&InterfaceKind::RequestResponse)
            .expect("interface kind should serialize");

        assert_eq!(encoded, "\"request_response\"");
    }

    #[test]
    fn event_round_trips() {
        let encoded =
            serde_json::to_string(&InterfaceKind::Event).expect("interface kind should serialize");

        let decoded: InterfaceKind =
            serde_json::from_str(&encoded).expect("interface kind should deserialize");

        assert_eq!(decoded, InterfaceKind::Event);
    }
}
