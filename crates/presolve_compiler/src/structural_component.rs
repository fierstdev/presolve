//! TypeScript-authoritative structural component and props projection.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{semantic_type_text, ControlFlowProvenanceV1, ObjectType, SemanticId, SemanticType};

pub const STRUCTURAL_COMPONENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentInheritanceStatusV1 {
    ResolvedPresolveComponent,
    Unresolved,
    UnsupportedMixin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentPropsResolutionV1 {
    DefaultEmpty,
    Resolved(SemanticType),
    UnresolvedGeneric,
}

/// Facts supplied by the TypeScript authority; source names never decide them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralComponentFactV1 {
    pub component: SemanticId,
    pub inheritance: ComponentInheritanceStatusV1,
    pub props: ComponentPropsResolutionV1,
    pub route_root: bool,
    pub provenance: ControlFlowProvenanceV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StructuralComponentPropV1 {
    pub name: String,
    pub semantic_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StructuralComponentRecordV1 {
    pub component: SemanticId,
    pub props: Vec<StructuralComponentPropV1>,
    pub route_root: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuralComponentDiagnosticReasonV1 {
    UnknownComponent,
    DuplicateFact,
    UnresolvedInheritance,
    UnsupportedMixinInheritance,
    InvalidPropsShape,
    RouteRootUnresolvedGenericProps,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StructuralComponentDiagnosticV1 {
    pub component: SemanticId,
    pub reason: StructuralComponentDiagnosticReasonV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StructuralComponentGraphV1 {
    pub schema_version: u32,
    pub components: Vec<StructuralComponentRecordV1>,
    pub diagnostics: Vec<StructuralComponentDiagnosticV1>,
}

/// Builds records only for resolved Presolve component facts.
#[must_use]
pub fn build_structural_component_graph_v1(
    components: &[SemanticId],
    facts: &[StructuralComponentFactV1],
) -> StructuralComponentGraphV1 {
    let known = components.iter().collect::<BTreeSet<_>>();
    let duplicate_facts = facts
        .iter()
        .fold(
            (BTreeSet::new(), BTreeSet::new()),
            |(mut seen, mut duplicate), fact| {
                if !seen.insert(&fact.component) {
                    duplicate.insert(&fact.component);
                }
                (seen, duplicate)
            },
        )
        .1;
    let mut records = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for fact in facts {
        if !known.contains(&fact.component) {
            diagnostics.push(diagnostic(
                fact,
                StructuralComponentDiagnosticReasonV1::UnknownComponent,
            ));
            continue;
        }
        if duplicate_facts.contains(&fact.component) {
            diagnostics.push(diagnostic(
                fact,
                StructuralComponentDiagnosticReasonV1::DuplicateFact,
            ));
            continue;
        }
        match fact.inheritance {
            ComponentInheritanceStatusV1::Unresolved => {
                diagnostics.push(diagnostic(
                    fact,
                    StructuralComponentDiagnosticReasonV1::UnresolvedInheritance,
                ));
                continue;
            }
            ComponentInheritanceStatusV1::UnsupportedMixin => {
                diagnostics.push(diagnostic(
                    fact,
                    StructuralComponentDiagnosticReasonV1::UnsupportedMixinInheritance,
                ));
                continue;
            }
            ComponentInheritanceStatusV1::ResolvedPresolveComponent => {}
        }
        let props = match &fact.props {
            ComponentPropsResolutionV1::DefaultEmpty => Vec::new(),
            ComponentPropsResolutionV1::Resolved(SemanticType::Object(shape)) => props(shape),
            ComponentPropsResolutionV1::Resolved(_) => {
                diagnostics.push(diagnostic(
                    fact,
                    StructuralComponentDiagnosticReasonV1::InvalidPropsShape,
                ));
                continue;
            }
            ComponentPropsResolutionV1::UnresolvedGeneric => {
                if fact.route_root {
                    diagnostics.push(diagnostic(
                        fact,
                        StructuralComponentDiagnosticReasonV1::RouteRootUnresolvedGenericProps,
                    ));
                }
                continue;
            }
        };
        records.insert(
            fact.component.clone(),
            StructuralComponentRecordV1 {
                component: fact.component.clone(),
                props,
                route_root: fact.route_root,
            },
        );
    }
    diagnostics.sort_by(|left, right| {
        (&left.component, left.reason).cmp(&(&right.component, right.reason))
    });
    StructuralComponentGraphV1 {
        schema_version: STRUCTURAL_COMPONENT_SCHEMA_VERSION,
        components: records.into_values().collect(),
        diagnostics,
    }
}

fn props(shape: &ObjectType) -> Vec<StructuralComponentPropV1> {
    shape
        .properties
        .iter()
        .map(|(name, semantic_type)| StructuralComponentPropV1 {
            name: name.clone(),
            semantic_type: semantic_type_text(semantic_type),
        })
        .collect()
}

fn diagnostic(
    fact: &StructuralComponentFactV1,
    reason: StructuralComponentDiagnosticReasonV1,
) -> StructuralComponentDiagnosticV1 {
    StructuralComponentDiagnosticV1 {
        component: fact.component.clone(),
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provenance() -> ControlFlowProvenanceV1 {
        ControlFlowProvenanceV1 {
            path: "src/App.tsx".into(),
            start: 0,
            end: 1,
            line: 1,
            column: 1,
        }
    }

    #[test]
    fn projects_explicit_props_without_injecting_children() {
        let card = SemanticId::component(None, "Card");
        let props = ObjectType {
            properties: BTreeMap::from([("title".into(), SemanticType::String)]),
        };
        let graph = build_structural_component_graph_v1(
            &[card.clone()],
            &[StructuralComponentFactV1 {
                component: card,
                inheritance: ComponentInheritanceStatusV1::ResolvedPresolveComponent,
                props: ComponentPropsResolutionV1::Resolved(SemanticType::Object(props)),
                route_root: false,
                provenance: provenance(),
            }],
        );
        assert_eq!(graph.schema_version, STRUCTURAL_COMPONENT_SCHEMA_VERSION);
        assert_eq!(graph.components[0].props[0].name, "title");
        assert!(!graph.components[0]
            .props
            .iter()
            .any(|prop| prop.name == "children"));
    }

    #[test]
    fn rejects_unresolved_route_generic_props_at_the_component_boundary() {
        let page = SemanticId::component(None, "Page");
        let graph = build_structural_component_graph_v1(
            &[page.clone()],
            &[StructuralComponentFactV1 {
                component: page,
                inheritance: ComponentInheritanceStatusV1::ResolvedPresolveComponent,
                props: ComponentPropsResolutionV1::UnresolvedGeneric,
                route_root: true,
                provenance: provenance(),
            }],
        );
        assert!(graph.components.is_empty());
        assert_eq!(
            graph.diagnostics[0].reason,
            StructuralComponentDiagnosticReasonV1::RouteRootUnresolvedGenericProps
        );
    }
}
