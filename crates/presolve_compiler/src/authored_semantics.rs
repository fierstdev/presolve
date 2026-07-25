//! Canonical authored semantics, normalized at the syntax/TypeScript boundary.
//!
//! The parser owns source syntax and the TypeScript-authority package owns
//! resolved symbols. This module accepts the product of joining those two
//! boundaries; it deliberately does not inspect intrinsic spelling or import
//! paths. Legacy decorator extraction is a separate lowering concern.

use std::path::PathBuf;

use presolve_parser::ParsedFile;
use serde::{Deserialize, Serialize};

pub const CANONICAL_AUTHORED_SEMANTICS_SCHEMA_VERSION: u32 = 1;

/// A serializable source range shared by the syntax and semantic boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AuthoredSourceRangeV1 {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

/// A resolved declaration identity supplied by the TypeScript authority adapter.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ResolvedIntrinsicIdentityV1 {
    pub name: String,
    pub flags: u32,
    pub declaration_modules: Vec<String>,
}

/// The canonical intrinsic classification of a syntax-selected use site.
///
/// `kind` is only produced by the resolved-identity registry. It must never be
/// inferred from source spelling by a compiler consumer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalIntrinsicKindV1 {
    Component,
    State,
    Action,
    Computed,
    Effect,
    Slot,
    Context,
    Provide,
    Consume,
    Form,
    Serialize,
    Field,
    Validate,
    Submit,
    Resource,
    Loader,
    ServerAction,
    Opaque,
}

/// The authority-backed basis for one syntax-selected semantic candidate.
///
/// Intrinsics require a resolved framework identity. TSX bindings and event
/// references are syntax facts whose expression/type validation is supplied by
/// TypeScript queries, but they are not framework intrinsics themselves.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthoredSemanticCandidateKindV1 {
    ResolvedIntrinsic {
        intrinsic_kind: CanonicalIntrinsicKindV1,
        intrinsic_identity: ResolvedIntrinsicIdentityV1,
    },
    TsxBinding,
    TsxEventReference,
}

/// One candidate selected from the general source AST and checked by the
/// TypeScript authority adapter.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ResolvedAuthoredSemanticCandidateV1 {
    /// The source declaration or use-site being described. This is data for
    /// tooling and stable snapshots, not an identity authority.
    pub subject: String,
    pub source: AuthoredSourceRangeV1,
    pub kind: AuthoredSemanticCandidateKindV1,
}

/// A normalized authored declaration which later compiler products extend.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CanonicalAuthoredDeclarationV1 {
    pub kind: CanonicalAuthoredDeclarationKindV1,
    pub subject: String,
    pub source: AuthoredSourceRangeV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intrinsic_identity: Option<ResolvedIntrinsicIdentityV1>,
}

/// The source-independent vocabulary emitted at the authored-semantics
/// boundary. Each case records a framework meaning, not legacy syntax.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalAuthoredDeclarationKindV1 {
    Component,
    State,
    Action,
    Computed,
    Effect,
    Slot,
    ContextToken,
    ContextProvider,
    ContextConsumer,
    Form,
    Serialization,
    FormField,
    Validation,
    Submission,
    Resource,
    RouteLoader,
    ServerAction,
    Capability,
    TsxBinding,
    TsxEventReference,
}

/// The canonical output of syntax selection plus resolved intrinsic identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalAuthoredSemanticModelV1 {
    pub schema_version: u32,
    pub source_path: PathBuf,
    pub declarations: Vec<CanonicalAuthoredDeclarationV1>,
}

/// A boundary violation while normalizing authored semantic candidates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthoredSemanticNormalizationErrorV1 {
    InvalidSourceRange {
        subject: String,
        start: usize,
        end: usize,
        source_length: usize,
    },
}

impl std::fmt::Display for AuthoredSemanticNormalizationErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSourceRange {
                subject,
                start,
                end,
                source_length,
            } => write!(
                formatter,
                "authored semantic candidate `{subject}` has invalid source range {start}..{end} for source length {source_length}"
            ),
        }
    }
}

impl std::error::Error for AuthoredSemanticNormalizationErrorV1 {}

/// Normalize already-resolved syntax candidates for one parser product.
///
/// This is intentionally the first point where the two V2 authorities meet:
/// `ParsedFile::syntax` supplies the source extent, while callers supply a
/// resolved intrinsic classification from `typescript-authority`. No source
/// text, decorator name, or module specifier is used for recognition here.
pub fn normalize_authored_semantics_v1(
    parsed: &ParsedFile,
    candidates: impl IntoIterator<Item = ResolvedAuthoredSemanticCandidateV1>,
) -> Result<CanonicalAuthoredSemanticModelV1, AuthoredSemanticNormalizationErrorV1> {
    let source_length = parsed.syntax.source.len();
    let mut declarations = candidates
        .into_iter()
        .map(|candidate| {
            if candidate.source.start > candidate.source.end || candidate.source.end > source_length
            {
                return Err(AuthoredSemanticNormalizationErrorV1::InvalidSourceRange {
                    subject: candidate.subject,
                    start: candidate.source.start,
                    end: candidate.source.end,
                    source_length,
                });
            }
            let (kind, mut intrinsic_identity) = declaration_kind(candidate.kind);
            if let Some(identity) = &mut intrinsic_identity {
                identity.declaration_modules.sort();
                identity.declaration_modules.dedup();
            }
            Ok(CanonicalAuthoredDeclarationV1 {
                kind,
                subject: candidate.subject,
                source: candidate.source,
                intrinsic_identity,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    declarations.sort();
    declarations.dedup();
    Ok(CanonicalAuthoredSemanticModelV1 {
        schema_version: CANONICAL_AUTHORED_SEMANTICS_SCHEMA_VERSION,
        source_path: parsed.path.clone(),
        declarations,
    })
}

fn declaration_kind(
    kind: AuthoredSemanticCandidateKindV1,
) -> (
    CanonicalAuthoredDeclarationKindV1,
    Option<ResolvedIntrinsicIdentityV1>,
) {
    let AuthoredSemanticCandidateKindV1::ResolvedIntrinsic {
        intrinsic_kind,
        intrinsic_identity,
    } = kind
    else {
        return match kind {
            AuthoredSemanticCandidateKindV1::TsxBinding => {
                (CanonicalAuthoredDeclarationKindV1::TsxBinding, None)
            }
            AuthoredSemanticCandidateKindV1::TsxEventReference => {
                (CanonicalAuthoredDeclarationKindV1::TsxEventReference, None)
            }
            AuthoredSemanticCandidateKindV1::ResolvedIntrinsic { .. } => unreachable!(),
        };
    };

    let declaration_kind = match intrinsic_kind {
        CanonicalIntrinsicKindV1::Component => CanonicalAuthoredDeclarationKindV1::Component,
        CanonicalIntrinsicKindV1::State => CanonicalAuthoredDeclarationKindV1::State,
        CanonicalIntrinsicKindV1::Action => CanonicalAuthoredDeclarationKindV1::Action,
        CanonicalIntrinsicKindV1::Computed => CanonicalAuthoredDeclarationKindV1::Computed,
        CanonicalIntrinsicKindV1::Effect => CanonicalAuthoredDeclarationKindV1::Effect,
        CanonicalIntrinsicKindV1::Slot => CanonicalAuthoredDeclarationKindV1::Slot,
        CanonicalIntrinsicKindV1::Context => CanonicalAuthoredDeclarationKindV1::ContextToken,
        CanonicalIntrinsicKindV1::Provide => CanonicalAuthoredDeclarationKindV1::ContextProvider,
        CanonicalIntrinsicKindV1::Consume => CanonicalAuthoredDeclarationKindV1::ContextConsumer,
        CanonicalIntrinsicKindV1::Form => CanonicalAuthoredDeclarationKindV1::Form,
        CanonicalIntrinsicKindV1::Serialize => CanonicalAuthoredDeclarationKindV1::Serialization,
        CanonicalIntrinsicKindV1::Field => CanonicalAuthoredDeclarationKindV1::FormField,
        CanonicalIntrinsicKindV1::Validate => CanonicalAuthoredDeclarationKindV1::Validation,
        CanonicalIntrinsicKindV1::Submit => CanonicalAuthoredDeclarationKindV1::Submission,
        CanonicalIntrinsicKindV1::Resource => CanonicalAuthoredDeclarationKindV1::Resource,
        CanonicalIntrinsicKindV1::Loader => CanonicalAuthoredDeclarationKindV1::RouteLoader,
        CanonicalIntrinsicKindV1::ServerAction => CanonicalAuthoredDeclarationKindV1::ServerAction,
        CanonicalIntrinsicKindV1::Opaque => CanonicalAuthoredDeclarationKindV1::Capability,
    };
    (declaration_kind, Some(intrinsic_identity))
}

#[cfg(test)]
mod tests {
    use presolve_parser::parse_file;

    use super::{
        normalize_authored_semantics_v1, AuthoredSemanticCandidateKindV1,
        AuthoredSemanticNormalizationErrorV1, AuthoredSourceRangeV1,
        CanonicalAuthoredDeclarationKindV1, CanonicalIntrinsicKindV1,
        ResolvedAuthoredSemanticCandidateV1, ResolvedIntrinsicIdentityV1,
    };

    fn candidate(
        subject: &str,
        start: usize,
        kind: CanonicalIntrinsicKindV1,
    ) -> ResolvedAuthoredSemanticCandidateV1 {
        ResolvedAuthoredSemanticCandidateV1 {
            subject: subject.to_owned(),
            source: AuthoredSourceRangeV1 {
                start,
                end: start + 5,
                line: 1,
                column: start + 1,
            },
            kind: AuthoredSemanticCandidateKindV1::ResolvedIntrinsic {
                intrinsic_kind: kind,
                intrinsic_identity: ResolvedIntrinsicIdentityV1 {
                    name: "renamedFrameworkExport".to_owned(),
                    flags: 2_097_152,
                    declaration_modules: vec![
                        "node_modules/@presolve/framework/index.d.ts".to_owned()
                    ],
                },
            },
        }
    }

    #[test]
    fn normalizes_resolved_candidates_without_using_source_spelling() {
        let parsed = parse_file("src/Card.tsx", "const Card = frameworkUse();");
        let state = candidate("Card.count", 20, CanonicalIntrinsicKindV1::State);
        let component = candidate("Card", 6, CanonicalIntrinsicKindV1::Component);

        let model =
            normalize_authored_semantics_v1(&parsed, [state.clone(), component.clone(), state])
                .expect("valid resolved candidates");

        assert_eq!(model.schema_version, 1);
        assert_eq!(model.declarations.len(), 2);
        assert_eq!(model.declarations[0].subject, "Card");
        assert_eq!(
            model.declarations[0].kind,
            CanonicalAuthoredDeclarationKindV1::Component
        );
        assert_eq!(model.declarations[1].subject, "Card.count");
        assert_eq!(
            model.declarations[1].kind,
            CanonicalAuthoredDeclarationKindV1::State
        );
        assert_eq!(
            model.declarations[1]
                .intrinsic_identity
                .as_ref()
                .unwrap()
                .name,
            "renamedFrameworkExport"
        );
        assert_eq!(
            serde_json::to_value(&model).expect("serializable model"),
            serde_json::json!({
                "schema_version": 1,
                "source_path": "src/Card.tsx",
                "declarations": [
                    {
                        "kind": "component",
                        "subject": "Card",
                        "source": { "start": 6, "end": 11, "line": 1, "column": 7 },
                        "intrinsic_identity": {
                            "name": "renamedFrameworkExport",
                            "flags": 2_097_152,
                            "declaration_modules": ["node_modules/@presolve/framework/index.d.ts"]
                        }
                    },
                    {
                        "kind": "state",
                        "subject": "Card.count",
                        "source": { "start": 20, "end": 25, "line": 1, "column": 21 },
                        "intrinsic_identity": {
                            "name": "renamedFrameworkExport",
                            "flags": 2_097_152,
                            "declaration_modules": ["node_modules/@presolve/framework/index.d.ts"]
                        }
                    }
                ]
            })
        );
    }

    #[test]
    fn retains_tsx_binding_and_event_facts_without_an_intrinsic_identity() {
        let parsed = parse_file(
            "src/Card.tsx",
            "const Card = <button onClick={save}>{count}</button>;",
        );
        let binding = ResolvedAuthoredSemanticCandidateV1 {
            subject: "count".to_owned(),
            source: AuthoredSourceRangeV1 {
                start: 44,
                end: 49,
                line: 1,
                column: 45,
            },
            kind: AuthoredSemanticCandidateKindV1::TsxBinding,
        };
        let event = ResolvedAuthoredSemanticCandidateV1 {
            subject: "save".to_owned(),
            source: AuthoredSourceRangeV1 {
                start: 37,
                end: 41,
                line: 1,
                column: 38,
            },
            kind: AuthoredSemanticCandidateKindV1::TsxEventReference,
        };

        let model = normalize_authored_semantics_v1(&parsed, [binding, event])
            .expect("TSX syntax candidates fit the source AST");

        assert_eq!(model.declarations.len(), 2);
        assert!(model.declarations.iter().any(|declaration| {
            declaration.kind == CanonicalAuthoredDeclarationKindV1::TsxBinding
                && declaration.intrinsic_identity.is_none()
        }));
        assert!(model.declarations.iter().any(|declaration| {
            declaration.kind == CanonicalAuthoredDeclarationKindV1::TsxEventReference
                && declaration.intrinsic_identity.is_none()
        }));
    }

    #[test]
    fn rejects_candidates_outside_the_general_source_ast_extent() {
        let parsed = parse_file("src/Card.tsx", "const Card = 1;");
        let error = normalize_authored_semantics_v1(
            &parsed,
            [candidate("Card", 99, CanonicalIntrinsicKindV1::Component)],
        )
        .expect_err("invalid range must not enter the canonical model");

        assert!(matches!(
            error,
            AuthoredSemanticNormalizationErrorV1::InvalidSourceRange { subject, .. }
                if subject == "Card"
        ));
    }
}
