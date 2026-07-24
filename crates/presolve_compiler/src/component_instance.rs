use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ComponentInstanceId, ComponentInvocationEntity, ComponentInvocationId,
    ComponentInvocationResolutionStatus, ComponentNode, ComponentRootId,
    ComponentStructuralRegionId, SemanticId, SourceProvenance, TemplateSemanticEntity,
    TemplateSemanticKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentBuildRootKind {
    Route,
    BuildEntry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentBuildRoot {
    pub id: ComponentRootId,
    pub component: SemanticId,
    pub kind: ComponentBuildRootKind,
    pub route_path: Option<String>,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentInstanceStatus {
    Planned,
    StructuralTemplate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentInstance {
    pub id: ComponentInstanceId,
    pub component: SemanticId,
    pub invocation: Option<ComponentInvocationId>,
    pub parent_instance: Option<ComponentInstanceId>,
    pub owner_root: ComponentRootId,
    pub structural_region: Option<ComponentStructuralRegionId>,
    pub depth: usize,
    pub status: ComponentInstanceStatus,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockedComponentInstanceReason {
    UnresolvedInvocation,
    UnsupportedDynamicInvocation,
    InvalidTarget,
    CompositionCycleBoundary,
    InvalidParentPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedComponentInstancePlan {
    pub id: ComponentInstanceId,
    pub invocation: ComponentInvocationId,
    pub parent_instance: ComponentInstanceId,
    pub owner_root: ComponentRootId,
    pub target_component: Option<SemanticId>,
    pub structural_region: Option<ComponentStructuralRegionId>,
    pub depth: usize,
    pub reason: BlockedComponentInstanceReason,
    pub provenance: SourceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ComponentInstancePlan {
    pub roots: BTreeMap<ComponentRootId, ComponentBuildRoot>,
    pub instances: BTreeMap<ComponentInstanceId, ComponentInstance>,
    pub blocked: BTreeMap<ComponentInstanceId, BlockedComponentInstancePlan>,
}

/// Plan the finite statically reachable component instance topology.
#[must_use]
pub fn plan_component_instances(
    components: &[ComponentNode],
    invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    template_entities: &[TemplateSemanticEntity],
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> ComponentInstancePlan {
    plan_component_instances_with_virtual_invocations(
        components,
        invocations,
        &BTreeMap::new(),
        template_entities,
        provenance,
    )
}

/// Plans authored and compiler-issued virtual component edges through one
/// instance topology. Virtual edges are admitted only by file-route lowering.
#[must_use]
pub fn plan_component_instances_with_virtual_invocations(
    components: &[ComponentNode],
    invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    virtual_invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    template_entities: &[TemplateSemanticEntity],
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> ComponentInstancePlan {
    let mut all_invocations = invocations.clone();
    all_invocations.extend(virtual_invocations.clone());
    let roots = collect_build_roots(components, &all_invocations, provenance);
    let mut plan = ComponentInstancePlan {
        roots,
        instances: BTreeMap::new(),
        blocked: BTreeMap::new(),
    };

    for root in plan.roots.values().cloned().collect::<Vec<_>>() {
        let root_instance_id = ComponentInstanceId::for_root(&root.id);
        plan.instances.insert(
            root_instance_id.clone(),
            ComponentInstance {
                id: root_instance_id.clone(),
                component: root.component.clone(),
                invocation: None,
                parent_instance: None,
                owner_root: root.id.clone(),
                structural_region: None,
                depth: 0,
                status: ComponentInstanceStatus::Planned,
                provenance: root.provenance.clone(),
            },
        );
        expand_component_instances(
            &root.component,
            &root_instance_id,
            &root.id,
            1,
            std::slice::from_ref(&root.component),
            components,
            &all_invocations,
            template_entities,
            &mut plan,
        );
    }

    plan
}

fn collect_build_roots(
    components: &[ComponentNode],
    invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> BTreeMap<ComponentRootId, ComponentBuildRoot> {
    let routed = components
        .iter()
        .filter(|component| component.element_name.is_some() && component.route_path.is_some())
        .collect::<Vec<_>>();
    let mut root_components = if routed.is_empty() {
        let incoming = invocations
            .values()
            .filter(|invocation| invocation.status == ComponentInvocationResolutionStatus::Resolved)
            .filter_map(|invocation| invocation.target_component.clone())
            .collect::<BTreeSet<_>>();
        components
            .iter()
            .filter(|component| {
                component.element_name.is_some() && !incoming.contains(&component.id)
            })
            .collect()
    } else {
        routed
    };
    if root_components.is_empty() {
        if let Some(component) = components
            .iter()
            .filter(|component| component.element_name.is_some())
            .min_by_key(|component| component.id.as_str())
        {
            root_components.push(component);
        }
    }

    root_components
        .into_iter()
        .filter_map(|component| {
            let id = ComponentRootId::for_component(&component.id);
            let provenance = provenance.get(&component.id)?.clone();
            Some((
                id.clone(),
                ComponentBuildRoot {
                    id,
                    component: component.id.clone(),
                    kind: if component.route_path.is_some() {
                        ComponentBuildRootKind::Route
                    } else {
                        ComponentBuildRootKind::BuildEntry
                    },
                    route_path: component.route_path.clone(),
                    provenance,
                },
            ))
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn expand_component_instances(
    component: &SemanticId,
    parent_instance: &ComponentInstanceId,
    owner_root: &ComponentRootId,
    depth: usize,
    component_path: &[SemanticId],
    components: &[ComponentNode],
    invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    template_entities: &[TemplateSemanticEntity],
    plan: &mut ComponentInstancePlan,
) {
    let owned_invocations = invocations
        .values()
        .filter(|invocation| invocation.owner_component == *component)
        .collect::<Vec<_>>();

    for invocation in owned_invocations {
        let id = ComponentInstanceId::for_invocation(parent_instance, &invocation.id);
        let structural_region = enclosing_structural_region(invocation, template_entities);
        let blocked_reason = match invocation.status {
            ComponentInvocationResolutionStatus::Resolved => invocation
                .target_component
                .as_ref()
                .filter(|target| {
                    components.iter().any(|component| {
                        component.id == **target && component.element_name.is_some()
                    })
                })
                .map_or(
                    Some(BlockedComponentInstanceReason::InvalidTarget),
                    |target| {
                        component_path
                            .contains(target)
                            .then_some(BlockedComponentInstanceReason::CompositionCycleBoundary)
                    },
                ),
            ComponentInvocationResolutionStatus::UnresolvedSymbol => {
                Some(BlockedComponentInstanceReason::UnresolvedInvocation)
            }
            ComponentInvocationResolutionStatus::UnsupportedDynamicTarget => {
                Some(BlockedComponentInstanceReason::UnsupportedDynamicInvocation)
            }
            ComponentInvocationResolutionStatus::ResolvedNonComponent
            | ComponentInvocationResolutionStatus::Ambiguous => {
                Some(BlockedComponentInstanceReason::InvalidTarget)
            }
        };

        if let Some(reason) = blocked_reason {
            plan.blocked.insert(
                id.clone(),
                BlockedComponentInstancePlan {
                    id,
                    invocation: invocation.id.clone(),
                    parent_instance: parent_instance.clone(),
                    owner_root: owner_root.clone(),
                    target_component: invocation.target_component.clone(),
                    structural_region,
                    depth,
                    reason,
                    provenance: invocation.provenance.clone(),
                },
            );
            continue;
        }

        let target = invocation
            .target_component
            .as_ref()
            .expect("resolved executable invocation has a target")
            .clone();
        let status = if structural_region.is_some() {
            ComponentInstanceStatus::StructuralTemplate
        } else {
            ComponentInstanceStatus::Planned
        };
        plan.instances.insert(
            id.clone(),
            ComponentInstance {
                id: id.clone(),
                component: target.clone(),
                invocation: Some(invocation.id.clone()),
                parent_instance: Some(parent_instance.clone()),
                owner_root: owner_root.clone(),
                structural_region,
                depth,
                status,
                provenance: invocation.provenance.clone(),
            },
        );
        let mut next_path = component_path.to_vec();
        next_path.push(target.clone());
        expand_component_instances(
            &target,
            &id,
            owner_root,
            depth + 1,
            &next_path,
            components,
            invocations,
            template_entities,
            plan,
        );
    }
}

fn enclosing_structural_region(
    invocation: &ComponentInvocationEntity,
    template_entities: &[TemplateSemanticEntity],
) -> Option<ComponentStructuralRegionId> {
    let invocation_span = &invocation.provenance.span;
    template_entities
        .iter()
        .filter(|entity| {
            matches!(
                entity.kind,
                TemplateSemanticKind::Conditional | TemplateSemanticKind::List
            ) && entity.provenance.path == invocation.provenance.path
                && entity.provenance.span.start <= invocation_span.start
                && invocation_span.end <= entity.provenance.span.end
        })
        .min_by_key(|entity| entity.provenance.span.end - entity.provenance.span.start)
        .map(|entity| {
            ComponentStructuralRegionId::for_template_entity(
                &entity.id,
                match entity.kind {
                    TemplateSemanticKind::Conditional => "conditional",
                    TemplateSemanticKind::List => "keyed-list",
                    _ => unreachable!("filtered structural template entity"),
                },
            )
        })
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model, build_application_semantic_model_for_unit,
        validate_application_semantic_model, BlockedComponentInstanceReason, CompilationUnit,
        ComponentBuildRootKind, ComponentInstanceStatus, SemanticEntityKind, SemanticOwner,
    };

    #[test]
    fn plans_one_root_nested_children_and_distinct_repeated_definition_instances() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/App.tsx",
            r#"
@component("x-card") class Card extends Component { render() { return <article />; } }
@component("x-shell") class Shell extends Component { render() { return <section><Card /></section>; } }
@component("x-page") class Page extends Component { render() { return <main><Shell /><Card /><Card /></main>; } }
"#,
        ));
        let plan = &asm.component_instance_plan;

        assert_eq!(plan.roots.len(), 1);
        assert_eq!(plan.instances.len(), 5);
        assert!(plan.blocked.is_empty());
        let root = plan
            .instances
            .values()
            .find(|instance| instance.depth == 0)
            .unwrap();
        assert!(root.parent_instance.is_none());
        assert!(root.invocation.is_none());
        let card_instances = plan
            .instances
            .values()
            .filter(|instance| instance.component.as_str().ends_with("component:x-card"))
            .collect::<Vec<_>>();
        assert_eq!(card_instances.len(), 3);
        assert_eq!(
            card_instances
                .iter()
                .map(|instance| instance.id.as_str())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            3
        );
        assert!(card_instances.iter().any(|instance| instance.depth == 2));
        assert!(
            card_instances
                .iter()
                .filter(|instance| instance.depth == 1)
                .count()
                == 2
        );
        assert_eq!(
            asm.owner(root.id.as_semantic_id()),
            Some(&SemanticOwner::Application)
        );
        assert!(asm
            .entity(root.id.as_semantic_id())
            .is_some_and(|entity| entity.kind() == SemanticEntityKind::ComponentInstance));
        assert!(
            validate_application_semantic_model(&asm).is_empty(),
            "canonical instance plans should pass ASM validation"
        );
    }

    #[test]
    fn retains_invalid_target_and_cycle_expansion_boundaries() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Blocked.tsx",
            r#"
@component("x-a") class A extends Component { render() { return <B />; } }
@component("x-b") class B extends Component { render() { return <A />; } }
type Model = string;
@component("x-page") class Page extends Component { render() { return <main><A /><Missing /><Model /><Registry.Card /></main>; } }
"#,
        ));
        let blocked = asm
            .component_instance_plan
            .blocked
            .values()
            .map(|item| item.reason)
            .collect::<std::collections::BTreeSet<_>>();

        assert!(blocked.contains(&BlockedComponentInstanceReason::CompositionCycleBoundary));
        assert!(blocked.contains(&BlockedComponentInstanceReason::UnresolvedInvocation));
        assert!(blocked.contains(&BlockedComponentInstanceReason::InvalidTarget));
        assert!(blocked.contains(&BlockedComponentInstanceReason::UnsupportedDynamicInvocation));
        assert!(asm
            .component_instance_plan
            .blocked
            .values()
            .all(|item| !asm.component_instance_plan.instances.contains_key(&item.id)));
    }

    #[test]
    fn selects_one_canonical_build_root_when_only_a_cycle_exists() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Cycle.tsx",
            r#"
@component("x-b") class B extends Component { render() { return <A />; } }
@component("x-a") class A extends Component { render() { return <B />; } }
"#,
        ));

        assert_eq!(asm.component_build_roots().len(), 1);
        assert!(asm.component_build_roots()[0]
            .component
            .as_str()
            .ends_with("component:x-a"));
        assert!(asm
            .blocked_component_instances()
            .iter()
            .any(|blocked| blocked.reason
                == BlockedComponentInstanceReason::CompositionCycleBoundary));
    }

    #[test]
    fn retains_structural_instance_templates_without_eager_branch_instances() {
        let asm = build_application_semantic_model(&presolve_parser::parse_file(
            "src/Structural.tsx",
            r#"
@component("x-card") class Card extends Component { render() { return <article />; } }
@component("x-page") class Page extends Component {
  shown = state(true);
  render() { return <main>{this.shown ? <Card /> : <span />}</main>; }
}
"#,
        ));
        let structural = asm
            .component_instance_plan
            .instances
            .values()
            .find(|instance| instance.status == ComponentInstanceStatus::StructuralTemplate)
            .expect("conditional component template");

        assert!(structural.structural_region.is_some());
        assert_eq!(structural.depth, 1);
    }

    #[test]
    fn uses_route_entries_and_keeps_multi_file_instance_ids_deterministic() {
        let sources = [
            (
                "src/Card.tsx",
                r#"
@component("x-card")
export class Card extends Component { render() { return <article />; } }
"#,
            ),
            (
                "src/Page.tsx",
                r#"
import { Card } from "./Card";
@component("x-page")
@route("/")
export class Page extends Component { render() { return <main><Card /><Card /></main>; } }
"#,
            ),
        ];
        let forward =
            build_application_semantic_model_for_unit(&CompilationUnit::parse_sources(sources));
        let reverse = build_application_semantic_model_for_unit(&CompilationUnit::parse_sources([
            sources[1], sources[0],
        ]));

        assert_eq!(
            forward.component_instance_plan,
            reverse.component_instance_plan
        );
        assert_eq!(forward.component_instance_plan.roots.len(), 1);
        let root = forward
            .component_instance_plan
            .roots
            .values()
            .next()
            .unwrap();
        assert_eq!(root.kind, ComponentBuildRootKind::Route);
        assert_eq!(root.route_path.as_deref(), Some("/"));
        assert!(root.component.as_str().ends_with("component:x-page"));
        assert_eq!(forward.component_instance_plan.instances.len(), 3);
    }
}
