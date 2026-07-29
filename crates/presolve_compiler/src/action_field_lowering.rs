//! Decorator-free V2 `action(handler)` field recognition.
//!
//! This is the action counterpart to State initializer lowering: the parser
//! selects direct instance-field calls, canonical component proof establishes
//! ownership, and TypeScript-resolved identity supplies the framework meaning.

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
pub struct ActionFieldSiteV1 {
    pub subject: String,
    pub declaration_source: AuthoredSourceRangeV1,
    pub callee_source: AuthoredSourceRangeV1,
}

/// TypeScript proof that a selected field-initializer callee is `action`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedActionFieldV1 {
    pub callee_source: AuthoredSourceRangeV1,
    pub action_identity: ResolvedIntrinsicIdentityV1,
    pub terminal_package_invocation: Option<ResolvedTerminalPackageInvocationV1>,
}

/// TypeScript-authoritative identity for one discarded named-import call that
/// is syntactically owned by the resolved Action handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTerminalPackageInvocationV1 {
    pub call_source: AuthoredSourceRangeV1,
    pub module_specifier: String,
    pub export_name: String,
    pub declaration_modules: Vec<String>,
}

/// A validated canonical V2 Action-recognition product.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionFieldLoweringV1 {
    pub sites: Vec<ActionFieldSiteV1>,
    pub model: CanonicalAuthoredSemanticModelV1,
}

/// A bad component- or authority-to-source join for V2 Action recognition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionFieldLoweringErrorV1 {
    ComponentSourcePathMismatch,
    UnknownComponentDeclaration { start: usize, end: usize },
    DuplicateResolution { start: usize, end: usize },
    UnknownActionResolution { start: usize, end: usize },
    InvalidAuthoredSemantics(AuthoredSemanticNormalizationErrorV1),
}

impl std::fmt::Display for ActionFieldLoweringErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ComponentSourcePathMismatch => write!(
                formatter,
                "component and Action authored-semantic products must describe the same source file"
            ),
            Self::UnknownComponentDeclaration { start, end } => write!(
                formatter,
                "canonical component declaration has no source class at {start}..{end}"
            ),
            Self::DuplicateResolution { start, end } => {
                write!(
                    formatter,
                    "duplicate Action field resolution at {start}..{end}"
                )
            }
            Self::UnknownActionResolution { start, end } => write!(
                formatter,
                "Action field resolution has no source field callee at {start}..{end}"
            ),
            Self::InvalidAuthoredSemantics(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ActionFieldLoweringErrorV1 {}

/// Select direct instance-field calls owned by canonical V2 components.
pub fn action_field_sites_v1(
    parsed: &ParsedFile,
    component_model: &CanonicalAuthoredSemanticModelV1,
) -> Result<Vec<ActionFieldSiteV1>, ActionFieldLoweringErrorV1> {
    if component_model.source_path != parsed.path {
        return Err(ActionFieldLoweringErrorV1::ComponentSourcePathMismatch);
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
            return Err(ActionFieldLoweringErrorV1::UnknownComponentDeclaration {
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
                (!property.is_static).then_some(ActionFieldSiteV1 {
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

/// Lower only authority-proven V2 Action field calls.
pub fn lower_action_fields_v1(
    parsed: &ParsedFile,
    component_model: &CanonicalAuthoredSemanticModelV1,
    resolutions: impl IntoIterator<Item = ResolvedActionFieldV1>,
) -> Result<ActionFieldLoweringV1, ActionFieldLoweringErrorV1> {
    let sites = action_field_sites_v1(parsed, component_model)?;
    let known_sites = sites
        .iter()
        .map(|site| range_key(site.callee_source))
        .collect::<BTreeSet<_>>();
    let mut resolution_by_site = BTreeMap::new();
    for resolution in resolutions {
        let key = range_key(resolution.callee_source);
        if !known_sites.contains(&key) {
            return Err(ActionFieldLoweringErrorV1::UnknownActionResolution {
                start: key.0,
                end: key.1,
            });
        }
        if resolution_by_site.insert(key, resolution).is_some() {
            return Err(ActionFieldLoweringErrorV1::DuplicateResolution {
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
                intrinsic_kind: CanonicalIntrinsicKindV1::Action,
                intrinsic_identity: resolution.action_identity.clone(),
            },
        })
    });
    let mut model = normalize_authored_semantics_v1(parsed, candidates)
        .map_err(ActionFieldLoweringErrorV1::InvalidAuthoredSemantics)?;
    for declaration in &mut model.declarations {
        let Some(site) = sites
            .iter()
            .find(|site| site.subject == declaration.subject)
        else {
            continue;
        };
        let Some(invocation) = resolution_by_site
            .get(&range_key(site.callee_source))
            .and_then(|resolution| resolution.terminal_package_invocation.as_ref())
        else {
            continue;
        };
        declaration.derived_evidence = Some(
            crate::DerivedAuthoredEvidenceV2::TerminalPackageInvocation {
                module_specifier: invocation.module_specifier.clone(),
                export_name: invocation.export_name.clone(),
                declaration_modules: invocation.declaration_modules.clone(),
            },
        );
    }
    Ok(ActionFieldLoweringV1 { sites, model })
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

    use super::{
        action_field_sites_v1, lower_action_fields_v1, ActionFieldLoweringErrorV1,
        ResolvedActionFieldV1,
    };

    fn identity(name: &str) -> ResolvedIntrinsicIdentityV1 {
        ResolvedIntrinsicIdentityV1 {
            name: name.to_owned(),
            flags: 32,
            declaration_modules: vec!["node_modules/presolve/src/index.d.ts".to_owned()],
        }
    }

    #[test]
    fn lowers_only_authority_proven_action_fields_of_canonical_components() {
        let source = r#"
class Counter extends Base {
  increment = activate(() => {});
  helper = ordinaryHelper(() => {});
  static invalid = activate(() => {});
}
@component() class LegacyOnly { increment = activate(() => {}); }
"#;
        let parsed = parse_file("src/Counter.tsx", source);
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
        let sites = action_field_sites_v1(&parsed, &components).unwrap();
        assert_eq!(sites.len(), 2);
        assert_eq!(sites[0].subject, "Counter.increment");
        assert_eq!(sites[1].subject, "Counter.helper");

        let model = lower_action_fields_v1(
            &parsed,
            &components,
            [ResolvedActionFieldV1 {
                callee_source: sites[0].callee_source,
                action_identity: identity("action"),
                terminal_package_invocation: None,
            }],
        )
        .unwrap();
        assert_eq!(model.model.declarations.len(), 1);
        assert_eq!(
            model.model.declarations[0].kind,
            CanonicalAuthoredDeclarationKindV1::Action
        );
        assert_eq!(model.model.declarations[0].subject, "Counter.increment");
    }

    #[test]
    fn rejects_dangling_action_authority_results() {
        let parsed = parse_file(
            "src/Counter.tsx",
            "class Counter extends Base { save = invoke(() => {}); }",
        );
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
        let error = lower_action_fields_v1(
            &parsed,
            &components,
            [ResolvedActionFieldV1 {
                callee_source: crate::AuthoredSourceRangeV1 {
                    start: 0,
                    end: 4,
                    line: 1,
                    column: 1,
                },
                action_identity: identity("action"),
                terminal_package_invocation: None,
            }],
        )
        .expect_err("resolved actions must join a canonical component field");
        assert!(matches!(
            error,
            ActionFieldLoweringErrorV1::UnknownActionResolution { .. }
        ));
    }
}
