//! Decorator-free V2 computed-getter recognition.
//!
//! Computed getters are derived candidates, not framework intrinsics. The
//! parser supplies source-faithful getter expressions; this lowering admits
//! only the closed direct-State, call-free subset described by the V2 contract.

use std::collections::{BTreeMap, BTreeSet};

use presolve_parser::{
    ParsedComputedExpression, ParsedComputedExpressionKind, ParsedFile, SourceSpan,
};

use crate::{
    normalize_authored_semantics_v1, AuthoredSemanticCandidateKindV1,
    AuthoredSemanticNormalizationErrorV1, AuthoredSourceRangeV1,
    CanonicalAuthoredDeclarationKindV1, CanonicalAuthoredSemanticModelV1,
    ResolvedAuthoredSemanticCandidateV1,
};

/// One analysis-proven non-intrinsic computed getter site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedGetterSiteV1 {
    pub subject: String,
    pub declaration_source: AuthoredSourceRangeV1,
    pub state_dependencies: Vec<String>,
    pub computed_dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
struct ComputedGetterCandidateV1 {
    source: AuthoredSourceRangeV1,
    state_reads: BTreeSet<String>,
    computed_reads: BTreeSet<String>,
}

/// The canonical derived-getter product for one source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedGetterLoweringV1 {
    pub sites: Vec<ComputedGetterSiteV1>,
    pub model: CanonicalAuthoredSemanticModelV1,
}

/// A source-to-canonical-model mismatch while selecting derived getters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComputedGetterLoweringErrorV1 {
    SourcePathMismatch,
    UnknownComponentDeclaration { start: usize, end: usize },
    InvalidAuthoredSemantics(AuthoredSemanticNormalizationErrorV1),
}

impl std::fmt::Display for ComputedGetterLoweringErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourcePathMismatch => write!(
                formatter,
                "canonical component/State model and derived computed getter source must match"
            ),
            Self::UnknownComponentDeclaration { start, end } => write!(
                formatter,
                "canonical component declaration has no source class at {start}..{end}"
            ),
            Self::InvalidAuthoredSemantics(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ComputedGetterLoweringErrorV1 {}

/// Select only getters that have complete direct-State and pure-expression
/// proof. Unsupported getters deliberately remain ordinary JavaScript.
pub fn computed_getter_sites_v1(
    parsed: &ParsedFile,
    model: &CanonicalAuthoredSemanticModelV1,
) -> Result<Vec<ComputedGetterSiteV1>, ComputedGetterLoweringErrorV1> {
    if model.source_path != parsed.path {
        return Err(ComputedGetterLoweringErrorV1::SourcePathMismatch);
    }
    let components = model
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
            return Err(ComputedGetterLoweringErrorV1::UnknownComponentDeclaration {
                start: declaration_key.0,
                end: declaration_key.1,
            });
        }
    }

    let state_names_by_component = model
        .declarations
        .iter()
        .filter(|declaration| declaration.kind == CanonicalAuthoredDeclarationKindV1::State)
        .filter_map(|declaration| {
            let (component, name) = declaration.subject.rsplit_once('.')?;
            Some((component.to_owned(), name.to_owned()))
        })
        .fold(
            BTreeMap::<String, BTreeSet<String>>::new(),
            |mut names, (component, name)| {
                names.entry(component).or_default().insert(name);
                names
            },
        );

    let mut sites = Vec::new();
    for class in parsed
        .classes
        .iter()
        .filter(|class| components.contains(&(class.name.clone(), range_key(range(class.span)))))
    {
        let state_names = state_names_by_component
            .get(&class.name)
            .cloned()
            .unwrap_or_default();
        let getter_names = class
            .methods
            .iter()
            .filter(|method| method.is_getter && !method.is_static && !method.is_async)
            .map(|method| method.name.clone())
            .collect::<BTreeSet<_>>();
        let mut candidates = BTreeMap::new();
        for method in class
            .methods
            .iter()
            .filter(|method| method.is_getter && !method.is_static && !method.is_async)
        {
            let Some(expression) = method.computed_expression.as_ref() else {
                continue;
            };
            let mut state_reads = BTreeSet::new();
            let mut computed_reads = BTreeSet::new();
            if !collect_getter_reads(
                expression,
                &state_names,
                &getter_names,
                &mut state_reads,
                &mut computed_reads,
            ) {
                continue;
            }
            candidates.insert(
                method.name.clone(),
                ComputedGetterCandidateV1 {
                    source: range(method.span),
                    state_reads,
                    computed_reads,
                },
            );
        }
        for (name, candidate) in &candidates {
            let Some(state_dependencies) =
                transitive_state_dependencies(name, &candidates, &mut BTreeSet::new())
            else {
                continue;
            };
            if state_dependencies.is_empty() {
                continue;
            }
            sites.push(ComputedGetterSiteV1 {
                subject: format!("{}.{}", class.name, name),
                declaration_source: candidate.source,
                state_dependencies: state_dependencies
                    .into_iter()
                    .map(|name| format!("{}.{}", class.name, name))
                    .collect(),
                computed_dependencies: candidate
                    .computed_reads
                    .iter()
                    .map(|name| format!("{}.{}", class.name, name))
                    .collect(),
            });
        }
    }
    sites.sort_by_key(|site| {
        (
            site.declaration_source.start,
            site.declaration_source.end,
            site.subject.clone(),
        )
    });
    Ok(sites)
}

/// Normalize the selected analysis-derived getters through authored semantics.
pub fn lower_computed_getters_v1(
    parsed: &ParsedFile,
    model: &CanonicalAuthoredSemanticModelV1,
) -> Result<ComputedGetterLoweringV1, ComputedGetterLoweringErrorV1> {
    let sites = computed_getter_sites_v1(parsed, model)?;
    let candidates = sites
        .iter()
        .map(|site| ResolvedAuthoredSemanticCandidateV1 {
            subject: site.subject.clone(),
            source: site.declaration_source,
            kind: AuthoredSemanticCandidateKindV1::DerivedComputedGetter {
                state_dependencies: site.state_dependencies.clone(),
                computed_dependencies: site.computed_dependencies.clone(),
            },
        });
    let model = normalize_authored_semantics_v1(parsed, candidates)
        .map_err(ComputedGetterLoweringErrorV1::InvalidAuthoredSemantics)?;
    Ok(ComputedGetterLoweringV1 { sites, model })
}

fn collect_getter_reads(
    expression: &ParsedComputedExpression,
    state_names: &BTreeSet<String>,
    getter_names: &BTreeSet<String>,
    state_reads: &mut BTreeSet<String>,
    computed_reads: &mut BTreeSet<String>,
) -> bool {
    match &expression.kind {
        ParsedComputedExpressionKind::Literal(_) => true,
        ParsedComputedExpressionKind::ThisMember(name) => {
            if state_names.contains(name) {
                state_reads.insert(name.clone());
                true
            } else if getter_names.contains(name) {
                computed_reads.insert(name.clone());
                true
            } else {
                false
            }
        }
        ParsedComputedExpressionKind::MemberAccess { object, .. } => collect_getter_reads(
            object,
            state_names,
            getter_names,
            state_reads,
            computed_reads,
        ),
        ParsedComputedExpressionKind::IndexAccess { object, index } => {
            collect_getter_reads(
                object,
                state_names,
                getter_names,
                state_reads,
                computed_reads,
            ) && collect_getter_reads(
                index,
                state_names,
                getter_names,
                state_reads,
                computed_reads,
            )
        }
        ParsedComputedExpressionKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            collect_getter_reads(
                condition,
                state_names,
                getter_names,
                state_reads,
                computed_reads,
            ) && collect_getter_reads(
                when_true,
                state_names,
                getter_names,
                state_reads,
                computed_reads,
            ) && collect_getter_reads(
                when_false,
                state_names,
                getter_names,
                state_reads,
                computed_reads,
            )
        }
        ParsedComputedExpressionKind::Template { expressions, .. } => {
            expressions.iter().all(|expression| {
                collect_getter_reads(
                    expression,
                    state_names,
                    getter_names,
                    state_reads,
                    computed_reads,
                )
            })
        }
        ParsedComputedExpressionKind::Call { .. } => false,
        ParsedComputedExpressionKind::Arithmetic { left, right, .. }
        | ParsedComputedExpressionKind::Comparison { left, right, .. }
        | ParsedComputedExpressionKind::Logical { left, right, .. }
        | ParsedComputedExpressionKind::NullishCoalescing { left, right } => {
            collect_getter_reads(left, state_names, getter_names, state_reads, computed_reads)
                && collect_getter_reads(
                    right,
                    state_names,
                    getter_names,
                    state_reads,
                    computed_reads,
                )
        }
        ParsedComputedExpressionKind::Unary { operand, .. } => collect_getter_reads(
            operand,
            state_names,
            getter_names,
            state_reads,
            computed_reads,
        ),
    }
}

fn transitive_state_dependencies(
    name: &str,
    candidates: &BTreeMap<String, ComputedGetterCandidateV1>,
    visiting: &mut BTreeSet<String>,
) -> Option<BTreeSet<String>> {
    let candidate = candidates.get(name)?;
    if !visiting.insert(name.to_owned()) {
        return None;
    }
    let mut states = candidate.state_reads.clone();
    for computed in &candidate.computed_reads {
        states.extend(transitive_state_dependencies(
            computed, candidates, visiting,
        )?);
    }
    visiting.remove(name);
    Some(states)
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
    use crate::{
        lower_component_inheritance_v1, lower_state_initializers_v1,
        CanonicalAuthoredDeclarationKindV1, DerivedAuthoredEvidenceV2,
        ResolvedComponentInheritanceV1, ResolvedIntrinsicIdentityV1, ResolvedStateInitializerV1,
    };

    use super::{computed_getter_sites_v1, lower_computed_getters_v1};

    fn identity(name: &str) -> ResolvedIntrinsicIdentityV1 {
        ResolvedIntrinsicIdentityV1 {
            name: name.to_owned(),
            flags: 32,
            declaration_modules: vec!["node_modules/presolve/src/index.d.ts".to_owned()],
        }
    }

    #[test]
    fn admits_only_direct_state_call_free_getters_without_decorator_recognition() {
        let parsed = presolve_parser::parse_file(
            "src/Counter.tsx",
            r#"
class Counter extends Base {
  count = state(0);
  get doubled() { return this.count * 2; }
  get quadrupled() { return this.doubled * 2; }
  get cycleA() { return this.cycleB; }
  get cycleB() { return this.cycleA; }
  get plain() { return "ordinary"; }
  get unknown() { return this.missing; }
  get called() { return Math.abs(this.count); }
  static get staticValue() { return 1; }
}
"#,
        );
        let component_site = crate::component_inheritance_sites_v1(&parsed)
            .into_iter()
            .next()
            .expect("component site");
        let components = lower_component_inheritance_v1(
            &parsed,
            [ResolvedComponentInheritanceV1 {
                heritage_source: component_site.heritage_source,
                component_identity: identity("Component"),
            }],
        )
        .expect("canonical component")
        .model;
        let state_site = crate::state_initializer_sites_v1(&parsed, &components)
            .expect("State sites")
            .into_iter()
            .next()
            .expect("State site");
        let states = lower_state_initializers_v1(
            &parsed,
            &components,
            [ResolvedStateInitializerV1 {
                callee_source: state_site.callee_source,
                state_identity: identity("state"),
            }],
        )
        .expect("canonical State")
        .model;
        let input = crate::compose_authored_semantics_v1([components, states])
            .expect("canonical source model");

        let sites = computed_getter_sites_v1(&parsed, &input).expect("derived sites");
        assert_eq!(sites.len(), 2);
        assert_eq!(sites[0].subject, "Counter.doubled");
        assert_eq!(sites[0].state_dependencies, ["Counter.count"]);
        assert!(sites[0].computed_dependencies.is_empty());
        assert_eq!(sites[1].subject, "Counter.quadrupled");
        assert_eq!(sites[1].state_dependencies, ["Counter.count"]);
        assert_eq!(sites[1].computed_dependencies, ["Counter.doubled"]);

        let lowered = lower_computed_getters_v1(&parsed, &input).expect("derived lowering");
        assert_eq!(lowered.model.schema_version, 8);
        let declaration = &lowered.model.declarations[0];
        assert_eq!(
            declaration.kind,
            CanonicalAuthoredDeclarationKindV1::Computed
        );
        assert!(declaration.intrinsic_identity.is_none());
        assert_eq!(
            declaration.derived_evidence,
            Some(DerivedAuthoredEvidenceV2::ComputedGetter {
                state_dependencies: vec!["Counter.count".to_owned()],
                computed_dependencies: Vec::new(),
            })
        );
        assert_eq!(lowered.model.declarations[1].subject, "Counter.quadrupled");
        assert_eq!(
            lowered.model.declarations[1].derived_evidence,
            Some(DerivedAuthoredEvidenceV2::ComputedGetter {
                state_dependencies: vec!["Counter.count".to_owned()],
                computed_dependencies: vec!["Counter.doubled".to_owned()],
            })
        );
    }
}
