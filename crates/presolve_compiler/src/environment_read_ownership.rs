//! Projects manifest-backed environment reads onto the existing ownership graph.

use crate::{
    build_environment_ownership_graph_v1, ControlFlowProvenanceV1, EnvironmentClassV1,
    EnvironmentOwnershipFactsV1, EnvironmentOwnershipGraphV1, EnvironmentOwnershipNodeV1,
    EnvironmentReadLoweringV1, LifetimeClassV1, SemanticId,
    ENVIRONMENT_READ_LOWERING_SCHEMA_VERSION,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentReadOwnershipErrorV1 {
    LoweringSchemaVersion(u32),
    LoweringDiagnostics(usize),
    Graph(crate::EnvironmentOwnershipErrorV1),
}

impl std::fmt::Display for EnvironmentReadOwnershipErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoweringSchemaVersion(version) => write!(
                formatter,
                "unsupported environment-read lowering schema version {version}"
            ),
            Self::LoweringDiagnostics(count) => write!(
                formatter,
                "cannot project environment ownership with {count} lowering diagnostic(s)"
            ),
            Self::Graph(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for EnvironmentReadOwnershipErrorV1 {}

/// Assigns every admitted environment read the browser/application ownership
/// class. Server values cannot reach this product because source lowering
/// rejects them before creating a read record.
pub fn build_environment_read_ownership_v1(
    lowering: &EnvironmentReadLoweringV1,
) -> Result<EnvironmentOwnershipGraphV1, EnvironmentReadOwnershipErrorV1> {
    if lowering.schema_version != ENVIRONMENT_READ_LOWERING_SCHEMA_VERSION {
        return Err(EnvironmentReadOwnershipErrorV1::LoweringSchemaVersion(
            lowering.schema_version,
        ));
    }
    if !lowering.diagnostics.is_empty() {
        return Err(EnvironmentReadOwnershipErrorV1::LoweringDiagnostics(
            lowering.diagnostics.len(),
        ));
    }
    let nodes = lowering
        .reads
        .iter()
        .map(|read| EnvironmentOwnershipNodeV1 {
            id: SemanticId::environment_read_in_module(&read.source_path, read.call_source.start),
            environment: EnvironmentClassV1::Browser,
            lifetime: LifetimeClassV1::Application,
            provenance: ControlFlowProvenanceV1 {
                path: read.source_path.to_string_lossy().into_owned(),
                start: read.call_source.start,
                end: read.call_source.end,
                line: read.call_source.line,
                column: read.call_source.column,
            },
        })
        .collect::<Vec<_>>();
    let browser_artifact_roots = nodes.iter().map(|node| node.id.clone()).collect();
    build_environment_ownership_graph_v1(&EnvironmentOwnershipFactsV1 {
        nodes,
        edges: Vec::new(),
        browser_artifact_roots,
        shared_publication_roots: Vec::new(),
    })
    .map_err(EnvironmentReadOwnershipErrorV1::Graph)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use presolve_parser::parse_file;

    use crate::{
        build_environment_input_manifest_v1, lower_environment_reads_v1, EnvironmentClassV1,
        ResolvedEnvironmentPublicReadV1,
    };

    use super::{build_environment_read_ownership_v1, EnvironmentReadOwnershipErrorV1};

    fn evidence(parsed: &presolve_parser::ParsedFile) -> ResolvedEnvironmentPublicReadV1 {
        let call = &parsed.call_expressions[0];
        ResolvedEnvironmentPublicReadV1 {
            call_source: crate::AuthoredSourceRangeV1 {
                start: call.span.start,
                end: call.span.end,
                line: call.span.line,
                column: call.span.column,
            },
            environment_public_identity: crate::ResolvedIntrinsicIdentityV1 {
                name: "public".into(),
                flags: 32,
                declaration_modules: vec!["presolve".into()],
            },
        }
    }

    #[test]
    fn consumes_only_admitted_browser_reads() {
        let parsed = parse_file(
            "src/environment.ts",
            "const name = runtimeEnvironment.public(\"PRESOLVE_PUBLIC_NAME\");",
        );
        let manifest = build_environment_input_manifest_v1(
            ".env",
            &BTreeMap::from([("PRESOLVE_PUBLIC_NAME".into(), "Presolve".into())]),
        )
        .unwrap();
        let lowering = lower_environment_reads_v1(&parsed, [evidence(&parsed)], Some(&manifest));
        let graph = build_environment_read_ownership_v1(&lowering).unwrap();

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].environment, EnvironmentClassV1::Browser);
        assert_eq!(
            graph.nodes[0].id.as_str(),
            "module:src/environment.ts/environment-read:13"
        );
        assert!(graph.diagnostics.is_empty());
    }

    #[test]
    fn refuses_to_reclassify_failed_environment_reads() {
        let parsed = parse_file(
            "src/environment.ts",
            "const name = runtimeEnvironment.public(dynamicName);",
        );
        let lowering = lower_environment_reads_v1(&parsed, [evidence(&parsed)], None);
        assert_eq!(
            build_environment_read_ownership_v1(&lowering),
            Err(EnvironmentReadOwnershipErrorV1::LoweringDiagnostics(1))
        );
    }
}
