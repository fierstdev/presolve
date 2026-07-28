//! Decorator-free V2 `field({...})` recognition inside canonical Forms.

use std::collections::{BTreeMap, BTreeSet};

use presolve_parser::{ParsedFile, SourceSpan};

use crate::{
    normalize_authored_semantics_v1, AuthoredSemanticCandidateKindV1,
    AuthoredSemanticNormalizationErrorV1, AuthoredSourceRangeV1,
    CanonicalAuthoredDeclarationKindV1, CanonicalAuthoredSemanticModelV1, CanonicalIntrinsicKindV1,
    ResolvedAuthoredSemanticCandidateV1, ResolvedIntrinsicIdentityV1,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFieldDefinitionSiteV1 {
    pub subject: String,
    pub owner_form_subject: String,
    pub declaration_source: AuthoredSourceRangeV1,
    pub callee_source: AuthoredSourceRangeV1,
    pub initial_source: Option<AuthoredSourceRangeV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedFormFieldValueClassificationV1 {
    FileArray,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFormFieldDefinitionV1 {
    pub callee_source: AuthoredSourceRangeV1,
    pub field_identity: ResolvedIntrinsicIdentityV1,
    pub value_classification: Option<ResolvedFormFieldValueClassificationV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFieldDefinitionLoweringV1 {
    pub sites: Vec<FormFieldDefinitionSiteV1>,
    pub model: CanonicalAuthoredSemanticModelV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormFieldDefinitionLoweringErrorV1 {
    SourcePathMismatch,
    DuplicateResolution { start: usize, end: usize },
    UnknownFieldResolution { start: usize, end: usize },
    FieldOutsideCanonicalForm { subject: String },
    InvalidAuthoredSemantics(AuthoredSemanticNormalizationErrorV1),
}

impl std::fmt::Display for FormFieldDefinitionLoweringErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourcePathMismatch => write!(
                formatter,
                "component, Form, and Form Field products must describe the same source file"
            ),
            Self::DuplicateResolution { start, end } => write!(
                formatter,
                "duplicate Form Field definition resolution at {start}..{end}"
            ),
            Self::UnknownFieldResolution { start, end } => write!(
                formatter,
                "Form Field resolution has no static source field call at {start}..{end}"
            ),
            Self::FieldOutsideCanonicalForm { subject } => write!(
                formatter,
                "resolved Form Field `{subject}` is not owned by a canonical V2 Form"
            ),
            Self::InvalidAuthoredSemantics(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for FormFieldDefinitionLoweringErrorV1 {}

/// Select nested call leaves from static object arguments of component fields.
/// The sites remain syntax-only until the outer Form and each leaf callee are
/// independently proven by TypeScript authority.
pub fn form_field_definition_sites_v1(
    parsed: &ParsedFile,
    component_model: &CanonicalAuthoredSemanticModelV1,
) -> Result<Vec<FormFieldDefinitionSiteV1>, FormFieldDefinitionLoweringErrorV1> {
    if component_model.source_path != parsed.path {
        return Err(FormFieldDefinitionLoweringErrorV1::SourcePathMismatch);
    }
    let component_names = component_model
        .declarations
        .iter()
        .filter(|declaration| declaration.kind == CanonicalAuthoredDeclarationKindV1::Component)
        .map(|declaration| declaration.subject.as_str())
        .collect::<BTreeSet<_>>();
    let mut sites = parsed
        .classes
        .iter()
        .filter(|class| component_names.contains(class.name.as_str()))
        .flat_map(|class| {
            class.properties.iter().flat_map(move |property| {
                let owner_form_subject = format!("{}.{}", class.name, property.name);
                property
                    .form_definition_shape
                    .iter()
                    .flat_map(|shape| &shape.fields)
                    .map(move |field| FormFieldDefinitionSiteV1 {
                        subject: format!("{owner_form_subject}.{}", field.path.join(".")),
                        owner_form_subject: owner_form_subject.clone(),
                        declaration_source: range(field.declaration_span),
                        callee_source: range(field.callee_span),
                        initial_source: field.initial_span.map(range),
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

pub fn lower_form_field_definitions_v1(
    parsed: &ParsedFile,
    component_model: &CanonicalAuthoredSemanticModelV1,
    form_model: &CanonicalAuthoredSemanticModelV1,
    resolutions: impl IntoIterator<Item = ResolvedFormFieldDefinitionV1>,
) -> Result<FormFieldDefinitionLoweringV1, FormFieldDefinitionLoweringErrorV1> {
    if form_model.source_path != parsed.path {
        return Err(FormFieldDefinitionLoweringErrorV1::SourcePathMismatch);
    }
    let sites = form_field_definition_sites_v1(parsed, component_model)?;
    let known_sites = sites
        .iter()
        .map(|site| range_key(site.callee_source))
        .collect::<BTreeSet<_>>();
    let forms = form_model
        .declarations
        .iter()
        .filter(|declaration| declaration.kind == CanonicalAuthoredDeclarationKindV1::Form)
        .map(|declaration| declaration.subject.as_str())
        .collect::<BTreeSet<_>>();
    let mut resolution_by_site = BTreeMap::new();
    for resolution in resolutions {
        let key = range_key(resolution.callee_source);
        if !known_sites.contains(&key) {
            return Err(FormFieldDefinitionLoweringErrorV1::UnknownFieldResolution {
                start: key.0,
                end: key.1,
            });
        }
        if resolution_by_site.insert(key, resolution).is_some() {
            return Err(FormFieldDefinitionLoweringErrorV1::DuplicateResolution {
                start: key.0,
                end: key.1,
            });
        }
    }
    let candidates = sites
        .iter()
        .filter_map(|site| {
            let resolution = resolution_by_site.get(&range_key(site.callee_source))?;
            Some((site, resolution))
        })
        .map(|(site, resolution)| {
            if !forms.contains(site.owner_form_subject.as_str()) {
                return Err(
                    FormFieldDefinitionLoweringErrorV1::FieldOutsideCanonicalForm {
                        subject: site.subject.clone(),
                    },
                );
            }
            Ok(ResolvedAuthoredSemanticCandidateV1 {
                subject: site.subject.clone(),
                source: site.declaration_source,
                kind: AuthoredSemanticCandidateKindV1::ResolvedIntrinsic {
                    intrinsic_kind: CanonicalIntrinsicKindV1::Field,
                    intrinsic_identity: resolution.field_identity.clone(),
                },
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut model = normalize_authored_semantics_v1(parsed, candidates)
        .map_err(FormFieldDefinitionLoweringErrorV1::InvalidAuthoredSemantics)?;
    for declaration in &mut model.declarations {
        let Some((_, resolution)) = sites
            .iter()
            .filter_map(|site| {
                resolution_by_site
                    .get(&range_key(site.callee_source))
                    .map(|resolution| (site, resolution))
            })
            .find(|(site, _)| site.subject == declaration.subject)
        else {
            continue;
        };
        if resolution.value_classification
            == Some(ResolvedFormFieldValueClassificationV1::FileArray)
        {
            declaration.derived_evidence =
                Some(crate::DerivedAuthoredEvidenceV2::FormFieldFileArray);
        }
    }
    Ok(FormFieldDefinitionLoweringV1 { sites, model })
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
    use std::collections::BTreeSet;

    use presolve_parser::parse_file;

    use crate::{
        lower_component_inheritance_v1, lower_form_definitions_v1, ResolvedComponentInheritanceV1,
        ResolvedFormDefinitionV1, ResolvedIntrinsicIdentityV1,
    };

    use super::{
        form_field_definition_sites_v1, lower_form_field_definitions_v1,
        ResolvedFormFieldDefinitionV1,
    };

    fn identity(name: &str) -> ResolvedIntrinsicIdentityV1 {
        ResolvedIntrinsicIdentityV1 {
            name: name.to_owned(),
            flags: 32,
            declaration_modules: vec!["presolve".to_owned()],
        }
    }

    #[test]
    fn lowers_nested_fields_only_below_authority_proven_forms() {
        let parsed = parse_file(
            "src/Profile.tsx",
            r#"class Profile extends Base {
  profile = makeForm({ fields: { name: makeField({ initial: "" }), address: { street: makeField({ initial: "" }) } } });
  lookalike = helper({ fields: { ignored: makeField({ initial: "" }) } });
}"#,
        );
        let component_site = crate::component_inheritance_sites_v1(&parsed)
            .pop()
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
        let forms = crate::form_definition_sites_v1(&parsed, &components).unwrap();
        let form_model = lower_form_definitions_v1(
            &parsed,
            &components,
            [ResolvedFormDefinitionV1 {
                callee_source: forms[0].callee_source,
                form_identity: identity("defineForm"),
            }],
        )
        .unwrap()
        .model;
        let sites = form_field_definition_sites_v1(&parsed, &components).unwrap();
        let lowering = lower_form_field_definitions_v1(
            &parsed,
            &components,
            &form_model,
            sites[..2].iter().map(|site| ResolvedFormFieldDefinitionV1 {
                callee_source: site.callee_source,
                field_identity: identity("field"),
                value_classification: None,
            }),
        )
        .unwrap();
        assert_eq!(lowering.model.declarations.len(), 2);
        let subjects = lowering
            .model
            .declarations
            .iter()
            .map(|declaration| declaration.subject.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            subjects,
            BTreeSet::from(["Profile.profile.address.street", "Profile.profile.name",])
        );
    }
}
