use std::fmt;

use serde::{Deserialize, Serialize};

/// Execution class of an application workload.
///
/// Workload kinds describe the artifact/runtime semantics required to realize a
/// workload. They do not identify a deployment environment or infrastructure
/// provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkloadKind {
    /// WebAssembly Component workload.
    Component,

    /// OCI-compatible container workload.
    Container,
}

impl fmt::Display for WorkloadKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Component => formatter.write_str("component"),
            Self::Container => formatter.write_str("container"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_serializes_as_component() {
        let encoded = serde_json::to_string(&WorkloadKind::Component)
            .expect("workload kind should serialize");

        assert_eq!(encoded, "\"component\"");
    }

    #[test]
    fn component_round_trips() {
        let encoded = serde_json::to_string(&WorkloadKind::Component)
            .expect("workload kind should serialize");

        let decoded: WorkloadKind =
            serde_json::from_str(&encoded).expect("workload kind should deserialize");

        assert_eq!(decoded, WorkloadKind::Component);
    }

    #[test]
    fn container_serializes_and_round_trips() {
        let encoded = serde_json::to_string(&WorkloadKind::Container)
            .expect("container workload kind should serialize");

        assert_eq!(encoded, "\"container\"");

        let decoded: WorkloadKind =
            serde_json::from_str(&encoded).expect("container workload kind should deserialize");

        assert_eq!(decoded, WorkloadKind::Container);
        assert_eq!(decoded.to_string(), "container");
    }
}
