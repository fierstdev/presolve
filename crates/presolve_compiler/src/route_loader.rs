//! Compiler-owned route-loader planning over closed semantic-package facts.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    BindingTable, ComponentNode, FileRouteGraphV1, ImportBindingTarget, SemanticPackageKind,
    SemanticPackageRouteLoader,
};

pub const ROUTE_LOADER_PLAN_SCHEMA_VERSION: u32 = 1;

#[must_use]
pub fn route_loader_plan_json_v1(plan: &RouteLoaderPlanV1) -> String {
    serde_json::to_string_pretty(plan).expect("route loader plan serializes") + "\n"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteLoaderPlanErrorV1 {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteLoaderPlanV1 {
    pub schema_version: u32,
    pub routes: Vec<RouteLoaderRouteV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteLoaderRouteV1 {
    pub path: String,
    pub page_component_id: String,
    pub loaders: Vec<RouteLoaderBindingV1>,
}

/// One fully resolved server loader. All values are exact package-contract
/// facts; this product contains no callback or executable package source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteLoaderBindingV1 {
    pub id: String,
    pub component_id: String,
    pub field: String,
    pub package: String,
    pub version: String,
    pub integrity: String,
    pub export: String,
    pub runtime_module: String,
    pub type_signature: String,
    pub input: String,
    pub cache_scope: String,
    pub max_age_seconds: Option<u64>,
    pub failure: String,
}

/// Resolves route page `@loader()` fields through the existing compiler
/// binding table and published route-loader capability records.
///
/// # Errors
///
/// Returns stable errors for malformed source facts or unbound/non-loader
/// package selections. No package implementation is inspected.
pub fn build_route_loader_plan_v1(
    components: &[ComponentNode],
    graph: &FileRouteGraphV1,
    bindings: &BindingTable,
) -> Result<RouteLoaderPlanV1, RouteLoaderPlanErrorV1> {
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
        !route_components.contains(&component.id)
            && !component.route_loader_declaration_candidates.is_empty()
    }) {
        return Err(RouteLoaderPlanErrorV1 {
            code: "PSROUTE1106_LOADER_NOT_ROUTE_PAGE",
            message: format!(
                "component `{}` declares @loader() but is not a conventional route page",
                component.id
            ),
        });
    }
    let mut routes = Vec::new();
    for route in &graph.routes {
        let Some(component) = components.get(&route.component) else {
            return Err(RouteLoaderPlanErrorV1 {
                code: "PSROUTE1101_LOADER_ROUTE_COMPONENT_MISSING",
                message: route.component.to_string(),
            });
        };
        let mut loaders = Vec::new();
        for candidate in &component.route_loader_declaration_candidates {
            if !candidate.decorator_invoked
                || candidate.decorator_argument_count != 1
                || candidate.endpoint_designator.is_none()
            {
                return Err(RouteLoaderPlanErrorV1 {
                    code: "PSROUTE1102_LOADER_DECLARATION_INVALID",
                    message: format!(
                        "route loader `{}` must use @loader(\"importedEndpoint\")",
                        candidate.field
                    ),
                });
            }
            if !candidate
                .declared_type
                .as_ref()
                .is_some_and(|type_| type_.text.starts_with("Resource<"))
            {
                return Err(RouteLoaderPlanErrorV1 {
                    code: "PSROUTE1103_LOADER_TYPE_INVALID",
                    message: format!(
                        "route loader `{}` must declare Resource<Data, Error>",
                        candidate.field
                    ),
                });
            }
            let designator = candidate
                .endpoint_designator
                .as_deref()
                .expect("validated above");
            let binding = bindings
                .resolve_import(&component.module_path, designator)
                .ok_or_else(|| RouteLoaderPlanErrorV1 {
                    code: "PSROUTE1104_LOADER_ENDPOINT_UNBOUND",
                    message: format!(
                        "route loader `{}` cannot resolve `{designator}`",
                        candidate.field
                    ),
                })?;
            let ImportBindingTarget::SemanticPackage {
                package,
                version,
                integrity,
                export,
                kind: SemanticPackageKind::Resource,
                type_signature,
                runtime_module,
                route_loader: Some(loader),
                ..
            } = &binding.target
            else {
                return Err(RouteLoaderPlanErrorV1 {
                    code: "PSROUTE1105_LOADER_CAPABILITY_INVALID",
                    message: format!(
                        "route loader `{}` must select an imported resource route_loader capability",
                        candidate.field
                    ),
                });
            };
            loaders.push(loader_binding(
                component,
                &candidate.field,
                package,
                version,
                integrity,
                export,
                runtime_module,
                type_signature,
                loader,
            ));
        }
        routes.push(RouteLoaderRouteV1 {
            path: route.path.clone(),
            page_component_id: route.component.to_string(),
            loaders,
        });
    }
    Ok(RouteLoaderPlanV1 {
        schema_version: ROUTE_LOADER_PLAN_SCHEMA_VERSION,
        routes,
    })
}

#[allow(clippy::too_many_arguments)]
fn loader_binding(
    component: &ComponentNode,
    field: &str,
    package: &str,
    version: &str,
    integrity: &str,
    export: &str,
    runtime_module: &str,
    type_signature: &str,
    loader: &SemanticPackageRouteLoader,
) -> RouteLoaderBindingV1 {
    let cache = &loader.cache;
    RouteLoaderBindingV1 {
        id: component.id.route_loader(field).to_string(),
        component_id: component.id.to_string(),
        field: field.into(),
        package: package.into(),
        version: version.into(),
        integrity: integrity.into(),
        export: export.into(),
        runtime_module: runtime_module.into(),
        type_signature: type_signature.into(),
        input: match loader.input {
            crate::SemanticPackageRouteLoaderInput::RouteParameters => "route_parameters".into(),
        },
        cache_scope: match cache.scope {
            crate::SemanticPackageServerCacheScope::NoStore => "no_store".into(),
            crate::SemanticPackageServerCacheScope::Private => "private".into(),
            crate::SemanticPackageServerCacheScope::Public => "public".into(),
        },
        max_age_seconds: cache.max_age_seconds,
        failure: match loader.failure {
            crate::SemanticPackageRouteLoaderFailure::Typed => "typed".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model_for_unit_with_packages, build_binding_table_with_packages,
        build_module_graph, build_route_loader_plan_v1, build_symbol_table,
        build_validated_file_route_graph_v1, parse_semantic_package_contract, CompilationUnit,
        SemanticPackageResolutionTable,
    };

    #[test]
    fn resolves_a_route_loader_only_from_an_integrity_bound_package_capability() {
        let unit = CompilationUnit::parse_sources([(
            "app/routes/posts/[slug].tsx",
            r#"
import { loadPost } from "post-service";
@component() class Post {
  @loader("loadPost") post!: Resource<Post, NotFound>;
  render() { return <article />; }
}
"#,
        )]);
        let mut packages = SemanticPackageResolutionTable::default();
        packages
            .insert(
                "post-service".into(),
                parse_semantic_package_contract(
                    r#"{"schema_version":1,"package":"post-service","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"loadPost":{"kind":"resource","type_signature":"RouteParameters -> Resource<Post, NotFound>","runtime_module":"dist/load-post.js","resume_policy":"reload","resource_endpoint":{"execution_boundary":"server","cancellation":"abort","resume":"reload"},"route_loader":{"input":"route_parameters","cache":{"scope":"public","max_age_seconds":60},"failure":"typed"}}}}"#,
                )
                .unwrap(),
            )
            .unwrap();
        let model = build_application_semantic_model_for_unit_with_packages(&unit, &packages);
        let graph = build_validated_file_route_graph_v1(&model).unwrap();
        let symbols = build_symbol_table(&unit);
        let modules = build_module_graph(&unit);
        let bindings = build_binding_table_with_packages(&unit, &symbols, &modules, &packages);

        let plan = build_route_loader_plan_v1(&model.components, &graph, &bindings).unwrap();
        assert_eq!(plan.routes.len(), 1);
        assert_eq!(plan.routes[0].loaders.len(), 1);
        let loader = &plan.routes[0].loaders[0];
        assert_eq!(
            loader.id,
            "module:app/routes/posts/[slug].tsx/component:presolve-post/route-loader:post"
        );
        assert_eq!(loader.package, "post-service");
        assert_eq!(loader.input, "route_parameters");
        assert_eq!(loader.cache_scope, "public");
        assert_eq!(loader.max_age_seconds, Some(60));
    }
}
