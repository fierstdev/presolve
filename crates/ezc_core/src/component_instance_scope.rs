use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ComponentInstanceId, ComponentInstancePlan, ComponentInstanceStatus, ComponentRootId,
    ComponentStructuralRegionId, SemanticId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentInstanceScopeNode {
    pub id: ComponentInstanceId,
    pub component: SemanticId,
    pub owner_root: ComponentRootId,
    pub depth: usize,
    pub structural_region: Option<ComponentStructuralRegionId>,
    pub status: ComponentInstanceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentInstanceScopeViolation {
    UnknownParent,
    UnknownChild,
    MissingParent,
    ParentReciprocity,
    MultipleParents,
    InvalidRoot,
    DepthMismatch,
    OwnerRootMismatch,
    Cycle,
    Unreachable,
    NonCanonicalOrdering,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentInstanceScopeDiagnostic {
    pub violation: ComponentInstanceScopeViolation,
    pub instance: ComponentInstanceId,
    pub related: Option<ComponentInstanceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ComponentInstanceScopeGraph {
    pub nodes: BTreeMap<ComponentInstanceId, ComponentInstanceScopeNode>,
    pub parent_by_instance: BTreeMap<ComponentInstanceId, ComponentInstanceId>,
    pub children_by_instance: BTreeMap<ComponentInstanceId, Vec<ComponentInstanceId>>,
    pub roots: Vec<ComponentInstanceId>,
}

impl ComponentInstanceScopeGraph {
    #[must_use]
    pub fn node(&self, id: &ComponentInstanceId) -> Option<&ComponentInstanceScopeNode> {
        self.nodes.get(id)
    }

    #[must_use]
    pub fn parent(&self, id: &ComponentInstanceId) -> Option<&ComponentInstanceId> {
        self.parent_by_instance.get(id)
    }

    #[must_use]
    pub fn children(&self, id: &ComponentInstanceId) -> &[ComponentInstanceId] {
        self.children_by_instance.get(id).map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn ancestors(&self, id: &ComponentInstanceId) -> Vec<&ComponentInstanceId> {
        let mut result = Vec::new();
        let mut seen = BTreeSet::from([id]);
        let mut current = id;
        while let Some(parent) = self.parent(current) {
            if !seen.insert(parent) {
                break;
            }
            result.push(parent);
            current = parent;
        }
        result
    }
}

/// Project executable H4 instances into the sole runtime composition ancestry graph.
#[must_use]
pub fn build_component_instance_scope_graph(
    plan: &ComponentInstancePlan,
) -> ComponentInstanceScopeGraph {
    let nodes = plan
        .instances
        .values()
        .map(|instance| {
            (
                instance.id.clone(),
                ComponentInstanceScopeNode {
                    id: instance.id.clone(),
                    component: instance.component.clone(),
                    owner_root: instance.owner_root.clone(),
                    depth: instance.depth,
                    structural_region: instance.structural_region.clone(),
                    status: instance.status,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let parent_by_instance = plan
        .instances
        .values()
        .filter_map(|instance| {
            instance
                .parent_instance
                .as_ref()
                .map(|parent| (instance.id.clone(), parent.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let mut children_by_instance = nodes
        .keys()
        .cloned()
        .map(|id| (id, Vec::new()))
        .collect::<BTreeMap<_, _>>();
    for (child, parent) in &parent_by_instance {
        if let Some(children) = children_by_instance.get_mut(parent) {
            children.push(child.clone());
        }
    }
    for children in children_by_instance.values_mut() {
        children.sort();
    }
    let roots = plan
        .instances
        .values()
        .filter(|instance| instance.parent_instance.is_none())
        .map(|instance| instance.id.clone())
        .collect();

    ComponentInstanceScopeGraph {
        nodes,
        parent_by_instance,
        children_by_instance,
        roots,
    }
}

/// Validate graph reciprocity, reachability, depth, root, cycle, and ordering invariants.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn validate_component_instance_scope_graph(
    graph: &ComponentInstanceScopeGraph,
) -> Vec<ComponentInstanceScopeDiagnostic> {
    let mut diagnostics = Vec::new();
    let root_set = graph.roots.iter().cloned().collect::<BTreeSet<_>>();

    if !is_sorted_unique(&graph.roots) {
        if let Some(root) = graph.roots.first() {
            push_diagnostic(
                &mut diagnostics,
                ComponentInstanceScopeViolation::NonCanonicalOrdering,
                root,
                None,
            );
        }
    }

    for (id, node) in &graph.nodes {
        match graph.parent_by_instance.get(id) {
            None if node.depth == 0 && root_set.contains(id) => {}
            None if node.depth > 0 => push_diagnostic(
                &mut diagnostics,
                ComponentInstanceScopeViolation::MissingParent,
                id,
                None,
            ),
            None => push_diagnostic(
                &mut diagnostics,
                ComponentInstanceScopeViolation::InvalidRoot,
                id,
                None,
            ),
            Some(parent) => {
                if graph.nodes.contains_key(parent) {
                    if graph.node(parent).is_some_and(|parent_node| {
                        node.depth != parent_node.depth.saturating_add(1)
                    }) {
                        push_diagnostic(
                            &mut diagnostics,
                            ComponentInstanceScopeViolation::DepthMismatch,
                            id,
                            Some(parent),
                        );
                    }
                    if graph
                        .node(parent)
                        .is_some_and(|parent_node| node.owner_root != parent_node.owner_root)
                    {
                        push_diagnostic(
                            &mut diagnostics,
                            ComponentInstanceScopeViolation::OwnerRootMismatch,
                            id,
                            Some(parent),
                        );
                    }
                    if !graph.children(parent).contains(id) {
                        push_diagnostic(
                            &mut diagnostics,
                            ComponentInstanceScopeViolation::ParentReciprocity,
                            id,
                            Some(parent),
                        );
                    }
                } else {
                    push_diagnostic(
                        &mut diagnostics,
                        ComponentInstanceScopeViolation::UnknownParent,
                        id,
                        Some(parent),
                    );
                }
                if root_set.contains(id) {
                    push_diagnostic(
                        &mut diagnostics,
                        ComponentInstanceScopeViolation::InvalidRoot,
                        id,
                        Some(parent),
                    );
                }
            }
        }
    }

    let mut observed_parents =
        BTreeMap::<ComponentInstanceId, BTreeSet<ComponentInstanceId>>::new();
    for (parent, children) in &graph.children_by_instance {
        if !graph.nodes.contains_key(parent) {
            push_diagnostic(
                &mut diagnostics,
                ComponentInstanceScopeViolation::UnknownParent,
                parent,
                None,
            );
        }
        if !is_sorted_unique(children) {
            push_diagnostic(
                &mut diagnostics,
                ComponentInstanceScopeViolation::NonCanonicalOrdering,
                parent,
                None,
            );
        }
        for child in children {
            if !graph.nodes.contains_key(child) {
                push_diagnostic(
                    &mut diagnostics,
                    ComponentInstanceScopeViolation::UnknownChild,
                    parent,
                    Some(child),
                );
                continue;
            }
            observed_parents
                .entry(child.clone())
                .or_default()
                .insert(parent.clone());
            if graph.parent(child) != Some(parent) {
                push_diagnostic(
                    &mut diagnostics,
                    ComponentInstanceScopeViolation::ParentReciprocity,
                    child,
                    Some(parent),
                );
            }
        }
    }
    for (child, parents) in observed_parents {
        if parents.len() > 1 {
            push_diagnostic(
                &mut diagnostics,
                ComponentInstanceScopeViolation::MultipleParents,
                &child,
                parents.iter().next(),
            );
        }
    }

    for id in graph.nodes.keys() {
        if parent_chain_cycles(graph, id) {
            push_diagnostic(
                &mut diagnostics,
                ComponentInstanceScopeViolation::Cycle,
                id,
                graph.parent(id),
            );
        }
    }

    let mut reachable = BTreeSet::new();
    for root in &graph.roots {
        collect_reachable(graph, root, &mut reachable);
    }
    for id in graph.nodes.keys().filter(|id| !reachable.contains(*id)) {
        push_diagnostic(
            &mut diagnostics,
            ComponentInstanceScopeViolation::Unreachable,
            id,
            graph.parent(id),
        );
    }

    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn parent_chain_cycles(graph: &ComponentInstanceScopeGraph, id: &ComponentInstanceId) -> bool {
    let mut seen = BTreeSet::new();
    let mut current = id;
    while let Some(parent) = graph.parent(current) {
        if !seen.insert(current) || parent == id {
            return true;
        }
        current = parent;
    }
    false
}

fn collect_reachable(
    graph: &ComponentInstanceScopeGraph,
    id: &ComponentInstanceId,
    reachable: &mut BTreeSet<ComponentInstanceId>,
) {
    if !reachable.insert(id.clone()) {
        return;
    }
    for child in graph.children(id) {
        collect_reachable(graph, child, reachable);
    }
}

fn is_sorted_unique<T: Ord>(items: &[T]) -> bool {
    items.windows(2).all(|pair| pair[0] < pair[1])
}

fn push_diagnostic(
    diagnostics: &mut Vec<ComponentInstanceScopeDiagnostic>,
    violation: ComponentInstanceScopeViolation,
    instance: &ComponentInstanceId,
    related: Option<&ComponentInstanceId>,
) {
    diagnostics.push(ComponentInstanceScopeDiagnostic {
        violation,
        instance: instance.clone(),
        related: related.cloned(),
    });
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, validate_component_instance_scope_graph,
        ComponentInstanceScopeViolation,
    };

    #[test]
    fn builds_exact_repeated_instance_ancestry_and_excludes_blocked_plans() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/Scope.tsx",
            r#"
@component("x-card") class Card extends Component { render() { return <article />; } }
@component("x-shell") class Shell extends Component { render() { return <section><Card /></section>; } }
@component("x-page") class Page extends Component { render() { return <main><Shell /><Card /><Missing /></main>; } }
"#,
        ));
        let graph = &asm.component_instance_scope;

        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.roots.len(), 1);
        assert!(validate_component_instance_scope_graph(graph).is_empty());
        assert!(asm
            .blocked_component_instances()
            .iter()
            .all(|blocked| !graph.nodes.contains_key(&blocked.id)));
        let nested_card = graph
            .nodes
            .values()
            .find(|node| node.depth == 2)
            .expect("nested Card");
        let ancestors = graph.ancestors(&nested_card.id);
        assert_eq!(ancestors.len(), 2);
        assert_eq!(graph.children(ancestors[0]).len(), 1);
    }

    #[test]
    fn reports_deterministic_integrity_violations_on_mutated_graphs() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/InvalidScope.tsx",
            r#"
@component("x-child") class Child extends Component { render() { return <span />; } }
@component("x-page") class Page extends Component { render() { return <main><Child /><Child /></main>; } }
"#,
        ));
        let mut graph = asm.component_instance_scope.clone();
        let root = graph.roots[0].clone();
        let child = graph.children(&root)[0].clone();
        let sibling = graph.children(&root)[1].clone();
        graph.parent_by_instance.insert(root.clone(), child.clone());
        graph
            .children_by_instance
            .get_mut(&child)
            .unwrap()
            .push(root.clone());
        graph
            .children_by_instance
            .get_mut(&root)
            .unwrap()
            .push(child.clone());
        graph
            .children_by_instance
            .get_mut(&sibling)
            .unwrap()
            .push(child.clone());
        graph.nodes.get_mut(&child).unwrap().depth = 9;
        graph.nodes.remove(&sibling);

        let first = validate_component_instance_scope_graph(&graph);
        let second = validate_component_instance_scope_graph(&graph);
        assert_eq!(first, second);
        let violations = first
            .iter()
            .map(|diagnostic| diagnostic.violation)
            .collect::<std::collections::BTreeSet<_>>();
        assert!(violations.contains(&ComponentInstanceScopeViolation::InvalidRoot));
        assert!(violations.contains(&ComponentInstanceScopeViolation::DepthMismatch));
        assert!(violations.contains(&ComponentInstanceScopeViolation::Cycle));
        assert!(violations.contains(&ComponentInstanceScopeViolation::MultipleParents));
        assert!(violations.contains(&ComponentInstanceScopeViolation::NonCanonicalOrdering));
        assert!(violations.contains(&ComponentInstanceScopeViolation::UnknownChild));
        assert!(violations.contains(&ComponentInstanceScopeViolation::UnknownParent));
    }
}
