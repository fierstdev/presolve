//! Compiler-owned route server-action planning over closed package facts.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    BindingTable, ComponentNode, FileRouteGraphV1, ImportBindingTarget, SemanticPackageKind,
    SemanticPackageServerAction,
};

pub const ROUTE_SERVER_ACTION_PLAN_SCHEMA_VERSION: u32 = 1;

#[must_use]
pub fn route_server_action_plan_json_v1(plan: &RouteServerActionPlanV1) -> String {
    serde_json::to_string_pretty(plan).expect("route server-action plan serializes") + "\n"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteServerActionPlanErrorV1 {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteServerActionPlanV1 {
    pub schema_version: u32,
    pub routes: Vec<RouteServerActionRouteV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteServerActionRouteV1 {
    pub path: String,
    pub page_component_id: String,
    pub actions: Vec<RouteServerActionBindingV1>,
}

/// One fully resolved server action. This handoff deliberately records only
/// the closed package capability; application method bodies are never server
/// code and package implementation source is never inspected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteServerActionBindingV1 {
    pub id: String,
    pub component_id: String,
    pub method: String,
    pub package: String,
    pub version: String,
    pub integrity: String,
    pub export: String,
    pub runtime_module: String,
    pub type_signature: String,
    pub resume_policy: String,
    pub input: String,
    pub response: String,
    pub failure: String,
}

/// Resolves route-page `@action() @serverAction()` methods through the
/// canonical binding table and published package capability records.
///
/// # Errors
///
/// Returns stable errors for malformed source declarations or unbound/non
/// server-action capabilities. It neither analyzes package code nor creates a
/// server execution model.
pub fn build_route_server_action_plan_v1(
    components: &[ComponentNode],
    graph: &FileRouteGraphV1,
    bindings: &BindingTable,
) -> Result<RouteServerActionPlanV1, RouteServerActionPlanErrorV1> {
    let components = components
        .iter()
        .map(|component| (component.id.clone(), component))
        .collect::<BTreeMap<_, _>>();
    let route_components = graph
        .routes
        .iter()
        .map(|route| route.component.clone())
        .collect::<BTreeSet<_>>();
    if let Some(component) = components.values().find(|component| {
        !route_components.contains(&component.id) && !component.server_action_facts.is_empty()
    }) {
        return Err(RouteServerActionPlanErrorV1 {
            code: "PSROUTE1116_SERVER_ACTION_NOT_ROUTE_PAGE",
            message: format!(
                "component `{}` declares @serverAction() but is not a conventional route page",
                component.id
            ),
        });
    }

    let mut routes = Vec::new();
    for route in &graph.routes {
        let Some(component) = components.get(&route.component) else {
            return Err(RouteServerActionPlanErrorV1 {
                code: "PSROUTE1111_SERVER_ACTION_ROUTE_COMPONENT_MISSING",
                message: route.component.to_string(),
            });
        };
        let mut actions = Vec::new();
        for fact in &component.server_action_facts {
            if !fact.invoked
                || fact.argument_count != 1
                || fact
                    .endpoint_designator
                    .as_ref()
                    .is_none_or(String::is_empty)
                || !fact.is_action
                || !fact.action_invoked
                || fact.is_async
                || fact.parameter_count != 0
                || fact.has_body_effects
            {
                return Err(RouteServerActionPlanErrorV1 {
                    code: "PSROUTE1112_SERVER_ACTION_DECLARATION_INVALID",
                    message: format!(
                        "server action `{}` must be an empty synchronous @action() @serverAction(\"importedEndpoint\") method with no parameters",
                        fact.method_name
                    ),
                });
            }
            let designator = fact
                .endpoint_designator
                .as_deref()
                .expect("validated above");
            let binding = bindings
                .resolve_import(&component.module_path, designator)
                .ok_or_else(|| RouteServerActionPlanErrorV1 {
                    code: "PSROUTE1113_SERVER_ACTION_ENDPOINT_UNBOUND",
                    message: format!(
                        "server action `{}` cannot resolve `{designator}`",
                        fact.method_name
                    ),
                })?;
            let ImportBindingTarget::SemanticPackage {
                package,
                version,
                integrity,
                export,
                kind: SemanticPackageKind::ServerAction,
                type_signature,
                runtime_module,
                resume_policy,
                server_action: Some(action),
                ..
            } = &binding.target
            else {
                return Err(RouteServerActionPlanErrorV1 {
                    code: "PSROUTE1114_SERVER_ACTION_CAPABILITY_INVALID",
                    message: format!(
                        "server action `{}` must select an imported server_action capability",
                        fact.method_name
                    ),
                });
            };
            actions.push(server_action_binding(
                component,
                &fact.method_name,
                package,
                version,
                integrity,
                export,
                runtime_module,
                type_signature,
                resume_policy,
                action,
            ));
        }
        routes.push(RouteServerActionRouteV1 {
            path: route.path.clone(),
            page_component_id: route.component.to_string(),
            actions,
        });
    }
    Ok(RouteServerActionPlanV1 {
        schema_version: ROUTE_SERVER_ACTION_PLAN_SCHEMA_VERSION,
        routes,
    })
}

#[allow(clippy::too_many_arguments)]
fn server_action_binding(
    component: &ComponentNode,
    method: &str,
    package: &str,
    version: &str,
    integrity: &str,
    export: &str,
    runtime_module: &str,
    type_signature: &str,
    resume_policy: &str,
    action: &SemanticPackageServerAction,
) -> RouteServerActionBindingV1 {
    RouteServerActionBindingV1 {
        id: component.id.server_action(method).to_string(),
        component_id: component.id.to_string(),
        method: method.into(),
        package: package.into(),
        version: version.into(),
        integrity: integrity.into(),
        export: export.into(),
        runtime_module: runtime_module.into(),
        type_signature: type_signature.into(),
        resume_policy: resume_policy.into(),
        input: match action.input {
            crate::SemanticPackageServerActionInput::FormData => "form_data".into(),
        },
        response: match action.response {
            crate::SemanticPackageServerActionResponse::Json => "json".into(),
            crate::SemanticPackageServerActionResponse::Redirect => "redirect".into(),
        },
        failure: match action.failure {
            crate::SemanticPackageRouteLoaderFailure::Typed => "typed".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model_for_unit_with_packages, build_binding_table_with_packages,
        build_module_graph, build_route_server_action_plan_v1, build_symbol_table,
        build_validated_file_route_graph_v1, parse_semantic_package_contract, CompilationUnit,
        SemanticPackageResolutionTable,
    };

    #[test]
    fn resolves_a_route_server_action_only_from_an_integrity_bound_package_capability() {
        let unit = CompilationUnit::parse_sources([(
            "app/routes/posts/[slug].tsx",
            r#"
import { savePost } from "post-service";
@component() class Post {
  @action() @serverAction("savePost") save(): void {}
  render() { return <form />; }
}
"#,
        )]);
        let mut packages = SemanticPackageResolutionTable::default();
        packages
            .insert(
                "post-service".into(),
                parse_semantic_package_contract(
                    r#"{"schema_version":1,"package":"post-service","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"savePost":{"kind":"server_action","type_signature":"FormData -> ServerActionResult","runtime_module":"dist/save-post.js","resume_policy":"cold_fallback","server_action":{"input":"form_data","response":"json","failure":"typed"}}}}"#,
                )
                .unwrap(),
            )
            .unwrap();
        let model = build_application_semantic_model_for_unit_with_packages(&unit, &packages);
        let graph = build_validated_file_route_graph_v1(&model).unwrap();
        let symbols = build_symbol_table(&unit);
        let modules = build_module_graph(&unit);
        let bindings = build_binding_table_with_packages(&unit, &symbols, &modules, &packages);

        let plan = build_route_server_action_plan_v1(&model.components, &graph, &bindings).unwrap();
        assert_eq!(plan.routes.len(), 1);
        assert_eq!(plan.routes[0].actions.len(), 1);
        let action = &plan.routes[0].actions[0];
        assert_eq!(action.method, "save");
        assert_eq!(action.package, "post-service");
        assert_eq!(action.input, "form_data");
        assert_eq!(action.response, "json");
        assert_eq!(action.resume_policy, "cold_fallback");
    }
}
