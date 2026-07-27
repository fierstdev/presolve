//! Decorator-free V2 component recognition at the syntax/TypeScript boundary.
//!
//! The parser selects every class heritage clause from the general source AST.
//! A TypeScript-authority client then proves which clauses resolve (directly or
//! indirectly) to Presolve's exported `Component` base.  This module joins
//! those products without looking at a base-class spelling, import path, or
//! decorator.

use std::collections::{BTreeMap, BTreeSet};

use presolve_parser::{ParsedFile, SourceSpan};

use crate::{
    normalize_authored_semantics_v1, AuthoredSemanticCandidateKindV1,
    AuthoredSemanticNormalizationErrorV1, AuthoredSourceRangeV1, CanonicalAuthoredSemanticModelV1,
    CanonicalIntrinsicKindV1, ResolvedAuthoredSemanticCandidateV1, ResolvedIntrinsicIdentityV1,
};

/// A source-AST-selected V2 class heritage clause awaiting TypeScript proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentInheritanceSiteV1 {
    pub subject: String,
    pub declaration_source: AuthoredSourceRangeV1,
    pub heritage_source: AuthoredSourceRangeV1,
}

/// TypeScript proof that a selected heritage clause resolves to `Component`.
///
/// The authority is responsible for walking indirect inheritance and aliases.
/// This compiler boundary validates only the exact source-AST join.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedComponentInheritanceV1 {
    pub heritage_source: AuthoredSourceRangeV1,
    pub component_identity: ResolvedIntrinsicIdentityV1,
}

/// A validated canonical V2 component-recognition product.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentInheritanceLoweringV1 {
    pub sites: Vec<ComponentInheritanceSiteV1>,
    pub model: CanonicalAuthoredSemanticModelV1,
}

/// A bad authority-to-source join for canonical component recognition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentInheritanceLoweringErrorV1 {
    DuplicateResolution { start: usize, end: usize },
    UnknownHeritageResolution { start: usize, end: usize },
    InvalidAuthoredSemantics(AuthoredSemanticNormalizationErrorV1),
}

impl std::fmt::Display for ComponentInheritanceLoweringErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateResolution { start, end } => {
                write!(
                    formatter,
                    "duplicate component inheritance resolution at {start}..{end}"
                )
            }
            Self::UnknownHeritageResolution { start, end } => write!(
                formatter,
                "component inheritance resolution has no source heritage clause at {start}..{end}"
            ),
            Self::InvalidAuthoredSemantics(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ComponentInheritanceLoweringErrorV1 {}

/// Select all class heritage clauses without assigning framework meaning.
#[must_use]
pub fn component_inheritance_sites_v1(parsed: &ParsedFile) -> Vec<ComponentInheritanceSiteV1> {
    let mut sites = parsed
        .classes
        .iter()
        .filter_map(|class| {
            let heritage = class.heritage.as_ref()?;
            Some(ComponentInheritanceSiteV1 {
                subject: class.name.clone(),
                declaration_source: range(class.span),
                heritage_source: range(heritage.span),
            })
        })
        .collect::<Vec<_>>();
    sites.sort_by_key(|site| {
        (
            site.heritage_source.start,
            site.heritage_source.end,
            site.subject.clone(),
        )
    });
    sites
}

/// Lower only authority-proven V2 component inheritance into canonical semantics.
///
/// An unresolved class heritage is intentionally absent rather than guessed
/// from its source spelling. Legacy decorators are not inspected here.
pub fn lower_component_inheritance_v1(
    parsed: &ParsedFile,
    resolutions: impl IntoIterator<Item = ResolvedComponentInheritanceV1>,
) -> Result<ComponentInheritanceLoweringV1, ComponentInheritanceLoweringErrorV1> {
    let sites = component_inheritance_sites_v1(parsed);
    let known_sites = sites
        .iter()
        .map(|site| range_key(site.heritage_source))
        .collect::<BTreeSet<_>>();
    let mut resolution_by_site = BTreeMap::new();
    for resolution in resolutions {
        let key = range_key(resolution.heritage_source);
        if !known_sites.contains(&key) {
            return Err(
                ComponentInheritanceLoweringErrorV1::UnknownHeritageResolution {
                    start: key.0,
                    end: key.1,
                },
            );
        }
        if resolution_by_site.insert(key, resolution).is_some() {
            return Err(ComponentInheritanceLoweringErrorV1::DuplicateResolution {
                start: key.0,
                end: key.1,
            });
        }
    }

    let candidates = sites.iter().filter_map(|site| {
        let resolution = resolution_by_site.get(&range_key(site.heritage_source))?;
        Some(ResolvedAuthoredSemanticCandidateV1 {
            subject: site.subject.clone(),
            source: site.declaration_source,
            kind: AuthoredSemanticCandidateKindV1::ResolvedIntrinsic {
                intrinsic_kind: CanonicalIntrinsicKindV1::Component,
                intrinsic_identity: resolution.component_identity.clone(),
            },
        })
    });
    let model = normalize_authored_semantics_v1(parsed, candidates)
        .map_err(ComponentInheritanceLoweringErrorV1::InvalidAuthoredSemantics)?;
    Ok(ComponentInheritanceLoweringV1 { sites, model })
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

    use crate::{CanonicalAuthoredDeclarationKindV1, ResolvedIntrinsicIdentityV1};

    use super::{
        component_inheritance_sites_v1, lower_component_inheritance_v1,
        ComponentInheritanceLoweringErrorV1, ResolvedComponentInheritanceV1,
    };

    fn resolution(start: usize, end: usize) -> ResolvedComponentInheritanceV1 {
        ResolvedComponentInheritanceV1 {
            heritage_source: crate::AuthoredSourceRangeV1 {
                start,
                end,
                line: 1,
                column: start + 1,
            },
            component_identity: ResolvedIntrinsicIdentityV1 {
                name: "Component".to_owned(),
                flags: 32,
                declaration_modules: vec!["node_modules/presolve/src/index.d.ts".to_owned()],
            },
        }
    }

    #[test]
    fn lowers_only_authority_proven_decorator_free_component_inheritance() {
        let source = r#"
import { Component as FrameworkBase } from "presolve";
class Base extends FrameworkBase {}
export class Counter extends Base { render() { return <main />; } }
@component() class LegacyOnly { render() { return <main />; } }
class NotAPresolveComponent extends Other {}
"#;
        let parsed = parse_file("app/routes/index.tsx", source);
        let sites = component_inheritance_sites_v1(&parsed);
        assert_eq!(sites.len(), 3);
        assert_eq!(sites[0].subject, "Base");
        assert_eq!(sites[1].subject, "Counter");

        let model = lower_component_inheritance_v1(
            &parsed,
            [resolution(
                sites[1].heritage_source.start,
                sites[1].heritage_source.end,
            )],
        )
        .expect("TypeScript-proven indirect inheritance is canonical");

        assert_eq!(model.model.declarations.len(), 1);
        assert_eq!(
            model.model.declarations[0].kind,
            CanonicalAuthoredDeclarationKindV1::Component
        );
        assert_eq!(model.model.declarations[0].subject, "Counter");
        assert!(model
            .model
            .declarations
            .iter()
            .all(|declaration| declaration.subject != "LegacyOnly"));
    }

    #[test]
    fn rejects_duplicate_and_dangling_authority_joins() {
        let parsed = parse_file("src/Card.tsx", "class Card extends FrameworkBase {}");
        let site = component_inheritance_sites_v1(&parsed).pop().unwrap();
        let duplicate = lower_component_inheritance_v1(
            &parsed,
            [
                resolution(site.heritage_source.start, site.heritage_source.end),
                resolution(site.heritage_source.start, site.heritage_source.end),
            ],
        )
        .expect_err("each heritage clause can have one authority result");
        assert!(matches!(
            duplicate,
            ComponentInheritanceLoweringErrorV1::DuplicateResolution { .. }
        ));

        let dangling = lower_component_inheritance_v1(&parsed, [resolution(0, 5)])
            .expect_err("authority results must join a source-AST heritage clause");
        assert!(matches!(
            dangling,
            ComponentInheritanceLoweringErrorV1::UnknownHeritageResolution { .. }
        ));
    }
}
