//! Decorator-free V2 `slot()` field recognition at the syntax/authority boundary.

use std::collections::{BTreeMap, BTreeSet};

use presolve_parser::{ParsedFile, SourceSpan};

use crate::{
    normalize_authored_semantics_v1, AuthoredSemanticCandidateKindV1,
    AuthoredSemanticNormalizationErrorV1, AuthoredSourceRangeV1,
    CanonicalAuthoredDeclarationKindV1, CanonicalAuthoredSemanticModelV1, CanonicalIntrinsicKindV1,
    ResolvedAuthoredSemanticCandidateV1, ResolvedIntrinsicIdentityV1,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotFieldSiteV1 {
    pub subject: String,
    pub declaration_source: AuthoredSourceRangeV1,
    pub callee_source: AuthoredSourceRangeV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSlotFieldV1 {
    pub callee_source: AuthoredSourceRangeV1,
    pub slot_identity: ResolvedIntrinsicIdentityV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotFieldLoweringV1 {
    pub sites: Vec<SlotFieldSiteV1>,
    pub model: CanonicalAuthoredSemanticModelV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotFieldLoweringErrorV1 {
    ComponentSourcePathMismatch,
    UnknownComponentDeclaration { start: usize, end: usize },
    DuplicateResolution { start: usize, end: usize },
    UnknownSlotResolution { start: usize, end: usize },
    InvalidAuthoredSemantics(AuthoredSemanticNormalizationErrorV1),
}

impl std::fmt::Display for SlotFieldLoweringErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ComponentSourcePathMismatch => write!(formatter, "component and Slot authored-semantic products must describe the same source file"),
            Self::UnknownComponentDeclaration { start, end } => write!(formatter, "canonical component declaration has no source class at {start}..{end}"),
            Self::DuplicateResolution { start, end } => write!(formatter, "duplicate Slot field resolution at {start}..{end}"),
            Self::UnknownSlotResolution { start, end } => write!(formatter, "Slot field resolution has no source field callee at {start}..{end}"),
            Self::InvalidAuthoredSemantics(error) => error.fmt(formatter),
        }
    }
}
impl std::error::Error for SlotFieldLoweringErrorV1 {}

/// Select direct, zero-argument `slot()` instance-field calls owned by
/// authority-proven V2 components. The callee remains syntax until the
/// TypeScript authority resolves it to Presolve's canonical intrinsic.
pub fn slot_field_sites_v1(
    parsed: &ParsedFile,
    component_model: &CanonicalAuthoredSemanticModelV1,
) -> Result<Vec<SlotFieldSiteV1>, SlotFieldLoweringErrorV1> {
    if component_model.source_path != parsed.path {
        return Err(SlotFieldLoweringErrorV1::ComponentSourcePathMismatch);
    }
    let components = component_model
        .declarations
        .iter()
        .filter(|declaration| declaration.kind == CanonicalAuthoredDeclarationKindV1::Component)
        .map(|declaration| (declaration.subject.clone(), range_key(declaration.source)))
        .collect::<BTreeSet<_>>();
    for (subject, declaration_key) in &components {
        if !parsed.classes.iter().any(|class| class.name == *subject && range_key(range(class.span)) == *declaration_key) {
            return Err(SlotFieldLoweringErrorV1::UnknownComponentDeclaration { start: declaration_key.0, end: declaration_key.1 });
        }
    }
    let mut sites = parsed.classes.iter()
        .filter(|class| components.contains(&(class.name.clone(), range_key(range(class.span)))))
        .flat_map(|class| class.properties.iter().filter_map(move |property| {
            let call = property.initializer_call.as_ref()?;
            (!property.is_static && property.initializer.as_deref() == Some("slot(...)") && call.argument_count == 0)
                .then_some(SlotFieldSiteV1 {
                    subject: format!("{}.{}", class.name, property.name),
                    declaration_source: range(property.span),
                    callee_source: range(call.callee_span),
                })
        }))
        .collect::<Vec<_>>();
    sites.sort_by_key(|site| (site.callee_source.start, site.callee_source.end, site.subject.clone()));
    Ok(sites)
}

pub fn lower_slot_fields_v1(
    parsed: &ParsedFile,
    component_model: &CanonicalAuthoredSemanticModelV1,
    resolutions: impl IntoIterator<Item = ResolvedSlotFieldV1>,
) -> Result<SlotFieldLoweringV1, SlotFieldLoweringErrorV1> {
    let sites = slot_field_sites_v1(parsed, component_model)?;
    let known_sites = sites.iter().map(|site| range_key(site.callee_source)).collect::<BTreeSet<_>>();
    let mut resolution_by_site = BTreeMap::new();
    for resolution in resolutions {
        let key = range_key(resolution.callee_source);
        if !known_sites.contains(&key) {
            return Err(SlotFieldLoweringErrorV1::UnknownSlotResolution { start: key.0, end: key.1 });
        }
        if resolution_by_site.insert(key, resolution).is_some() {
            return Err(SlotFieldLoweringErrorV1::DuplicateResolution { start: key.0, end: key.1 });
        }
    }
    let candidates = sites.iter().filter_map(|site| {
        let resolution = resolution_by_site.get(&range_key(site.callee_source))?;
        Some(ResolvedAuthoredSemanticCandidateV1 {
            subject: site.subject.clone(),
            source: site.declaration_source,
            kind: AuthoredSemanticCandidateKindV1::ResolvedIntrinsic {
                intrinsic_kind: CanonicalIntrinsicKindV1::Slot,
                intrinsic_identity: resolution.slot_identity.clone(),
            },
        })
    });
    let model = normalize_authored_semantics_v1(parsed, candidates)
        .map_err(SlotFieldLoweringErrorV1::InvalidAuthoredSemantics)?;
    Ok(SlotFieldLoweringV1 { sites, model })
}

fn range(span: SourceSpan) -> AuthoredSourceRangeV1 {
    AuthoredSourceRangeV1 { start: span.start, end: span.end, line: span.line, column: span.column }
}
fn range_key(range: AuthoredSourceRangeV1) -> (usize, usize) { (range.start, range.end) }

#[cfg(test)]
mod tests {
    use presolve_parser::parse_file;

    use crate::{
        lower_component_inheritance_v1, CanonicalAuthoredDeclarationKindV1,
        ResolvedComponentInheritanceV1, ResolvedIntrinsicIdentityV1,
    };

    use super::{lower_slot_fields_v1, slot_field_sites_v1, ResolvedSlotFieldV1};

    fn identity(name: &str) -> ResolvedIntrinsicIdentityV1 {
        ResolvedIntrinsicIdentityV1 {
            name: name.to_owned(), flags: 32,
            declaration_modules: vec!["node_modules/presolve/src/index.d.ts".to_owned()],
        }
    }

    #[test]
    fn lowers_only_authority_proven_zero_argument_slot_fields() {
        let parsed = parse_file("src/Panel.tsx", r#"
class Panel extends Component {
  children: SlotContent = slot();
  ignored = unrelated();
  malformed: SlotContent = slot("header");
  static invalid: SlotContent = slot();
}
"#);
        let component_site = crate::component_inheritance_sites_v1(&parsed).remove(0);
        let components = lower_component_inheritance_v1(&parsed, [ResolvedComponentInheritanceV1 {
            heritage_source: component_site.heritage_source,
            component_identity: identity("Component"),
        }]).unwrap().model;
        let sites = slot_field_sites_v1(&parsed, &components).unwrap();
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].subject, "Panel.children");
        let lowered = lower_slot_fields_v1(&parsed, &components, [ResolvedSlotFieldV1 {
            callee_source: sites[0].callee_source,
            slot_identity: identity("slot"),
        }]).unwrap();
        assert_eq!(lowered.model.declarations[0].kind, CanonicalAuthoredDeclarationKindV1::Slot);
        assert_eq!(lowered.model.declarations[0].subject, "Panel.children");
    }
}
