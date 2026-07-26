//! Deterministic browser-environment artifact projection.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::{EnvironmentReadLoweringV1, ENVIRONMENT_READ_LOWERING_SCHEMA_VERSION};

pub const ENVIRONMENT_PUBLICATION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentPublicationArtifactV1 {
    pub schema_version: u32,
    pub browser_values: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentPublicationErrorV1 {
    LoweringSchemaVersion(u32),
    LoweringDiagnostics(usize),
    ConflictingValue { name: String },
}

impl std::fmt::Display for EnvironmentPublicationErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoweringSchemaVersion(version) => write!(
                formatter,
                "unsupported environment-read lowering schema version {version}"
            ),
            Self::LoweringDiagnostics(count) => write!(
                formatter,
                "cannot publish environment reads with {count} lowering diagnostic(s)"
            ),
            Self::ConflictingValue { name } => write!(
                formatter,
                "environment read records disagree on browser value for `{name}`"
            ),
        }
    }
}

impl std::error::Error for EnvironmentPublicationErrorV1 {}

/// Publishes only the exact values proven by source lowering.  A diagnostic is
/// a whole-product failure, so an adapter never receives a partial artifact.
pub fn build_environment_publication_artifact_v1(
    lowering: &EnvironmentReadLoweringV1,
) -> Result<EnvironmentPublicationArtifactV1, EnvironmentPublicationErrorV1> {
    if lowering.schema_version != ENVIRONMENT_READ_LOWERING_SCHEMA_VERSION {
        return Err(EnvironmentPublicationErrorV1::LoweringSchemaVersion(
            lowering.schema_version,
        ));
    }
    if !lowering.diagnostics.is_empty() {
        return Err(EnvironmentPublicationErrorV1::LoweringDiagnostics(
            lowering.diagnostics.len(),
        ));
    }
    let mut browser_values = BTreeMap::new();
    for read in &lowering.reads {
        if let Some(previous) = browser_values.insert(read.name.clone(), read.browser_value.clone())
        {
            if previous != read.browser_value {
                return Err(EnvironmentPublicationErrorV1::ConflictingValue {
                    name: read.name.clone(),
                });
            }
        }
    }
    Ok(EnvironmentPublicationArtifactV1 {
        schema_version: ENVIRONMENT_PUBLICATION_SCHEMA_VERSION,
        browser_values,
    })
}

#[must_use]
pub fn environment_publication_artifact_json_v1(
    artifact: &EnvironmentPublicationArtifactV1,
) -> String {
    serde_json::to_string_pretty(artifact).expect("environment publication artifact serializes")
        + "\n"
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use presolve_parser::parse_file;

    use crate::{
        build_environment_input_manifest_v1, lower_environment_reads_v1,
        EnvironmentReadDiagnosticCodeV1, ResolvedEnvironmentPublicReadV1,
    };

    use super::{
        build_environment_publication_artifact_v1, environment_publication_artifact_json_v1,
        EnvironmentPublicationErrorV1, ENVIRONMENT_PUBLICATION_SCHEMA_VERSION,
    };

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
    fn publishes_only_source_proven_public_values() {
        let parsed = parse_file(
            "src/environment.ts",
            "const name = runtimeEnvironment.public(\"PRESOLVE_PUBLIC_NAME\");",
        );
        let manifest = build_environment_input_manifest_v1(
            ".env",
            &BTreeMap::from([
                ("PRESOLVE_PUBLIC_NAME".into(), "Presolve".into()),
                ("DATABASE_URL".into(), "postgres://secret".into()),
            ]),
        )
        .unwrap();
        let lowering = lower_environment_reads_v1(&parsed, [evidence(&parsed)], Some(&manifest));
        let artifact = build_environment_publication_artifact_v1(&lowering).unwrap();

        assert_eq!(
            artifact.schema_version,
            ENVIRONMENT_PUBLICATION_SCHEMA_VERSION
        );
        assert_eq!(
            artifact.browser_values,
            BTreeMap::from([("PRESOLVE_PUBLIC_NAME".into(), "Presolve".into(),)])
        );
        assert!(!environment_publication_artifact_json_v1(&artifact).contains("postgres://secret"));
    }

    #[test]
    fn rejects_any_diagnostic_instead_of_publishing_a_partial_artifact() {
        let parsed = parse_file(
            "src/environment.ts",
            "const name = runtimeEnvironment.public(dynamicName);",
        );
        let lowering = lower_environment_reads_v1(&parsed, [evidence(&parsed)], None);
        assert_eq!(
            lowering.diagnostics[0].code,
            EnvironmentReadDiagnosticCodeV1::InvalidCallArgument
        );
        assert_eq!(
            build_environment_publication_artifact_v1(&lowering),
            Err(EnvironmentPublicationErrorV1::LoweringDiagnostics(1))
        );
    }
}
