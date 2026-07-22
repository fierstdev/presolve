use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BlockedComponentInstanceReason, ComponentInstanceId, ComponentInstancePlan,
    ComponentInvocationEntity, ComponentInvocationId, ComponentInvocationResolutionStatus,
    SemanticId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentCompositionCycle {
    pub components: Vec<SemanticId>,
    pub invocations: Vec<ComponentInvocationId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ComponentCompositionAnalysis {
    pub adjacency: BTreeMap<SemanticId, Vec<SemanticId>>,
    pub cycles: Vec<ComponentCompositionCycle>,
    pub blocked_cycle_by_instance: BTreeMap<ComponentInstanceId, usize>,
}

/// Analyze only canonical resolved definition-to-definition invocation edges.
#[must_use]
pub fn analyze_component_composition(
    components: &BTreeSet<SemanticId>,
    invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    plan: &ComponentInstancePlan,
) -> ComponentCompositionAnalysis {
    let adjacency_sets = components
        .iter()
        .map(|component| {
            let targets = invocations
                .values()
                .filter(|invocation| {
                    invocation.status == ComponentInvocationResolutionStatus::Resolved
                        && invocation.owner_component == *component
                })
                .filter_map(|invocation| invocation.target_component.clone())
                .filter(|target| components.contains(target))
                .collect::<BTreeSet<_>>();
            (component.clone(), targets)
        })
        .collect::<BTreeMap<_, _>>();
    let reverse = reverse_adjacency(&adjacency_sets);
    let mut visited = BTreeSet::new();
    let mut finish = Vec::new();
    for component in components {
        visit(component, &adjacency_sets, &mut visited, &mut finish);
    }
    visited.clear();
    let mut cycles = Vec::new();
    for component in finish.into_iter().rev() {
        if !visited.insert(component.clone()) {
            continue;
        }
        let mut members = BTreeSet::new();
        collect_component(&component, &reverse, &mut visited, &mut members);
        let self_cycle = members.len() == 1
            && adjacency_sets
                .get(&component)
                .is_some_and(|targets| targets.contains(&component));
        if members.len() > 1 || self_cycle {
            let cycle_invocations = invocations
                .values()
                .filter(|invocation| {
                    invocation.status == ComponentInvocationResolutionStatus::Resolved
                        && members.contains(&invocation.owner_component)
                        && invocation
                            .target_component
                            .as_ref()
                            .is_some_and(|target| members.contains(target))
                })
                .map(|invocation| invocation.id.clone())
                .collect();
            cycles.push(ComponentCompositionCycle {
                components: members.into_iter().collect(),
                invocations: cycle_invocations,
            });
        }
    }
    cycles.sort_by(|left, right| {
        (&left.components, &left.invocations).cmp(&(&right.components, &right.invocations))
    });
    let blocked_cycle_by_instance = plan
        .blocked
        .values()
        .filter(|blocked| {
            blocked.reason == BlockedComponentInstanceReason::CompositionCycleBoundary
        })
        .filter_map(|blocked| {
            let invocation = invocations.get(&blocked.invocation)?;
            let target = invocation.target_component.as_ref()?;
            cycles
                .iter()
                .position(|cycle| {
                    cycle.components.contains(&invocation.owner_component)
                        && cycle.components.contains(target)
                        && cycle.invocations.contains(&invocation.id)
                })
                .map(|cycle| (blocked.id.clone(), cycle))
        })
        .collect();
    let adjacency = adjacency_sets
        .into_iter()
        .map(|(source, targets)| (source, targets.into_iter().collect()))
        .collect();

    ComponentCompositionAnalysis {
        adjacency,
        cycles,
        blocked_cycle_by_instance,
    }
}

fn reverse_adjacency(
    adjacency: &BTreeMap<SemanticId, BTreeSet<SemanticId>>,
) -> BTreeMap<SemanticId, BTreeSet<SemanticId>> {
    let mut reverse = adjacency
        .keys()
        .cloned()
        .map(|component| (component, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for (source, targets) in adjacency {
        for target in targets {
            if let Some(sources) = reverse.get_mut(target) {
                sources.insert(source.clone());
            }
        }
    }
    reverse
}

fn visit(
    component: &SemanticId,
    adjacency: &BTreeMap<SemanticId, BTreeSet<SemanticId>>,
    visited: &mut BTreeSet<SemanticId>,
    finish: &mut Vec<SemanticId>,
) {
    if !visited.insert(component.clone()) {
        return;
    }
    if let Some(targets) = adjacency.get(component) {
        for target in targets {
            visit(target, adjacency, visited, finish);
        }
    }
    finish.push(component.clone());
}

fn collect_component(
    component: &SemanticId,
    reverse: &BTreeMap<SemanticId, BTreeSet<SemanticId>>,
    visited: &mut BTreeSet<SemanticId>,
    members: &mut BTreeSet<SemanticId>,
) {
    members.insert(component.clone());
    if let Some(sources) = reverse.get(component) {
        for source in sources {
            if visited.insert(source.clone()) {
                collect_component(source, reverse, visited, members);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{build_application_semantic_model, validate_application_semantic_model};

    #[test]
    fn detects_self_two_node_and_long_cycles_with_exact_boundaries() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/Cycles.tsx",
            r#"
@component("x-self") class SelfCycle extends Component { render() { return <SelfCycle />; } }
@component("x-a") class A extends Component { render() { return <B />; } }
@component("x-b") class B extends Component { render() { return <A />; } }
@component("x-c") class C extends Component { render() { return <D />; } }
@component("x-d") class D extends Component { render() { return <E />; } }
@component("x-e") class E extends Component { render() { return <C />; } }
@component("x-page") class Page extends Component { render() { return <main><SelfCycle /><A /><C /></main>; } }
"#,
        ));
        let analysis = &asm.component_composition;
        assert_eq!(analysis.cycles.len(), 3);
        assert_eq!(
            analysis
                .cycles
                .iter()
                .map(|cycle| cycle.components.len())
                .collect::<Vec<_>>(),
            vec![2, 3, 1]
        );
        assert_eq!(analysis.blocked_cycle_by_instance.len(), 3);
        assert!(asm.component_instance_plan.instances.len() < 10);
        assert!(validate_application_semantic_model(&asm).is_empty());
    }

    #[test]
    fn excludes_slot_ownership_and_unresolved_invocations_from_cycle_edges() {
        let asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/Acyclic.tsx",
            r#"
@component("x-card") class Card extends Component {
  @slot() children!: SlotContent;
  render() { return <article><slot /></article>; }
}
@component("x-page") class Page extends Component {
  render() { return <Card><Missing /></Card>; }
}
"#,
        ));
        assert!(asm.component_composition.cycles.is_empty());
        assert!(asm
            .component_composition
            .blocked_cycle_by_instance
            .is_empty());
    }

    #[test]
    fn validation_rejects_mutated_cycle_analysis() {
        let mut asm = build_application_semantic_model(&ezc_parser::parse_file(
            "src/ValidateCycle.tsx",
            r#"
@component("x-a") class A extends Component { render() { return <A />; } }
"#,
        ));
        asm.component_composition.cycles.clear();
        assert!(validate_application_semantic_model(&asm)
            .iter()
            .any(|diagnostic| diagnostic.code == "PSASM1197"));
    }
}
