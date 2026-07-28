//! Versioned codec declarations over canonical semantic types.
//!
//! This product is a protocol ledger. It does not encode values, decode durable
//! bytes, or replace the existing Form and resume codec authorities.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::{resume_value_codec, semantic_type_text, SemanticType};

pub const CODEC_PROTOCOL_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CodecClassificationV1 {
    Approved,
    Rejected,
}

/// Independent classifications; these must never be reduced to one boolean.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CodecClassificationsV1 {
    pub runtime_validated: CodecClassificationV1,
    pub form_serializable: CodecClassificationV1,
    pub network_serializable: CodecClassificationV1,
    pub html_publishable: CodecClassificationV1,
    pub resume_serializable: CodecClassificationV1,
    pub structured_cloneable: CodecClassificationV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CodecRepresentationV1 {
    Json,
    FormDataScalar,
    UrlEncodedScalar,
    PlatformValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CodecEnvironmentV1 {
    Server,
    Browser,
    Shared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CodecBehaviorV1 {
    Total,
    Fallible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CodecFailureBehaviorV1 {
    RejectValue,
    RejectPayload,
    SurfaceDiagnostic,
}

/// One explicit codec contract. The source type remains compiler-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodecDeclarationV1 {
    pub id: String,
    pub source_type: SemanticType,
    pub representation: CodecRepresentationV1,
    pub environments: Vec<CodecEnvironmentV1>,
    pub encode_behavior: CodecBehaviorV1,
    pub decode_behavior: CodecBehaviorV1,
    pub version: u32,
    pub failure_behavior: CodecFailureBehaviorV1,
}

/// Serializable inspection record that retains all codec declaration fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CodecProtocolRecordV1 {
    pub id: String,
    pub source_type: String,
    pub representation: CodecRepresentationV1,
    pub environments: Vec<CodecEnvironmentV1>,
    pub encode_behavior: CodecBehaviorV1,
    pub decode_behavior: CodecBehaviorV1,
    pub version: u32,
    pub failure_behavior: CodecFailureBehaviorV1,
    pub classifications: CodecClassificationsV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CodecProtocolDiagnosticV1 {
    pub codec: String,
    pub reason: CodecProtocolDiagnosticReasonV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CodecProtocolDiagnosticReasonV1 {
    DuplicateCodecId,
    MissingEnvironment,
    MissingVersion,
    UnsupportedSourceType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CodecProtocolV1 {
    pub schema_version: u32,
    pub codecs: Vec<CodecProtocolRecordV1>,
    pub diagnostics: Vec<CodecProtocolDiagnosticV1>,
}

/// Produces codec declaration records and diagnoses unsupported types early.
#[must_use]
pub fn build_codec_protocol_v1(declarations: &[CodecDeclarationV1]) -> CodecProtocolV1 {
    let duplicate_ids = declarations
        .iter()
        .fold(
            (BTreeSet::new(), BTreeSet::new()),
            |(mut seen, mut duplicates), declaration| {
                if !seen.insert(declaration.id.as_str()) {
                    duplicates.insert(declaration.id.as_str());
                }
                (seen, duplicates)
            },
        )
        .1;
    let mut codecs = Vec::new();
    let mut diagnostics = Vec::new();
    for declaration in declarations {
        let classifications = classify(&declaration.source_type);
        if duplicate_ids.contains(declaration.id.as_str()) {
            diagnostics.push(diagnostic(
                declaration,
                CodecProtocolDiagnosticReasonV1::DuplicateCodecId,
            ));
        }
        if declaration.environments.is_empty() {
            diagnostics.push(diagnostic(
                declaration,
                CodecProtocolDiagnosticReasonV1::MissingEnvironment,
            ));
        }
        if declaration.version == 0 {
            diagnostics.push(diagnostic(
                declaration,
                CodecProtocolDiagnosticReasonV1::MissingVersion,
            ));
        }
        if classifications.network_serializable == CodecClassificationV1::Rejected {
            diagnostics.push(diagnostic(
                declaration,
                CodecProtocolDiagnosticReasonV1::UnsupportedSourceType,
            ));
        }
        let mut environments = declaration.environments.clone();
        environments.sort();
        environments.dedup();
        codecs.push(CodecProtocolRecordV1 {
            id: declaration.id.clone(),
            source_type: semantic_type_text(&declaration.source_type),
            representation: declaration.representation,
            environments,
            encode_behavior: declaration.encode_behavior,
            decode_behavior: declaration.decode_behavior,
            version: declaration.version,
            failure_behavior: declaration.failure_behavior,
            classifications,
        });
    }
    codecs.sort_by(|left, right| left.id.cmp(&right.id));
    diagnostics.sort_by(|left, right| {
        (left.codec.as_str(), left.reason).cmp(&(right.codec.as_str(), right.reason))
    });
    CodecProtocolV1 {
        schema_version: CODEC_PROTOCOL_SCHEMA_VERSION,
        codecs,
        diagnostics,
    }
}

fn diagnostic(
    declaration: &CodecDeclarationV1,
    reason: CodecProtocolDiagnosticReasonV1,
) -> CodecProtocolDiagnosticV1 {
    CodecProtocolDiagnosticV1 {
        codec: declaration.id.clone(),
        reason,
    }
}

fn classify(semantic_type: &SemanticType) -> CodecClassificationsV1 {
    let structural = structural_serializable(semantic_type);
    CodecClassificationsV1 {
        runtime_validated: approved(!matches!(
            semantic_type,
            SemanticType::Unknown | SemanticType::Never
        )),
        form_serializable: approved(form_serializable(semantic_type)),
        network_serializable: approved(structural),
        html_publishable: approved(html_publishable(semantic_type)),
        resume_serializable: approved(resume_value_codec(semantic_type).is_ok()),
        structured_cloneable: approved(structured_cloneable(semantic_type)),
    }
}

const fn approved(value: bool) -> CodecClassificationV1 {
    if value {
        CodecClassificationV1::Approved
    } else {
        CodecClassificationV1::Rejected
    }
}

fn structural_serializable(semantic_type: &SemanticType) -> bool {
    match semantic_type {
        SemanticType::Unknown
        | SemanticType::Never
        | SemanticType::Form
        | SemanticType::SlotContent => false,
        SemanticType::File => false,
        SemanticType::Null
        | SemanticType::Boolean
        | SemanticType::Number
        | SemanticType::String
        | SemanticType::BooleanLiteral(_)
        | SemanticType::NumberLiteral(_)
        | SemanticType::StringLiteral(_) => true,
        SemanticType::Array(item) => structural_serializable(item),
        SemanticType::Tuple(items) | SemanticType::Union(items) => {
            items.iter().all(structural_serializable)
        }
        SemanticType::Object(object) => object.properties.values().all(structural_serializable),
        SemanticType::Resource(resource) => {
            resource.serializable
                && structural_serializable(&resource.data)
                && structural_serializable(&resource.error)
        }
    }
}

fn form_serializable(semantic_type: &SemanticType) -> bool {
    match semantic_type {
        SemanticType::File => true,
        SemanticType::Array(item) => form_serializable(item),
        SemanticType::Tuple(items) | SemanticType::Union(items) => {
            items.iter().all(form_serializable)
        }
        SemanticType::Object(object) => object.properties.values().all(form_serializable),
        _ => structural_serializable(semantic_type),
    }
}

fn structured_cloneable(semantic_type: &SemanticType) -> bool {
    match semantic_type {
        SemanticType::File => true,
        SemanticType::Array(item) => structured_cloneable(item),
        SemanticType::Tuple(items) | SemanticType::Union(items) => {
            items.iter().all(structured_cloneable)
        }
        SemanticType::Object(object) => object.properties.values().all(structured_cloneable),
        SemanticType::Resource(_) => false,
        _ => structural_serializable(semantic_type),
    }
}

fn html_publishable(semantic_type: &SemanticType) -> bool {
    matches!(
        semantic_type,
        SemanticType::Null
            | SemanticType::Boolean
            | SemanticType::Number
            | SemanticType::String
            | SemanticType::BooleanLiteral(_)
            | SemanticType::NumberLiteral(_)
            | SemanticType::StringLiteral(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declaration(id: &str, source_type: SemanticType) -> CodecDeclarationV1 {
        CodecDeclarationV1 {
            id: id.into(),
            source_type,
            representation: CodecRepresentationV1::Json,
            environments: vec![CodecEnvironmentV1::Browser, CodecEnvironmentV1::Server],
            encode_behavior: CodecBehaviorV1::Fallible,
            decode_behavior: CodecBehaviorV1::Fallible,
            version: 1,
            failure_behavior: CodecFailureBehaviorV1::RejectPayload,
        }
    }

    #[test]
    fn keeps_serialization_classes_independent() {
        let protocol = build_codec_protocol_v1(&[declaration(
            "tuple",
            SemanticType::Tuple(vec![SemanticType::String, SemanticType::Number]),
        )]);
        let classifications = &protocol.codecs[0].classifications;
        assert_eq!(
            classifications.network_serializable,
            CodecClassificationV1::Approved
        );
        assert_eq!(
            classifications.resume_serializable,
            CodecClassificationV1::Rejected
        );
        assert_eq!(
            classifications.html_publishable,
            CodecClassificationV1::Rejected
        );
    }

    #[test]
    fn classifies_platform_files_without_collapsing_resume_and_form_data() {
        let protocol = build_codec_protocol_v1(&[declaration(
            "files",
            SemanticType::Array(Box::new(SemanticType::File)),
        )]);
        let classifications = &protocol.codecs[0].classifications;
        assert_eq!(
            classifications.runtime_validated,
            CodecClassificationV1::Approved
        );
        assert_eq!(
            classifications.form_serializable,
            CodecClassificationV1::Approved
        );
        assert_eq!(
            classifications.network_serializable,
            CodecClassificationV1::Rejected
        );
        assert_eq!(
            classifications.resume_serializable,
            CodecClassificationV1::Rejected
        );
        assert_eq!(
            classifications.structured_cloneable,
            CodecClassificationV1::Approved
        );
    }

    #[test]
    fn retains_codec_contract_and_diagnoses_nonserializable_types_early() {
        let mut invalid = declaration("invalid", SemanticType::Form);
        invalid.environments.clear();
        invalid.version = 0;
        let protocol = build_codec_protocol_v1(&[invalid]);
        assert_eq!(protocol.schema_version, CODEC_PROTOCOL_SCHEMA_VERSION);
        assert_eq!(
            protocol.codecs[0].encode_behavior,
            CodecBehaviorV1::Fallible
        );
        assert_eq!(protocol.diagnostics.len(), 3);
        assert!(protocol.diagnostics.iter().any(|diagnostic| {
            diagnostic.reason == CodecProtocolDiagnosticReasonV1::UnsupportedSourceType
        }));
    }
}
