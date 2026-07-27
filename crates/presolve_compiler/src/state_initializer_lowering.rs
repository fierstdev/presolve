//! Decorator-free V2 `state(initial)` recognition at the syntax/authority boundary.
//!
//! Direct field-initializer calls are selected by the parser without attaching
//! framework meaning. This module admits only instance fields of already
//! authority-proven V2 components, then joins an exact resolved callee to the
//! canonical authored-semantic model.

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
pub struct StateInitializerSiteV1 {
    pub subject: String,
    pub declaration_source: AuthoredSourceRangeV1,
    pub callee_source: AuthoredSourceRangeV1,
}

/// TypeScript proof that a selected direct field-initializer callee is `state`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedStateInitializerV1 {
    pub callee_source: AuthoredSourceRangeV1,
    pub state_identity: ResolvedIntrinsicIdentityV1,
}

/// A validated canonical V2 State-recognition product.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateInitializerLoweringV1 {
    pub sites: Vec<StateInitializerSiteV1>,
    pub model: CanonicalAuthoredSemanticModelV1,
}

/// A bad component- or authority-to-source join for V2 State recognition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateInitializerLoweringErrorV1 {
    ComponentSourcePathMismatch,
    UnknownComponentDeclaration { start: usize, end: usize },
    DuplicateResolution { start: usize, end: usize },
    UnknownStateResolution { start: usize, end: usize },
    InvalidAuthoredSemantics(AuthoredSemanticNormalizationErrorV1),
}

impl std::fmt::Display for StateInitializerLoweringErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ComponentSourcePathMismatch => write!(
                formatter,
                "component and State authored-semantic products must describe the same source file"
            ),
            Self::UnknownComponentDeclaration { start, end } => write!(
                formatter,
                "canonical component declaration has no source class at {start}..{end}"
            ),
            Self::DuplicateResolution { start, end } => {
                write!(
                    formatter,
                    "duplicate State initializer resolution at {start}..{end}"
                )
            }
            Self::UnknownStateResolution { start, end } => write!(
                formatter,
                "State initializer resolution has no source field callee at {start}..{end}"
            ),
            Self::InvalidAuthoredSemantics(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for StateInitializerLoweringErrorV1 {}

/// Select direct instance-field calls owned by canonical V2 components.
pub fn state_initializer_sites_v1(
    parsed: &ParsedFile,
    component_model: &CanonicalAuthoredSemanticModelV1,
) -> Result<Vec<StateInitializerSiteV1>, StateInitializerLoweringErrorV1> {
    if component_model.source_path != parsed.path {
        return Err(StateInitializerLoweringErrorV1::ComponentSourcePathMismatch);
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
            return Err(
                StateInitializerLoweringErrorV1::UnknownComponentDeclaration {
                    start: declaration_key.0,
                    end: declaration_key.1,
                },
            );
        }
    }

    let mut sites = parsed
        .classes
        .iter()
        .filter(|class| components.contains(&(class.name.clone(), range_key(range(class.span)))))
        .flat_map(|class| {
            class.properties.iter().filter_map(move |property| {
                let call = property.initializer_call.as_ref()?;
                (!property.is_static).then_some(StateInitializerSiteV1 {
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

/// Lower only authority-proven V2 State initializer calls.
pub fn lower_state_initializers_v1(
    parsed: &ParsedFile,
    component_model: &CanonicalAuthoredSemanticModelV1,
    resolutions: impl IntoIterator<Item = ResolvedStateInitializerV1>,
) -> Result<StateInitializerLoweringV1, StateInitializerLoweringErrorV1> {
    let sites = state_initializer_sites_v1(parsed, component_model)?;
    let known_sites = sites
        .iter()
        .map(|site| range_key(site.callee_source))
        .collect::<BTreeSet<_>>();
    let mut resolution_by_site = BTreeMap::new();
    for resolution in resolutions {
        let key = range_key(resolution.callee_source);
        if !known_sites.contains(&key) {
            return Err(StateInitializerLoweringErrorV1::UnknownStateResolution {
                start: key.0,
                end: key.1,
            });
        }
        if resolution_by_site.insert(key, resolution).is_some() {
            return Err(StateInitializerLoweringErrorV1::DuplicateResolution {
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
                intrinsic_kind: CanonicalIntrinsicKindV1::State,
                intrinsic_identity: resolution.state_identity.clone(),
            },
        })
    });
    let model = normalize_authored_semantics_v1(parsed, candidates)
        .map_err(StateInitializerLoweringErrorV1::InvalidAuthoredSemantics)?;
    Ok(StateInitializerLoweringV1 { sites, model })
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
        lower_state_initializers_v1, state_initializer_sites_v1, ResolvedStateInitializerV1,
        StateInitializerLoweringErrorV1,
    };

    fn identity(name: &str) -> ResolvedIntrinsicIdentityV1 {
        ResolvedIntrinsicIdentityV1 {
            name: name.to_owned(),
            flags: 32,
            declaration_modules: vec!["node_modules/presolve/src/index.d.ts".to_owned()],
        }
    }

    #[test]
    fn lowers_only_authority_proven_instance_state_calls_of_canonical_components() {
        let source = r#"
class Counter extends Base {
  count = reactiveCell(0);
  label = ordinaryHelper("count");
  static invalid = reactiveCell(1);
}
@component() class LegacyOnly { count = reactiveCell(0); }
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
        let sites = state_initializer_sites_v1(&parsed, &components).unwrap();
        assert_eq!(sites.len(), 2);
        assert_eq!(sites[0].subject, "Counter.count");
        assert_eq!(sites[1].subject, "Counter.label");

        let model = lower_state_initializers_v1(
            &parsed,
            &components,
            [ResolvedStateInitializerV1 {
                callee_source: sites[0].callee_source,
                state_identity: identity("state"),
            }],
        )
        .unwrap();
        assert_eq!(model.model.declarations.len(), 1);
        assert_eq!(
            model.model.declarations[0].kind,
            CanonicalAuthoredDeclarationKindV1::State
        );
        assert_eq!(model.model.declarations[0].subject, "Counter.count");
    }

    #[test]
    fn rejects_dangling_state_authority_results() {
        let parsed = parse_file(
            "src/Counter.tsx",
            "class Counter extends Base { value = cell(0); }",
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
        let error = lower_state_initializers_v1(
            &parsed,
            &components,
            [ResolvedStateInitializerV1 {
                callee_source: crate::AuthoredSourceRangeV1 {
                    start: 0,
                    end: 5,
                    line: 1,
                    column: 1,
                },
                state_identity: identity("state"),
            }],
        )
        .expect_err("resolved calls must join an instance field initializer");
        assert!(matches!(
            error,
            StateInitializerLoweringErrorV1::UnknownStateResolution { .. }
        ));
    }
}
