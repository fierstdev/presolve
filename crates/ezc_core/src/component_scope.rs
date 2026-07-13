use std::collections::{BTreeMap, BTreeSet};

use crate::{ComponentNode, SemanticId};

/// Immutable compiler-owned component ancestry input for Context visibility.
///
/// Phase G populates this graph reflexively only. Future compiler-owned
/// composition semantics may add validated parent edges without changing the
/// Context resolution algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentScopeGraph {
    pub components: Vec<SemanticId>,
    pub parent_by_component: BTreeMap<SemanticId, SemanticId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentScopeDiagnostic {
    pub message: String,
}

impl ComponentScopeGraph {
    #[must_use]
    pub fn reflexive(components: &[ComponentNode]) -> Self {
        let mut component_ids = components
            .iter()
            .map(|component| component.id.clone())
            .collect::<Vec<_>>();
        component_ids.sort();
        component_ids.dedup();
        Self {
            components: component_ids,
            parent_by_component: BTreeMap::new(),
        }
    }

    /// Constructs a canonical scope graph from pre-existing compiler-owned
    /// parent facts. This is a testable seam for future composition lowering;
    /// it never infers a parent relation from imports or source structure.
    #[must_use]
    pub fn with_parent_relations(
        components: &[ComponentNode],
        parent_by_component: BTreeMap<SemanticId, SemanticId>,
    ) -> Self {
        Self {
            components: Self::reflexive(components).components,
            parent_by_component,
        }
    }

    #[must_use]
    pub fn parent_component(&self, component: &SemanticId) -> Option<&SemanticId> {
        self.parent_by_component.get(component)
    }

    /// Returns `self` followed by canonical parents, nearest first.
    #[must_use]
    pub fn ancestor_chain(&self, component: &SemanticId) -> Vec<SemanticId> {
        if !self.components.contains(component) {
            return Vec::new();
        }

        let mut chain = vec![component.clone()];
        let mut seen = BTreeSet::from([component.clone()]);
        let mut current = component;
        while let Some(parent) = self.parent_component(current) {
            if !seen.insert(parent.clone()) {
                break;
            }
            chain.push(parent.clone());
            current = parent;
        }
        chain
    }

    #[must_use]
    pub fn diagnostics(&self) -> Vec<ComponentScopeDiagnostic> {
        let component_ids = self.components.iter().collect::<BTreeSet<_>>();
        let mut diagnostics = Vec::new();

        for (child, parent) in &self.parent_by_component {
            if !component_ids.contains(child) || !component_ids.contains(parent) {
                diagnostics.push(ComponentScopeDiagnostic {
                    message: format!(
                        "component scope relation `{child}` -> `{parent}` references a missing component"
                    ),
                });
            }
        }

        for component in &self.components {
            let mut seen = BTreeSet::from([component.clone()]);
            let mut current = component;
            while let Some(parent) = self.parent_component(current) {
                if !seen.insert(parent.clone()) {
                    diagnostics.push(ComponentScopeDiagnostic {
                        message: format!("component scope graph contains a cycle at `{component}`"),
                    });
                    break;
                }
                current = parent;
            }
        }
        diagnostics.sort_by(|left, right| left.message.cmp(&right.message));
        diagnostics.dedup();
        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::ComponentScopeGraph;
    use crate::build_component_graph;

    #[test]
    fn preserves_explicit_parent_edges_without_inferring_any() {
        let parsed = ezc_parser::parse_file(
            "src/components.tsx",
            r#"
@component("x-parent")
class Parent extends Component { render() { return <main />; } }
@component("x-child")
class Child extends Component { render() { return <main />; } }
"#,
        );
        let components = build_component_graph(&parsed).components;
        let parent = components[0].id.clone();
        let child = components[1].id.clone();
        let scope = ComponentScopeGraph::with_parent_relations(
            &components,
            BTreeMap::from([(child.clone(), parent.clone())]),
        );

        assert_eq!(scope.ancestor_chain(&child), vec![child, parent]);
        assert!(scope.diagnostics().is_empty());
    }

    #[test]
    fn reports_component_scope_cycles() {
        let parsed = ezc_parser::parse_file(
            "src/components.tsx",
            r#"
@component("x-first")
class First extends Component { render() { return <main />; } }
@component("x-second")
class Second extends Component { render() { return <main />; } }
"#,
        );
        let components = build_component_graph(&parsed).components;
        let first = components[0].id.clone();
        let second = components[1].id.clone();
        let scope = ComponentScopeGraph::with_parent_relations(
            &components,
            BTreeMap::from([(first.clone(), second.clone()), (second, first)]),
        );

        assert!(!scope.diagnostics().is_empty());
    }
}
