//! Compatibility lowering from legacy decorator sites into authored semantics.
//!
//! This module reads legacy syntax only to enumerate decorator locations and
//! their declared subjects. Framework recognition remains exclusively with a
//! resolved-identity record supplied by the TypeScript authority adapter.

use std::collections::{BTreeMap, BTreeSet};

use presolve_parser::{ParsedDecorator, ParsedFile, SourceSpan};

use crate::{
    normalize_authored_semantics_v1, AuthoredSemanticCandidateKindV1,
    AuthoredSemanticNormalizationErrorV1, AuthoredSourceRangeV1, CanonicalAuthoredSemanticModelV1,
    CanonicalIntrinsicKindV1, ResolvedAuthoredSemanticCandidateV1, ResolvedIntrinsicIdentityV1,
};

/// A legacy decorator site that a TypeScript authority client may resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyDecoratorSiteV1 {
    pub subject: String,
    pub declaration_source: AuthoredSourceRangeV1,
    pub decorator_source: AuthoredSourceRangeV1,
}

/// A TypeScript-resolved framework intrinsic attached to one legacy decorator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyDecoratorResolutionV1 {
    /// The exact legacy decorator range, used only as a cross-boundary join key.
    pub decorator_source: AuthoredSourceRangeV1,
    pub intrinsic_kind: CanonicalIntrinsicKindV1,
    pub intrinsic_identity: ResolvedIntrinsicIdentityV1,
}

/// A validated compatibility product ready for existing semantic consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyDecoratorLoweringV1 {
    pub sites: Vec<LegacyDecoratorSiteV1>,
    pub model: CanonicalAuthoredSemanticModelV1,
}

/// A bad cross-boundary join from the TypeScript adapter to source syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyDecoratorLoweringErrorV1 {
    DuplicateResolution { start: usize, end: usize },
    UnknownDecoratorResolution { start: usize, end: usize },
    InvalidAuthoredSemantics(AuthoredSemanticNormalizationErrorV1),
}

impl std::fmt::Display for LegacyDecoratorLoweringErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateResolution { start, end } => {
                write!(
                    formatter,
                    "duplicate legacy decorator resolution at {start}..{end}"
                )
            }
            Self::UnknownDecoratorResolution { start, end } => {
                write!(
                    formatter,
                    "legacy decorator resolution has no source site at {start}..{end}"
                )
            }
            Self::InvalidAuthoredSemantics(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for LegacyDecoratorLoweringErrorV1 {}

/// Enumerate every legacy decorator without assigning any framework meaning.
#[must_use]
pub fn legacy_decorator_sites_v1(parsed: &ParsedFile) -> Vec<LegacyDecoratorSiteV1> {
    let mut sites = Vec::new();
    for class in &parsed.classes {
        extend_sites(&mut sites, &class.name, class.span, &class.decorators);
        for property in &class.properties {
            extend_sites(
                &mut sites,
                &format!("{}.{}", class.name, property.name),
                property.span,
                &property.decorators,
            );
        }
        for method in &class.methods {
            let method_subject = format!("{}.{}", class.name, method.name);
            extend_sites(&mut sites, &method_subject, method.span, &method.decorators);
            for parameter in &method.parameters {
                extend_sites(
                    &mut sites,
                    &format!("{method_subject}.{}", parameter.name),
                    parameter.span,
                    &parameter.decorators,
                );
            }
        }
    }
    sites.sort_by_key(|site| {
        (
            site.decorator_source.start,
            site.decorator_source.end,
            site.subject.clone(),
        )
    });
    sites
}

/// Lower only legacy decorators already recognized by resolved symbol identity.
///
/// Unresolved decorators are intentionally ignored: this compiler does not
/// claim arbitrary decorator spellings. Every supplied resolution, however,
/// must join to exactly one parser-discovered decorator site.
pub fn lower_legacy_decorators_v1(
    parsed: &ParsedFile,
    resolutions: impl IntoIterator<Item = LegacyDecoratorResolutionV1>,
) -> Result<LegacyDecoratorLoweringV1, LegacyDecoratorLoweringErrorV1> {
    let sites = legacy_decorator_sites_v1(parsed);
    let known_sites = sites
        .iter()
        .map(|site| range_key(site.decorator_source))
        .collect::<BTreeSet<_>>();
    let mut resolution_by_site = BTreeMap::new();
    for resolution in resolutions {
        let key = range_key(resolution.decorator_source);
        if !known_sites.contains(&key) {
            return Err(LegacyDecoratorLoweringErrorV1::UnknownDecoratorResolution {
                start: key.0,
                end: key.1,
            });
        }
        if resolution_by_site.insert(key, resolution).is_some() {
            return Err(LegacyDecoratorLoweringErrorV1::DuplicateResolution {
                start: key.0,
                end: key.1,
            });
        }
    }

    let candidates = sites.iter().filter_map(|site| {
        let resolution = resolution_by_site.get(&range_key(site.decorator_source))?;
        Some(ResolvedAuthoredSemanticCandidateV1 {
            subject: site.subject.clone(),
            source: site.declaration_source,
            kind: AuthoredSemanticCandidateKindV1::ResolvedIntrinsic {
                intrinsic_kind: resolution.intrinsic_kind.clone(),
                intrinsic_identity: resolution.intrinsic_identity.clone(),
            },
        })
    });
    let model = normalize_authored_semantics_v1(parsed, candidates)
        .map_err(LegacyDecoratorLoweringErrorV1::InvalidAuthoredSemantics)?;
    Ok(LegacyDecoratorLoweringV1 { sites, model })
}

fn extend_sites(
    sites: &mut Vec<LegacyDecoratorSiteV1>,
    subject: &str,
    declaration_span: SourceSpan,
    decorators: &[ParsedDecorator],
) {
    sites.extend(decorators.iter().map(|decorator| LegacyDecoratorSiteV1 {
        subject: subject.to_owned(),
        declaration_source: range(declaration_span),
        decorator_source: range(decorator.span),
    }));
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
        CanonicalAuthoredDeclarationKindV1, CanonicalIntrinsicKindV1, ResolvedIntrinsicIdentityV1,
    };

    use super::{
        legacy_decorator_sites_v1, lower_legacy_decorators_v1, LegacyDecoratorLoweringErrorV1,
        LegacyDecoratorResolutionV1,
    };

    fn resolution(
        start: usize,
        end: usize,
        kind: CanonicalIntrinsicKindV1,
    ) -> LegacyDecoratorResolutionV1 {
        LegacyDecoratorResolutionV1 {
            decorator_source: crate::AuthoredSourceRangeV1 {
                start,
                end,
                line: 1,
                column: start + 1,
            },
            intrinsic_kind: kind,
            intrinsic_identity: ResolvedIntrinsicIdentityV1 {
                name: "frameworkExportUnderAnAlias".to_owned(),
                flags: 2_097_152,
                declaration_modules: vec!["node_modules/@presolve/framework/index.d.ts".to_owned()],
            },
        }
    }

    #[test]
    fn lowers_only_resolved_decorator_sites_not_decorator_spelling() {
        let source = r#"
@unrelatedName()
class Counter {
  @alsoUnrelated() count = 0;
  @stillUnrelated() increment() {}
}
"#;
        let parsed = parse_file("src/Counter.tsx", source);
        let sites = legacy_decorator_sites_v1(&parsed);
        assert_eq!(sites.len(), 3);

        let product = lower_legacy_decorators_v1(
            &parsed,
            [
                resolution(
                    sites[0].decorator_source.start,
                    sites[0].decorator_source.end,
                    CanonicalIntrinsicKindV1::Component,
                ),
                resolution(
                    sites[1].decorator_source.start,
                    sites[1].decorator_source.end,
                    CanonicalIntrinsicKindV1::State,
                ),
                resolution(
                    sites[2].decorator_source.start,
                    sites[2].decorator_source.end,
                    CanonicalIntrinsicKindV1::Action,
                ),
            ],
        )
        .expect("authority-resolved decorators lower into the canonical model");

        assert_eq!(product.model.declarations.len(), 3);
        assert!(product.model.declarations.iter().any(|declaration| {
            declaration.subject == "Counter"
                && declaration.kind == CanonicalAuthoredDeclarationKindV1::Component
        }));
        assert!(product.model.declarations.iter().any(|declaration| {
            declaration.subject == "Counter.count"
                && declaration.kind == CanonicalAuthoredDeclarationKindV1::State
        }));
        assert!(product.model.declarations.iter().any(|declaration| {
            declaration.subject == "Counter.increment"
                && declaration.kind == CanonicalAuthoredDeclarationKindV1::Action
        }));
    }

    #[test]
    fn rejects_resolutions_that_do_not_join_to_a_legacy_site() {
        let parsed = parse_file("src/Counter.tsx", "class Counter {}");
        let error = lower_legacy_decorators_v1(
            &parsed,
            [resolution(0, 10, CanonicalIntrinsicKindV1::Component)],
        )
        .expect_err("adapter data must join to parser syntax");

        assert!(matches!(
            error,
            LegacyDecoratorLoweringErrorV1::UnknownDecoratorResolution { .. }
        ));
    }
}
