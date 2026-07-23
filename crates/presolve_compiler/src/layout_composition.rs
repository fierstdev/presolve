//! Compiler-owned validation and planning for ergonomic file-route layouts.

use std::collections::BTreeMap;

use crate::{ApplicationSemanticModel, FileRouteGraphV1, SemanticId, SlotKind};

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
    let components = model
        .components
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
        routes.push(LayoutCompositionRouteV1 {
            path: route.path.clone(),
            page: route.component.clone(),
            layouts: route.layouts.clone(),
        });
    }
    Ok(LayoutCompositionPlanV1 { routes })
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
    }
}
