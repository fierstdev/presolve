//! TypeScript-authoritative V2 Form validation recognition.

use std::collections::{BTreeMap, BTreeSet};

use presolve_parser::{ParsedFile, SourceSpan};

use crate::{
    normalize_authored_semantics_v1, AuthoredSemanticCandidateKindV1,
    AuthoredSemanticNormalizationErrorV1, AuthoredSourceRangeV1,
    CanonicalAuthoredDeclarationKindV1, CanonicalAuthoredSemanticModelV1, CanonicalIntrinsicKindV1,
    ResolvedAuthoredSemanticCandidateV1, ResolvedIntrinsicIdentityV1,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormValidationDefinitionSiteV1 {
    pub subject: String,
    pub owner_field_subject: String,
    pub declaration_source: AuthoredSourceRangeV1,
    pub callee_source: AuthoredSourceRangeV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFormValidationDefinitionV1 {
    pub callee_source: AuthoredSourceRangeV1,
    pub kind: ResolvedFormValidationDefinitionKindV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedFormValidationDefinitionKindV1 {
    PresolveRule {
        validation_identity: ResolvedIntrinsicIdentityV1,
    },
    StandardSchema {
        module_specifier: String,
        export_name: String,
        declaration_modules: Vec<String>,
        input_type: Option<String>,
        output_type: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormValidationDefinitionLoweringErrorV1 {
    SourcePathMismatch,
    DuplicateResolution { start: usize, end: usize },
    UnknownResolution { start: usize, end: usize },
    ValidationOutsideCanonicalField { subject: String },
    InvalidAuthoredSemantics(AuthoredSemanticNormalizationErrorV1),
}

impl std::fmt::Display for FormValidationDefinitionLoweringErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourcePathMismatch => write!(
                formatter,
                "Form Field and validation products must describe the same source file"
            ),
            Self::DuplicateResolution { start, end } => {
                write!(
                    formatter,
                    "duplicate Form validation resolution at {start}..{end}"
                )
            }
            Self::UnknownResolution { start, end } => {
                write!(
                    formatter,
                    "unknown Form validation resolution at {start}..{end}"
                )
            }
            Self::ValidationOutsideCanonicalField { subject } => write!(
                formatter,
                "resolved Form validation `{subject}` is not owned by a canonical Form Field"
            ),
            Self::InvalidAuthoredSemantics(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for FormValidationDefinitionLoweringErrorV1 {}

pub fn form_validation_definition_sites_v1(
    parsed: &ParsedFile,
    component_model: &CanonicalAuthoredSemanticModelV1,
) -> Result<Vec<FormValidationDefinitionSiteV1>, FormValidationDefinitionLoweringErrorV1> {
    if component_model.source_path != parsed.path {
        return Err(FormValidationDefinitionLoweringErrorV1::SourcePathMismatch);
    }
    let components = component_model
        .declarations
        .iter()
        .filter(|declaration| declaration.kind == CanonicalAuthoredDeclarationKindV1::Component)
        .map(|declaration| declaration.subject.as_str())
        .collect::<BTreeSet<_>>();
    let mut sites = Vec::new();
    for class in parsed
        .classes
        .iter()
        .filter(|class| components.contains(class.name.as_str()))
    {
        for property in &class.properties {
            let Some(shape) = property.form_definition_shape.as_ref() else {
                continue;
            };
            for field in &shape.fields {
                let owner = format!("{}.{}.{}", class.name, property.name, field.path.join("."));
                for (ordinal, validation) in field.validations.iter().enumerate() {
                    let Some(callee_span) = validation.callee_span else {
                        continue;
                    };
                    sites.push(FormValidationDefinitionSiteV1 {
                        subject: format!("{owner}.validation.{ordinal}"),
                        owner_field_subject: owner.clone(),
                        declaration_source: range(validation.span),
                        callee_source: range(callee_span),
                    });
                }
            }
        }
    }
    sites.sort_by_key(|site| (site.callee_source.start, site.subject.clone()));
    Ok(sites)
}

pub fn lower_form_validation_definitions_v1(
    parsed: &ParsedFile,
    component_model: &CanonicalAuthoredSemanticModelV1,
    field_model: &CanonicalAuthoredSemanticModelV1,
    resolutions: impl IntoIterator<Item = ResolvedFormValidationDefinitionV1>,
) -> Result<CanonicalAuthoredSemanticModelV1, FormValidationDefinitionLoweringErrorV1> {
    if field_model.source_path != parsed.path {
        return Err(FormValidationDefinitionLoweringErrorV1::SourcePathMismatch);
    }
    let sites = form_validation_definition_sites_v1(parsed, component_model)?;
    let known = sites
        .iter()
        .map(|site| range_key(site.callee_source))
        .collect::<BTreeSet<_>>();
    let fields = field_model
        .declarations
        .iter()
        .filter(|declaration| declaration.kind == CanonicalAuthoredDeclarationKindV1::FormField)
        .map(|declaration| declaration.subject.as_str())
        .collect::<BTreeSet<_>>();
    let mut by_site = BTreeMap::new();
    for resolution in resolutions {
        let key = range_key(resolution.callee_source);
        if !known.contains(&key) {
            return Err(FormValidationDefinitionLoweringErrorV1::UnknownResolution {
                start: key.0,
                end: key.1,
            });
        }
        if by_site.insert(key, resolution).is_some() {
            return Err(
                FormValidationDefinitionLoweringErrorV1::DuplicateResolution {
                    start: key.0,
                    end: key.1,
                },
            );
        }
    }
    let candidates = sites
        .iter()
        .filter_map(|site| {
            by_site
                .get(&range_key(site.callee_source))
                .map(|proof| (site, proof))
        })
        .map(|(site, proof)| {
            if !fields.contains(site.owner_field_subject.as_str()) {
                return Err(
                    FormValidationDefinitionLoweringErrorV1::ValidationOutsideCanonicalField {
                        subject: site.subject.clone(),
                    },
                );
            }
            let kind = match &proof.kind {
                ResolvedFormValidationDefinitionKindV1::PresolveRule {
                    validation_identity,
                } => AuthoredSemanticCandidateKindV1::ResolvedIntrinsic {
                    intrinsic_kind: CanonicalIntrinsicKindV1::Validate,
                    intrinsic_identity: validation_identity.clone(),
                },
                ResolvedFormValidationDefinitionKindV1::StandardSchema {
                    module_specifier,
                    export_name,
                    declaration_modules,
                    input_type,
                    output_type,
                } => AuthoredSemanticCandidateKindV1::DerivedStandardSchemaValidation {
                    module_specifier: module_specifier.clone(),
                    export_name: export_name.clone(),
                    declaration_modules: declaration_modules.clone(),
                    input_type: input_type.clone(),
                    output_type: output_type.clone(),
                },
            };
            Ok(ResolvedAuthoredSemanticCandidateV1 {
                subject: site.subject.clone(),
                source: site.declaration_source,
                kind,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    normalize_authored_semantics_v1(parsed, candidates)
        .map_err(FormValidationDefinitionLoweringErrorV1::InvalidAuthoredSemantics)
}

fn range(span: SourceSpan) -> AuthoredSourceRangeV1 {
    AuthoredSourceRangeV1 {
        start: span.start,
        end: span.end,
        line: span.line,
        column: span.column,
    }
}

fn range_key(range: AuthoredSourceRangeV1) -> (usize, usize) {
    (range.start, range.end)
}
