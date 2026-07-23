//! Compiler-owned validation and planning for ergonomic file-route layouts.

use std::collections::BTreeMap;

use crate::{
    ApplicationSemanticModel, ComponentInstancePlan, ComponentInvocationEntity,
    ComponentInvocationId, ComponentInvocationResolutionStatus, ComponentNode, FileRouteGraphV1,
    SemanticId, SlotKind, SourceProvenance, TemplatePositionId, TemplateSemanticEntity,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutCompositionPlanV1 {
    pub routes: Vec<LayoutCompositionRouteV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutCompositionRouteV1 {
    pub path: String,
    pub page: SemanticId,
    /// Ordered outermost to nearest layout.
    pub layouts: Vec<SemanticId>,
    pub edges: Vec<LayoutCompositionEdgeV1>,
}

/// One compiler-issued default-Slot child edge. It is deliberately not an
/// authored JSX invocation and carries no source-content fragment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutCompositionEdgeV1 {
    pub invocation: ComponentInvocationId,
    pub caller: SemanticId,
    pub callee: SemanticId,
    pub position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutCompositionErrorV1 {
    pub code: &'static str,
    pub message: String,
}

/// Validates automatic default-Slot composition without changing the authored
/// component instance plan. Instance/Slot lowering consumes this plan later.
pub fn build_layout_composition_plan_v1(
    model: &ApplicationSemanticModel,
    graph: &FileRouteGraphV1,
) -> Result<LayoutCompositionPlanV1, LayoutCompositionErrorV1> {
    build_layout_composition_plan_from_components_v1(&model.components, graph)
}

/// Component-fact variant used while assembling a file-route application
/// model, before instance/runtime products exist.
pub fn build_layout_composition_plan_from_components_v1(
    component_nodes: &[crate::ComponentNode],
    graph: &FileRouteGraphV1,
) -> Result<LayoutCompositionPlanV1, LayoutCompositionErrorV1> {
    let components = component_nodes
        .iter()
        .map(|component| (component.id.clone(), component))
        .collect::<BTreeMap<_, _>>();
    let mut routes = Vec::new();
    for route in &graph.routes {
        for layout in &route.layouts {
            let Some(component) = components.get(layout) else {
                return Err(LayoutCompositionErrorV1 {
                    code: "PSROUTE1022_LAYOUT_COMPOSITION_UNSUPPORTED",
                    message: format!("missing layout component {layout}"),
                });
            };
            let defaults = component
                .slot_declarations
                .iter()
                .filter(|slot| slot.kind == SlotKind::Default)
                .count();
            match defaults {
                0 => {
                    return Err(LayoutCompositionErrorV1 {
                        code: "PSROUTE1020_LAYOUT_DEFAULT_SLOT_MISSING",
                        message: format!("layout component {layout} has no default @slot()"),
                    })
                }
                1 => {}
                _ => {
                    return Err(LayoutCompositionErrorV1 {
                        code: "PSROUTE1021_LAYOUT_DEFAULT_SLOT_AMBIGUOUS",
                        message: format!(
                            "layout component {layout} has multiple default @slot() declarations"
                        ),
                    })
                }
            }
        }
        let mut chain = route.layouts.clone();
        chain.push(route.component.clone());
        let edges = chain
            .windows(2)
            .enumerate()
            .map(|(position, pair)| LayoutCompositionEdgeV1 {
                invocation: ComponentInvocationId::for_layout_composition(
                    &pair[0],
                    &route.component,
                    position,
                ),
                caller: pair[0].clone(),
                callee: pair[1].clone(),
                position,
            })
            .collect();
        routes.push(LayoutCompositionRouteV1 {
            path: route.path.clone(),
            page: route.component.clone(),
            layouts: route.layouts.clone(),
            edges,
        });
    }
    Ok(LayoutCompositionPlanV1 { routes })
}

/// Converts compiler-issued layout edges into the same resolved invocation
/// facts consumed by component instance planning. No authored JSX or source
/// content fragment is fabricated.
#[must_use]
pub fn layout_composition_virtual_invocations_v1(
    model: &ApplicationSemanticModel,
    plan: &LayoutCompositionPlanV1,
) -> BTreeMap<ComponentInvocationId, ComponentInvocationEntity> {
    plan.routes
        .iter()
        .flat_map(|route| route.edges.iter())
        .filter_map(|edge| {
            let provenance = model.provenance.get(&edge.caller)?.clone();
            let template_entity = edge.caller.template();
            Some((
                edge.invocation.clone(),
                ComponentInvocationEntity {
                    id: edge.invocation.clone(),
                    owner_component: edge.caller.clone(),
                    target_component: Some(edge.callee.clone()),
                    authored_symbol: "<presolve-layout-child>".into(),
                    template_entity: template_entity.clone(),
                    source_position: TemplatePositionId::for_template_entity(&template_entity),
                    status: ComponentInvocationResolutionStatus::Resolved,
                    provenance,
                    virtual_layout_composition: true,
                },
            ))
        })
        .collect()
}

/// Produces the canonical composed instance topology for a file-route source
/// set. Generic application-model builders retain authored-only planning.
pub fn plan_file_route_component_instances_v1(
    components: &[ComponentNode],
    graph: &FileRouteGraphV1,
    authored_invocations: &BTreeMap<ComponentInvocationId, ComponentInvocationEntity>,
    template_entities: &[TemplateSemanticEntity],
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> Result<ComponentInstancePlan, LayoutCompositionErrorV1> {
    let plan = build_layout_composition_plan_from_components_v1(components, graph)?;
    let virtuals = virtual_invocations_from_provenance(&plan, provenance);
    Ok(crate::plan_component_instances_with_virtual_invocations(
        components,
        authored_invocations,
        &virtuals,
        template_entities,
        provenance,
    ))
}

fn virtual_invocations_from_provenance(
    plan: &LayoutCompositionPlanV1,
    provenance: &BTreeMap<SemanticId, SourceProvenance>,
) -> BTreeMap<ComponentInvocationId, ComponentInvocationEntity> {
    plan.routes
        .iter()
        .flat_map(|route| route.edges.iter())
        .filter_map(|edge| {
            let provenance = provenance.get(&edge.caller)?.clone();
            let template_entity = edge.caller.template();
            Some((
                edge.invocation.clone(),
                ComponentInvocationEntity {
                    id: edge.invocation.clone(),
                    owner_component: edge.caller.clone(),
                    target_component: Some(edge.callee.clone()),
                    authored_symbol: "<presolve-layout-child>".into(),
                    template_entity: template_entity.clone(),
                    source_position: TemplatePositionId::for_template_entity(&template_entity),
                    status: ComponentInvocationResolutionStatus::Resolved,
                    provenance,
                    virtual_layout_composition: true,
                },
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model_for_unit, build_validated_file_route_graph_v1,
        CompilationUnit,
    };

    use super::*;

    #[test]
    fn requires_an_exact_default_slot_for_each_conventional_layout() {
        let model = build_application_semantic_model_for_unit(&CompilationUnit::parse_sources([
            (
                "app/layout.tsx",
                r#"@component() class Shell extends Component { @slot() children!: SlotContent; render() { return <main><slot /></main>; } }"#,
            ),
            (
                "app/routes/index.tsx",
                r#"@component() class Home extends Component { render() { return <p>Home</p>; } }"#,
            ),
        ]));
        let graph = build_validated_file_route_graph_v1(&model).unwrap();
        let plan = build_layout_composition_plan_v1(&model, &graph).unwrap();
        assert_eq!(plan.routes[0].layouts.len(), 1);
        assert_eq!(plan.routes[0].edges.len(), 1);
        assert_eq!(plan.routes[0].edges[0].caller, plan.routes[0].layouts[0]);
        assert_eq!(plan.routes[0].edges[0].callee, plan.routes[0].page);
        let virtuals = layout_composition_virtual_invocations_v1(&model, &plan);
        assert_eq!(virtuals.len(), 1);
        assert!(virtuals
            .values()
            .all(|invocation| invocation.virtual_layout_composition));
    }
}
