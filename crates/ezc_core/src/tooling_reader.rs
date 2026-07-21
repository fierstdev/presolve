//! L11-B byte-only readers for the explicitly approved L3 tooling products.
//!
//! A reader negotiates first, then delegates to an existing strict canonical
//! decoder. It never opens a path, parses source, invokes a compiler session,
//! or widens L3--L8 persistence.

#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

use std::fmt;

use crate::platform::{
    decode_workspace_graph_json_v1, decode_workspace_snapshot_json_v1, WorkspaceGraph,
    WorkspaceSnapshot,
};
use crate::tooling_schema::{
    decode_tooling_schema_negotiation_request_v1, negotiate_tooling_schema_v1,
};

pub const WORKSPACE_SNAPSHOT_TOOLING_SCHEMA_V1: &str = "presolve.workspace-snapshot";
pub const WORKSPACE_GRAPH_TOOLING_SCHEMA_V1: &str = "presolve.workspace-graph";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolingProductV1 {
    WorkspaceSnapshot(WorkspaceSnapshot),
    WorkspaceGraph(WorkspaceGraph),
}

impl ToolingProductV1 {
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        match self {
            Self::WorkspaceSnapshot(_) => WORKSPACE_SNAPSHOT_TOOLING_SCHEMA_V1,
            Self::WorkspaceGraph(_) => WORKSPACE_GRAPH_TOOLING_SCHEMA_V1,
        }
    }

    #[must_use]
    pub const fn version(&self) -> u32 {
        1
    }

    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        match self {
            Self::WorkspaceSnapshot(snapshot) => snapshot.snapshot_id.as_str(),
            Self::WorkspaceGraph(graph) => graph.snapshot_id.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolingProductReadErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for ToolingProductReadErrorV1 {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ToolingProductReadErrorV1 {}

/// Reads one caller-supplied immutable L3 product after strict L10
/// negotiation. This function owns no filesystem access: its caller supplies
/// exactly one byte slice.
pub fn read_tooling_product_v1(
    schema: &str,
    versions: &[u32],
    bytes: &[u8],
) -> Result<ToolingProductV1, ToolingProductReadErrorV1> {
    let negotiation = serde_json::to_vec(&serde_json::json!({
        "schema": schema,
        "versions": versions,
    }))
    .expect("tooling negotiation request serializes");
    let request = decode_tooling_schema_negotiation_request_v1(&negotiation).map_err(|error| {
        ToolingProductReadErrorV1 {
            code: "L11T001",
            message: format!("schema negotiation rejected: {error}"),
        }
    })?;
    let accepted =
        negotiate_tooling_schema_v1(&request).map_err(|error| ToolingProductReadErrorV1 {
            code: "L11T001",
            message: format!("schema negotiation rejected: {error}"),
        })?;
    if accepted.accepted_version != 1 {
        return Err(ToolingProductReadErrorV1 {
            code: "L11T006",
            message: "accepted tooling schema version does not match the reader".into(),
        });
    }

    match accepted.schema.as_str() {
        WORKSPACE_SNAPSHOT_TOOLING_SCHEMA_V1 => decode_workspace_snapshot_json_v1(bytes)
            .map(ToolingProductV1::WorkspaceSnapshot)
            .map_err(|error| ToolingProductReadErrorV1 {
                code: "L11T003",
                message: format!("workspace snapshot fails strict canonical validation: {error:?}"),
            }),
        WORKSPACE_GRAPH_TOOLING_SCHEMA_V1 => decode_workspace_graph_json_v1(bytes)
            .map(ToolingProductV1::WorkspaceGraph)
            .map_err(|error| ToolingProductReadErrorV1 {
                code: "L11T003",
                message: format!("workspace graph fails strict canonical validation: {error:?}"),
            }),
        _ => Err(ToolingProductReadErrorV1 {
            code: "L11T005",
            message: "no accepted public reader exists for the negotiated schema".into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{
        derive_workspace_id_v1, CacheLimits, CancellationToken, CompilationOutcome,
        CompileWorkspaceRequest, CompilerSessionState, RequestedCompilationMode, WorkspaceInput,
        WorkspaceSource,
    };

    fn canonical_products() -> (Vec<u8>, Vec<u8>) {
        let workspace = WorkspaceInput::new(vec![WorkspaceSource {
            path: "src/ToolingFixture.ts".into(),
            source: "export const toolingFixture = 1;\n".into(),
            language: None,
        }]);
        let workspace_id = derive_workspace_id_v1(&workspace.configuration).unwrap();
        let mut session = CompilerSessionState::new(
            workspace_id,
            workspace.compiler_contract.clone(),
            CacheLimits::default(),
        );
        let CompilationOutcome::Committed(result) =
            session.compile_workspace(CompileWorkspaceRequest {
                workspace,
                mode: RequestedCompilationMode::Full,
                cancellation: CancellationToken::new(),
            })
        else {
            panic!("tooling reader fixture compilation must commit");
        };
        let snapshot = result.snapshot.to_canonical_json().unwrap();
        let graph = result.graph.to_canonical_json().unwrap();
        assert!(!String::from_utf8_lossy(&snapshot).contains("toolingFixture"));
        assert!(!String::from_utf8_lossy(&graph).contains("toolingFixture"));
        (snapshot, graph)
    }

    #[test]
    fn l11b_reads_only_strict_canonical_l3_product_fixtures() {
        let (snapshot_bytes, graph_bytes) = canonical_products();
        let snapshot =
            read_tooling_product_v1(WORKSPACE_SNAPSHOT_TOOLING_SCHEMA_V1, &[1], &snapshot_bytes)
                .unwrap();
        let graph =
            read_tooling_product_v1(WORKSPACE_GRAPH_TOOLING_SCHEMA_V1, &[2, 1], &graph_bytes)
                .unwrap();
        assert_eq!(snapshot.schema(), WORKSPACE_SNAPSHOT_TOOLING_SCHEMA_V1);
        assert_eq!(graph.schema(), WORKSPACE_GRAPH_TOOLING_SCHEMA_V1);
        assert_eq!(snapshot.version(), 1);
        assert_eq!(graph.version(), 1);
        assert_eq!(snapshot.snapshot_id(), graph.snapshot_id());
    }

    #[test]
    fn l11b_rejects_unavailable_products_and_noncanonical_bytes() {
        let (snapshot_bytes, _) = canonical_products();
        for schema in ["presolve.build-trace", "presolve.unknown"] {
            let error = read_tooling_product_v1(schema, &[1], &snapshot_bytes).unwrap_err();
            assert_eq!(error.code, "L11T001");
        }
        for schema in ["presolve.workspace-configuration", "presolve.watch-event"] {
            let error = read_tooling_product_v1(schema, &[1], &snapshot_bytes).unwrap_err();
            assert_eq!(error.code, "L11T005");
        }

        let mut noncanonical = snapshot_bytes;
        noncanonical.pop();
        let error =
            read_tooling_product_v1(WORKSPACE_SNAPSHOT_TOOLING_SCHEMA_V1, &[1], &noncanonical)
                .unwrap_err();
        assert_eq!(error.code, "L11T003");
    }

    #[test]
    fn l11b_reader_order_and_version_validation_are_deterministic() {
        let (snapshot_bytes, graph_bytes) = canonical_products();
        let forward = [
            (
                WORKSPACE_SNAPSHOT_TOOLING_SCHEMA_V1,
                snapshot_bytes.as_slice(),
            ),
            (WORKSPACE_GRAPH_TOOLING_SCHEMA_V1, graph_bytes.as_slice()),
        ]
        .map(|(schema, bytes)| read_tooling_product_v1(schema, &[1], bytes).unwrap());
        let reverse = [
            (WORKSPACE_GRAPH_TOOLING_SCHEMA_V1, graph_bytes.as_slice()),
            (
                WORKSPACE_SNAPSHOT_TOOLING_SCHEMA_V1,
                snapshot_bytes.as_slice(),
            ),
        ]
        .map(|(schema, bytes)| read_tooling_product_v1(schema, &[1], bytes).unwrap());
        assert_eq!(forward[0].snapshot_id(), reverse[1].snapshot_id());
        assert_eq!(forward[1].snapshot_id(), reverse[0].snapshot_id());

        let error =
            read_tooling_product_v1(WORKSPACE_GRAPH_TOOLING_SCHEMA_V1, &[1, 1], &graph_bytes)
                .unwrap_err();
        assert_eq!(error.code, "L11T001");
    }
}
