//! Manifest-backed lowering for TypeScript-proven V2 environment reads.

use std::path::PathBuf;

use presolve_parser::{ParsedCallArgument, ParsedFile};
use serde::Serialize;

use crate::{
    AuthoredSourceRangeV1, EnvironmentInputManifestV1, ResolvedEnvironmentPublicReadV1,
    ResolvedIntrinsicIdentityV1, ENVIRONMENT_INPUT_SCHEMA_VERSION,
};

pub const ENVIRONMENT_READ_LOWERING_SCHEMA_VERSION: u32 = 1;

/// One browser-safe environment value joined to its exact source and
/// TypeScript-authoritative framework identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnvironmentReadRecordV1 {
    pub source_path: PathBuf,
    pub call_source: AuthoredSourceRangeV1,
    pub name_source: AuthoredSourceRangeV1,
    pub name: String,
    pub browser_value: String,
    pub environment_public_identity: ResolvedIntrinsicIdentityV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentReadDiagnosticCodeV1 {
    InvalidCallArgument,
    ManifestRequired,
    ManifestSchemaUnsupported,
    ServerValueForbidden,
    NameNotBrowserPublic,
    ValueNotDeclared,
}

impl EnvironmentReadDiagnosticCodeV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidCallArgument => "PSENV1101_INVALID_CALL_ARGUMENT",
            Self::ManifestRequired => "PSENV1102_MANIFEST_REQUIRED",
            Self::ManifestSchemaUnsupported => "PSENV1103_MANIFEST_SCHEMA_UNSUPPORTED",
            Self::ServerValueForbidden => "PSENV1104_SERVER_VALUE_FORBIDDEN",
            Self::NameNotBrowserPublic => "PSENV1105_NAME_NOT_BROWSER_PUBLIC",
            Self::ValueNotDeclared => "PSENV1106_VALUE_NOT_DECLARED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnvironmentReadDiagnosticV1 {
    pub code: EnvironmentReadDiagnosticCodeV1,
    pub call_source: AuthoredSourceRangeV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnvironmentReadLoweringV1 {
    pub schema_version: u32,
    pub reads: Vec<EnvironmentReadRecordV1>,
    pub diagnostics: Vec<EnvironmentReadDiagnosticV1>,
}

/// Lowers only exact authority evidence.  Every rejected input omits a read
/// record, so no browser value is exposed while diagnostics remain stable.
#[must_use]
pub fn lower_environment_reads_v1(
    parsed: &ParsedFile,
    evidence: impl IntoIterator<Item = ResolvedEnvironmentPublicReadV1>,
    manifest: Option<&EnvironmentInputManifestV1>,
) -> EnvironmentReadLoweringV1 {
    let mut reads = Vec::new();
    let mut diagnostics = Vec::new();
    for resolved in evidence {
        let Some(call) = parsed.call_expressions.iter().find(|call| {
            call.span.start == resolved.call_source.start
                && call.span.end == resolved.call_source.end
        }) else {
            diagnostics.push(EnvironmentReadDiagnosticV1 {
                code: EnvironmentReadDiagnosticCodeV1::InvalidCallArgument,
                call_source: resolved.call_source,
            });
            continue;
        };
        let [ParsedCallArgument::StringLiteral { value, span }] = call.arguments.as_slice() else {
            diagnostics.push(EnvironmentReadDiagnosticV1 {
                code: EnvironmentReadDiagnosticCodeV1::InvalidCallArgument,
                call_source: resolved.call_source,
            });
            continue;
        };
        let Some(manifest) = manifest else {
            diagnostics.push(EnvironmentReadDiagnosticV1 {
                code: EnvironmentReadDiagnosticCodeV1::ManifestRequired,
                call_source: resolved.call_source,
            });
            continue;
        };
        if manifest.schema_version != ENVIRONMENT_INPUT_SCHEMA_VERSION {
            diagnostics.push(EnvironmentReadDiagnosticV1 {
                code: EnvironmentReadDiagnosticCodeV1::ManifestSchemaUnsupported,
                call_source: resolved.call_source,
            });
            continue;
        }
        if manifest.server_value_names.binary_search(value).is_ok() {
            diagnostics.push(EnvironmentReadDiagnosticV1 {
                code: EnvironmentReadDiagnosticCodeV1::ServerValueForbidden,
                call_source: resolved.call_source,
            });
            continue;
        }
        if !value.starts_with("PRESOLVE_PUBLIC_") || value.len() == "PRESOLVE_PUBLIC_".len() {
            diagnostics.push(EnvironmentReadDiagnosticV1 {
                code: EnvironmentReadDiagnosticCodeV1::NameNotBrowserPublic,
                call_source: resolved.call_source,
            });
            continue;
        }
        let Some(browser_value) = manifest.browser_values.get(value) else {
            diagnostics.push(EnvironmentReadDiagnosticV1 {
                code: EnvironmentReadDiagnosticCodeV1::ValueNotDeclared,
                call_source: resolved.call_source,
            });
            continue;
        };
        reads.push(EnvironmentReadRecordV1 {
            source_path: parsed.path.clone(),
            call_source: resolved.call_source,
            name_source: range_from_parser_span(*span),
            name: value.clone(),
            browser_value: browser_value.clone(),
            environment_public_identity: resolved.environment_public_identity,
        });
    }
    reads.sort_by_key(|read| (read.source_path.clone(), read.call_source.start));
    diagnostics.sort_by_key(|diagnostic| {
        (
            diagnostic.call_source.start,
            diagnostic.call_source.end,
            diagnostic.code,
        )
    });
    EnvironmentReadLoweringV1 {
        schema_version: ENVIRONMENT_READ_LOWERING_SCHEMA_VERSION,
        reads,
        diagnostics,
    }
}

fn range_from_parser_span(span: presolve_parser::SourceSpan) -> AuthoredSourceRangeV1 {
    AuthoredSourceRangeV1 {
        start: span.start,
        end: span.end,
        line: span.line,
        column: span.column,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use presolve_parser::parse_file;

    use crate::{build_environment_input_manifest_v1, ResolvedEnvironmentPublicReadV1};

    use super::{
        lower_environment_reads_v1, EnvironmentReadDiagnosticCodeV1,
        ENVIRONMENT_READ_LOWERING_SCHEMA_VERSION,
    };

    fn evidence(
        parsed: &presolve_parser::ParsedFile,
        index: usize,
    ) -> ResolvedEnvironmentPublicReadV1 {
        let call = &parsed.call_expressions[index];
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
    fn joins_only_literal_public_reads_to_manifest_values() {
        let source = r#"
const appName = runtimeEnvironment.public("PRESOLVE_PUBLIC_APP_NAME");
const database = runtimeEnvironment.public("DATABASE_URL");
const dynamic = runtimeEnvironment.public(name);
const unprefixed = runtimeEnvironment.public("UNDECLARED_VALUE");
const missing = runtimeEnvironment.public("PRESOLVE_PUBLIC_MISSING");
"#;
        let parsed = parse_file("src/environment.ts", source);
        let manifest = build_environment_input_manifest_v1(
            ".env",
            &BTreeMap::from([
                ("PRESOLVE_PUBLIC_APP_NAME".into(), "Presolve".into()),
                ("DATABASE_URL".into(), "postgres://secret".into()),
            ]),
        )
        .unwrap();
        let lowered = lower_environment_reads_v1(
            &parsed,
            [
                evidence(&parsed, 0),
                evidence(&parsed, 1),
                evidence(&parsed, 2),
                evidence(&parsed, 3),
                evidence(&parsed, 4),
            ],
            Some(&manifest),
        );

        assert_eq!(
            lowered.schema_version,
            ENVIRONMENT_READ_LOWERING_SCHEMA_VERSION
        );
        assert_eq!(lowered.reads.len(), 1);
        assert_eq!(lowered.reads[0].name, "PRESOLVE_PUBLIC_APP_NAME");
        assert_eq!(lowered.reads[0].browser_value, "Presolve");
        assert!(!format!("{lowered:?}").contains("postgres://secret"));
        assert_eq!(
            lowered
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            vec![
                EnvironmentReadDiagnosticCodeV1::ServerValueForbidden,
                EnvironmentReadDiagnosticCodeV1::InvalidCallArgument,
                EnvironmentReadDiagnosticCodeV1::NameNotBrowserPublic,
                EnvironmentReadDiagnosticCodeV1::ValueNotDeclared,
            ]
        );
    }

    #[test]
    fn fails_closed_without_a_current_manifest() {
        let parsed = parse_file(
            "src/environment.ts",
            "const appName = runtimeEnvironment.public(\"PRESOLVE_PUBLIC_APP_NAME\");",
        );
        let lowered = lower_environment_reads_v1(&parsed, [evidence(&parsed, 0)], None);
        assert!(lowered.reads.is_empty());
        assert_eq!(
            lowered.diagnostics[0].code,
            EnvironmentReadDiagnosticCodeV1::ManifestRequired
        );
        assert_eq!(
            lowered.diagnostics[0].code.as_str(),
            "PSENV1102_MANIFEST_REQUIRED"
        );
    }
}
