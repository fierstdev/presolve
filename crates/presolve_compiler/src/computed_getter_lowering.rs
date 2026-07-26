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
        for method in class
            .methods
            .iter()
            .filter(|method| method.is_getter && !method.is_static && !method.is_async)
        {
            let Some(expression) = method.computed_expression.as_ref() else {
                continue;
            };
            let mut names = BTreeSet::new();
            if !collect_direct_state_reads(expression, &state_names, &mut names) || names.is_empty()
            {
                continue;
            }
            sites.push(ComputedGetterSiteV1 {
                subject: format!("{}.{}", class.name, method.name),
                declaration_source: range(method.span),
                state_dependencies: names
                    .into_iter()
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
            },
        });
    let model = normalize_authored_semantics_v1(parsed, candidates)
        .map_err(ComputedGetterLoweringErrorV1::InvalidAuthoredSemantics)?;
    Ok(ComputedGetterLoweringV1 { sites, model })
}

fn collect_direct_state_reads(
    expression: &ParsedComputedExpression,
    state_names: &BTreeSet<String>,
    reads: &mut BTreeSet<String>,
) -> bool {
    match &expression.kind {
        ParsedComputedExpressionKind::Literal(_) => true,
        ParsedComputedExpressionKind::ThisMember(name) => {
            if state_names.contains(name) {
                reads.insert(name.clone());
                true
            } else {
                false
            }
        }
        ParsedComputedExpressionKind::MemberAccess { object, .. } => {
            collect_direct_state_reads(object, state_names, reads)
        }
        ParsedComputedExpressionKind::IndexAccess { object, index } => {
            collect_direct_state_reads(object, state_names, reads)
                && collect_direct_state_reads(index, state_names, reads)
        }
        ParsedComputedExpressionKind::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            collect_direct_state_reads(condition, state_names, reads)
                && collect_direct_state_reads(when_true, state_names, reads)
                && collect_direct_state_reads(when_false, state_names, reads)
        }
        ParsedComputedExpressionKind::Template { expressions, .. } => expressions
            .iter()
            .all(|expression| collect_direct_state_reads(expression, state_names, reads)),
        ParsedComputedExpressionKind::Call { .. } => false,
        ParsedComputedExpressionKind::Arithmetic { left, right, .. }
        | ParsedComputedExpressionKind::Comparison { left, right, .. }
        | ParsedComputedExpressionKind::Logical { left, right, .. }
        | ParsedComputedExpressionKind::NullishCoalescing { left, right } => {
            collect_direct_state_reads(left, state_names, reads)
                && collect_direct_state_reads(right, state_names, reads)
        }
        ParsedComputedExpressionKind::Unary { operand, .. } => {
            collect_direct_state_reads(operand, state_names, reads)
        }
    }
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
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].subject, "Counter.doubled");
        assert_eq!(sites[0].state_dependencies, ["Counter.count"]);

        let lowered = lower_computed_getters_v1(&parsed, &input).expect("derived lowering");
        assert_eq!(lowered.model.schema_version, 2);
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
            })
        );
    }
}
