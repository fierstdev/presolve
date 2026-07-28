//! Decorator-free V2 `defineForm({...})` field recognition.
//!
//! The parser selects direct instance-field initializer calls without assigning
//! framework meaning. Canonical component proof establishes ownership and the
//! TypeScript authority proves that the selected callee is Presolve's exported
//! `defineForm` intrinsic.

use std::collections::{BTreeMap, BTreeSet};

use presolve_parser::{ParsedFile, SourceSpan};

use crate::{
    normalize_authored_semantics_v1, AuthoredSemanticCandidateKindV1,
    AuthoredSemanticNormalizationErrorV1, AuthoredSourceRangeV1,
    CanonicalAuthoredDeclarationKindV1, CanonicalAuthoredSemanticModelV1, CanonicalIntrinsicKindV1,
    ResolvedAuthoredSemanticCandidateV1, ResolvedIntrinsicIdentityV1,
};

/// A direct instance-field initializer call on an authority-proven component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormDefinitionSiteV1 {
    pub subject: String,
    pub declaration_source: AuthoredSourceRangeV1,
    pub callee_source: AuthoredSourceRangeV1,
}

/// TypeScript proof that a selected field initializer is `defineForm`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFormDefinitionV1 {
    pub callee_source: AuthoredSourceRangeV1,
    pub form_identity: ResolvedIntrinsicIdentityV1,
}

/// A validated canonical V2 Form-recognition product.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormDefinitionLoweringV1 {
    pub sites: Vec<FormDefinitionSiteV1>,
    pub model: CanonicalAuthoredSemanticModelV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormDefinitionLoweringErrorV1 {
    ComponentSourcePathMismatch,
    UnknownComponentDeclaration { start: usize, end: usize },
    DuplicateResolution { start: usize, end: usize },
    UnknownFormResolution { start: usize, end: usize },
    InvalidAuthoredSemantics(AuthoredSemanticNormalizationErrorV1),
}

impl std::fmt::Display for FormDefinitionLoweringErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ComponentSourcePathMismatch => write!(
                formatter,
                "component and Form authored-semantic products must describe the same source file"
            ),
            Self::UnknownComponentDeclaration { start, end } => write!(
                formatter,
                "canonical component declaration has no source class at {start}..{end}"
            ),
            Self::DuplicateResolution { start, end } => {
                write!(
                    formatter,
                    "duplicate Form definition resolution at {start}..{end}"
                )
            }
            Self::UnknownFormResolution { start, end } => write!(
                formatter,
                "Form definition resolution has no source field callee at {start}..{end}"
            ),
            Self::InvalidAuthoredSemantics(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for FormDefinitionLoweringErrorV1 {}

/// Select direct instance-field calls owned by canonical V2 components.
pub fn form_definition_sites_v1(
    parsed: &ParsedFile,
    component_model: &CanonicalAuthoredSemanticModelV1,
) -> Result<Vec<FormDefinitionSiteV1>, FormDefinitionLoweringErrorV1> {
    if component_model.source_path != parsed.path {
        return Err(FormDefinitionLoweringErrorV1::ComponentSourcePathMismatch);
    }
    let components = component_model
        .declarations
        .iter()
        .filter(|declaration| declaration.kind == CanonicalAuthoredDeclarationKindV1::Component)
        .map(|declaration| (declaration.subject.clone(), range_key(declaration.source)))
        .collect::<BTreeSet<_>>();
    for (subject, declaration_key) in &components {
        if !parsed
            .classes
            .iter()
            .any(|class| class.name == *subject && range_key(range(class.span)) == *declaration_key)
        {
            return Err(FormDefinitionLoweringErrorV1::UnknownComponentDeclaration {
                start: declaration_key.0,
                end: declaration_key.1,
            });
        }
    }

    let mut sites = parsed
        .classes
        .iter()
        .filter(|class| components.contains(&(class.name.clone(), range_key(range(class.span)))))
        .flat_map(|class| {
            class.properties.iter().filter_map(move |property| {
                let call = property.initializer_call.as_ref()?;
                (!property.is_static).then_some(FormDefinitionSiteV1 {
                    subject: format!("{}.{}", class.name, property.name),
                    declaration_source: range(property.span),
                    callee_source: range(call.callee_span),
                })
            })
        })
        .collect::<Vec<_>>();
    sites.sort_by_key(|site| {
        (
            site.callee_source.start,
            site.callee_source.end,
            site.subject.clone(),
        )
    });
    Ok(sites)
}

/// Lower only authority-proven V2 `defineForm` calls.
pub fn lower_form_definitions_v1(
    parsed: &ParsedFile,
    component_model: &CanonicalAuthoredSemanticModelV1,
    resolutions: impl IntoIterator<Item = ResolvedFormDefinitionV1>,
) -> Result<FormDefinitionLoweringV1, FormDefinitionLoweringErrorV1> {
    let sites = form_definition_sites_v1(parsed, component_model)?;
    let known_sites = sites
        .iter()
        .map(|site| range_key(site.callee_source))
        .collect::<BTreeSet<_>>();
    let mut resolution_by_site = BTreeMap::new();
    for resolution in resolutions {
        let key = range_key(resolution.callee_source);
        if !known_sites.contains(&key) {
            return Err(FormDefinitionLoweringErrorV1::UnknownFormResolution {
                start: key.0,
                end: key.1,
            });
        }
        if resolution_by_site.insert(key, resolution).is_some() {
            return Err(FormDefinitionLoweringErrorV1::DuplicateResolution {
                start: key.0,
                end: key.1,
            });
        }
    }
    let candidates = sites.iter().filter_map(|site| {
        let resolution = resolution_by_site.get(&range_key(site.callee_source))?;
        Some(ResolvedAuthoredSemanticCandidateV1 {
            subject: site.subject.clone(),
            source: site.declaration_source,
            kind: AuthoredSemanticCandidateKindV1::ResolvedIntrinsic {
                intrinsic_kind: CanonicalIntrinsicKindV1::Form,
                intrinsic_identity: resolution.form_identity.clone(),
            },
        })
    });
    let model = normalize_authored_semantics_v1(parsed, candidates)
        .map_err(FormDefinitionLoweringErrorV1::InvalidAuthoredSemantics)?;
    Ok(FormDefinitionLoweringV1 { sites, model })
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

#[cfg(test)]
mod tests {
    use presolve_parser::parse_file;

    use crate::{
        lower_component_inheritance_v1, CanonicalAuthoredDeclarationKindV1,
        ResolvedComponentInheritanceV1, ResolvedIntrinsicIdentityV1,
    };

    use super::{form_definition_sites_v1, lower_form_definitions_v1, ResolvedFormDefinitionV1};

    fn identity(name: &str) -> ResolvedIntrinsicIdentityV1 {
        ResolvedIntrinsicIdentityV1 {
            name: name.to_owned(),
            flags: 32,
            declaration_modules: vec!["node_modules/presolve/src/index.d.ts".to_owned()],
        }
    }

    #[test]
    fn lowers_only_authority_proven_form_definitions() {
        let source = r#"
class Profile extends Base {
  profile = createForm({ fields: {} });
  lookalike = helper({ fields: {} });
}
"#;
        let parsed = parse_file("src/Profile.tsx", source);
        let component_site = crate::component_inheritance_sites_v1(&parsed)
            .into_iter()
            .next()
            .unwrap();
        let components = lower_component_inheritance_v1(
            &parsed,
            [ResolvedComponentInheritanceV1 {
                heritage_source: component_site.heritage_source,
                component_identity: identity("Component"),
            }],
        )
        .unwrap()
        .model;
        let sites = form_definition_sites_v1(&parsed, &components).unwrap();
        let lowering = lower_form_definitions_v1(
            &parsed,
            &components,
            [ResolvedFormDefinitionV1 {
                callee_source: sites[0].callee_source,
                form_identity: identity("defineForm"),
            }],
        )
        .unwrap();
        assert_eq!(lowering.model.declarations.len(), 1);
        assert_eq!(
            lowering.model.declarations[0].kind,
            CanonicalAuthoredDeclarationKindV1::Form
        );
        assert_eq!(lowering.model.declarations[0].subject, "Profile.profile");
    }
}
