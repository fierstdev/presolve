//! Compiler-owned route-loader planning over closed semantic-package facts.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    resume_value_codec, semantic_type_text, ApplicationSemanticModel, BindingTable, ComponentNode,
    FileRouteGraphV1, ImportBindingTarget, ResourceActivationId, ResourceId, ResumeValueCodec,
    SemanticPackageKind, SemanticPackageRouteLoader, SemanticType,
};

pub const ROUTE_LOADER_PLAN_SCHEMA_VERSION: u32 = 2;

#[must_use]
pub fn route_loader_plan_json_v2(plan: &RouteLoaderPlanV2) -> String {
    serde_json::to_string_pretty(plan).expect("route loader plan serializes") + "\n"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteLoaderPlanErrorV2 {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteLoaderPlanV2 {
    pub schema_version: u32,
    pub routes: Vec<RouteLoaderRouteV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteLoaderRouteV2 {
    pub path: String,
    pub page_component_id: String,
    pub loaders: Vec<RouteLoaderBindingV2>,
}

/// One fully resolved server loader. All values are exact package-contract
/// facts; this product contains no callback or executable package source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteLoaderBindingV2 {
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
    pub data_type: String,
    pub error_type: String,
    pub data_codec: ResumeValueCodec,
    pub error_codec: ResumeValueCodec,
    pub resource_declaration_id: String,
    pub resource_activation_id: String,
    pub component_instance_id: String,
    pub state_slot_id: String,
    pub data_slot_id: String,
    pub error_slot_id: String,
    pub parameters: Vec<RouteLoaderParameterV2>,
    pub normalization: RouteLoaderNormalizationV2,
    pub cache_key: RouteLoaderCacheKeyV2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteLoaderParameterV2 {
    pub name: String,
    pub segment_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteLoaderNormalizationV2 {
    pub percent_decoding: String,
    pub invalid_utf8: String,
    pub duplicate_parameters: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteLoaderCacheKeyV2 {
    pub digest: String,
    pub ingredients: Vec<String>,
    pub private_partition: bool,
}

/// Resolves route page loader fields through the existing compiler
/// binding table and published route-loader capability records.
///
/// # Errors
///
/// Returns stable errors for malformed source facts or unbound/non-loader
/// package selections. No package implementation is inspected.
pub fn build_route_loader_plan_v2(
    model: &ApplicationSemanticModel,
    graph: &FileRouteGraphV1,
    bindings: &BindingTable,
) -> Result<RouteLoaderPlanV2, RouteLoaderPlanErrorV2> {
    let components = model
        .components
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
        return Err(RouteLoaderPlanErrorV2 {
            code: "PSROUTE1106_LOADER_NOT_ROUTE_PAGE",
            message: format!(
                "component `{}` declares a route loader but is not a conventional route page",
                component.id
            ),
        });
    }
    let mut routes = Vec::new();
    for route in &graph.routes {
        let Some(component) = components.get(&route.component) else {
            return Err(RouteLoaderPlanErrorV2 {
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
                return Err(RouteLoaderPlanErrorV2 {
                    code: "PSROUTE1102_LOADER_DECLARATION_INVALID",
                    message: format!(
                        "route loader `{}` must use one canonical loader field or legacy @loader(\"importedEndpoint\") declaration",
                        candidate.field
                    ),
                });
            }
            if !candidate
                .declared_type
                .as_ref()
                .is_some_and(|type_| type_.text.starts_with("Resource<"))
            {
                return Err(RouteLoaderPlanErrorV2 {
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
                .ok_or_else(|| RouteLoaderPlanErrorV2 {
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
                return Err(RouteLoaderPlanErrorV2 {
                    code: "PSROUTE1105_LOADER_CAPABILITY_INVALID",
                    message: format!(
                        "route loader `{}` must select an imported resource route_loader capability",
                        candidate.field
                    ),
                });
            };
            loaders.push(loader_binding(
                model,
                bindings,
                route,
                component,
                candidate,
                &candidate.field,
                package,
                version,
                integrity,
                export,
                runtime_module,
                type_signature,
                loader,
            )?);
        }
        routes.push(RouteLoaderRouteV2 {
            path: route.path.clone(),
            page_component_id: route.component.to_string(),
            loaders,
        });
    }
    Ok(RouteLoaderPlanV2 {
        schema_version: ROUTE_LOADER_PLAN_SCHEMA_VERSION,
        routes,
    })
}

#[allow(clippy::too_many_arguments)]
fn loader_binding(
    model: &ApplicationSemanticModel,
    bindings: &BindingTable,
    route: &crate::FileRouteNodeV1,
    component: &ComponentNode,
    candidate: &crate::AuthoredRouteLoaderDeclarationFact,
    field: &str,
    package: &str,
    version: &str,
    integrity: &str,
    export: &str,
    runtime_module: &str,
    type_signature: &str,
    loader: &SemanticPackageRouteLoader,
) -> Result<RouteLoaderBindingV2, RouteLoaderPlanErrorV2> {
    let cache = &loader.cache;
    let declared = candidate
        .declared_type
        .as_ref()
        .ok_or_else(|| RouteLoaderPlanErrorV2 {
            code: "PSROUTE1103_LOADER_TYPE_INVALID",
            message: format!("route loader `{field}` must declare Resource<Data, Error>"),
        })?;
    let resolved = model
        .semantic_types
        .resolve_declared_type(declared, Some(bindings))
        .ok_or_else(|| RouteLoaderPlanErrorV2 {
            code: "PSROUTE1107_LOADER_CODEC_INVALID",
            message: format!("route loader `{field}` has an unresolved Resource type"),
        })?;
    let SemanticType::Resource(resource_type) = resolved.semantic_type else {
        return Err(RouteLoaderPlanErrorV2 {
            code: "PSROUTE1103_LOADER_TYPE_INVALID",
            message: format!("route loader `{field}` must declare Resource<Data, Error>"),
        });
    };
    let data_codec =
        resume_value_codec(&resource_type.data).map_err(|_| RouteLoaderPlanErrorV2 {
            code: "PSROUTE1107_LOADER_CODEC_INVALID",
            message: format!("route loader `{field}` data type has no closed runtime codec"),
        })?;
    let error_codec =
        resume_value_codec(&resource_type.error).map_err(|_| RouteLoaderPlanErrorV2 {
            code: "PSROUTE1107_LOADER_CODEC_INVALID",
            message: format!("route loader `{field}` error type has no closed runtime codec"),
        })?;
    let instances = model
        .component_instance_plan
        .instances
        .values()
        .filter(|instance| instance.component == component.id)
        .collect::<Vec<_>>();
    let [instance] = instances.as_slice() else {
        return Err(RouteLoaderPlanErrorV2 {
            code: "PSROUTE1108_LOADER_INSTANCE_MISSING",
            message: format!(
                "route loader `{field}` requires exactly one route component instance for `{}`; found {}",
                route.path,
                instances.len()
            ),
        });
    };
    let declaration_id = ResourceId::for_owner(&component.id, field);
    let activation_id = ResourceActivationId::for_component_instance(&instance.id, &declaration_id);
    let parameters = route
        .path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .enumerate()
        .filter_map(|(segment_index, segment)| {
            segment
                .strip_prefix(':')
                .map(|name| RouteLoaderParameterV2 {
                    name: name.to_owned(),
                    segment_index,
                })
        })
        .collect::<Vec<_>>();
    if parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<BTreeSet<_>>()
        .len()
        != parameters.len()
    {
        return Err(RouteLoaderPlanErrorV2 {
            code: "PSROUTE1109_LOADER_PARAMETER_DUPLICATE",
            message: format!("route `{}` repeats a loader parameter name", route.path),
        });
    }
    let binding = RouteLoaderBindingV2 {
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
        data_type: semantic_type_text(&resource_type.data),
        error_type: semantic_type_text(&resource_type.error),
        data_codec,
        error_codec,
        resource_declaration_id: declaration_id.as_str().to_owned(),
        resource_activation_id: activation_id.as_str().to_owned(),
        component_instance_id: instance.id.as_str().to_owned(),
        state_slot_id: activation_id.state_slot().as_str().to_owned(),
        data_slot_id: activation_id.data_slot().as_str().to_owned(),
        error_slot_id: activation_id.error_slot().as_str().to_owned(),
        parameters,
        normalization: RouteLoaderNormalizationV2 {
            percent_decoding: "strict_utf8".into(),
            invalid_utf8: "reject".into(),
            duplicate_parameters: "reject".into(),
        },
        cache_key: RouteLoaderCacheKeyV2 {
            digest: "sha256".into(),
            ingredients: vec![
                "loader_capability_id".into(),
                "canonical_ordered_route_parameters_json".into(),
            ],
            private_partition: matches!(
                cache.scope,
                crate::SemanticPackageServerCacheScope::Private
            ),
        },
    };
    Ok(binding)
}

#[cfg(test)]
mod tests {
    use crate::{
        build_application_semantic_model_for_unit_with_packages, build_binding_table_with_packages,
        build_module_graph, build_route_loader_plan_v2, build_symbol_table,
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
  @loader("loadPost") post!: Resource<string, string>;
  render() { return <article />; }
}
"#,
        )]);
        let mut packages = SemanticPackageResolutionTable::default();
        packages
            .insert(
                "post-service".into(),
                parse_semantic_package_contract(
                    r#"{"schema_version":1,"package":"post-service","version":"1.2.3","integrity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","exports":{"loadPost":{"kind":"resource","type_signature":"(RouteParameters, AbortSignal) -> Promise<RouteLoaderResult>","runtime_module":"dist/load-post.js","resume_policy":"reload","resource_endpoint":{"execution_boundary":"server","cancellation":"abort","resume":"reload"},"route_loader":{"input":"route_parameters","cache":{"scope":"public","max_age_seconds":60},"failure":"typed"}}}}"#,
                )
                .unwrap(),
            )
            .unwrap();
        let model = build_application_semantic_model_for_unit_with_packages(&unit, &packages);
        let graph = build_validated_file_route_graph_v1(&model).unwrap();
        let symbols = build_symbol_table(&unit);
        let modules = build_module_graph(&unit);
        let bindings = build_binding_table_with_packages(&unit, &symbols, &modules, &packages);

        let plan = build_route_loader_plan_v2(&model, &graph, &bindings).unwrap();
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
