use crate::application_semantic_model::ApplicationSemanticModel;
use crate::semantic_id::SemanticId;
use crate::{
    build_application_publication_product_v1, validate_application_publication_request_v1,
    ApplicationPublicationErrorV1, ApplicationPublicationProductV1,
    ApplicationPublicationRequestV1,
};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteGraph {
    pub routes: Vec<RouteNode>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteNode {
    pub path: String,
    pub component: SemanticId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteGraphError {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteManifestV1 {
    pub schema_version: u32,
    pub routes: Vec<RouteManifestEntryV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteManifestEntryV1 {
    pub path: String,
    pub component_id: String,
    pub artifact_root: String,
}

#[must_use]
pub fn build_route_graph(model: &ApplicationSemanticModel) -> RouteGraph {
    RouteGraph {
        routes: model
            .components
            .iter()
            .filter_map(|component| {
                component.route_path.as_ref().map(|path| RouteNode {
                    path: path.clone(),
                    component: component.id.clone(),
                })
            })
            .collect(),
    }
}

/// Produces the deterministic static route product accepted by Phase Q Q1.
///
/// # Errors
///
/// Returns a stable error for malformed or duplicate route identities.
pub fn build_validated_route_graph_v1(
    model: &ApplicationSemanticModel,
) -> Result<RouteGraph, RouteGraphError> {
    let mut graph = build_route_graph(model);
    graph
        .routes
        .sort_by(|left, right| left.path.cmp(&right.path));
    for route in &graph.routes {
        if !is_static_route_path(&route.path) {
            return Err(RouteGraphError {
                code: "PSROUTE1001_INVALID_STATIC_ROUTE_PATH",
                message: format!(
                    "route path `{}` must begin with / and contain no dynamic segments",
                    route.path
                ),
            });
        }
    }
    if graph
        .routes
        .windows(2)
        .any(|pair| pair[0].path == pair[1].path)
    {
        return Err(RouteGraphError {
            code: "PSROUTE1002_DUPLICATE_ROUTE_PATH",
            message: "each static route path must identify exactly one component".into(),
        });
    }
    Ok(graph)
}

#[must_use]
pub fn route_manifest_v1(graph: &RouteGraph) -> RouteManifestV1 {
    RouteManifestV1 {
        schema_version: 1,
        routes: graph
            .routes
            .iter()
            .map(|route| RouteManifestEntryV1 {
                path: route.path.clone(),
                component_id: route.component.to_string(),
                artifact_root: route_artifact_root(&route.path),
            })
            .collect(),
    }
}

/// Builds the complete static multi-route publication inventory solely from
/// compiler products. Each route receives an immutable namespaced application
/// artifact family; no host or JavaScript layer combines page artifacts.
///
/// # Errors
///
/// Returns route-identity or canonical application-publication errors.
pub fn build_static_route_publication_v1(
    requests: Vec<ApplicationPublicationRequestV1>,
) -> Result<(RouteManifestV1, BTreeMap<std::path::PathBuf, Vec<u8>>), RouteGraphError> {
    if requests.is_empty() {
        return Err(RouteGraphError {
            code: "PSROUTE1003_EMPTY_ROUTE_SET",
            message: "static route publication requires explicit route entries".into(),
        });
    }
    let mut artifacts = BTreeMap::new();
    let mut routes = Vec::new();
    for request in requests {
        let validated =
            validate_application_publication_request_v1(request).map_err(route_request_error)?;
        let model = crate::build_application_semantic_model_for_unit_with_packages(
            &validated.unit,
            &validated.request.package_contracts,
        );
        let graph = build_validated_route_graph_v1(&model)?;
        let route = graph
            .routes
            .iter()
            .find(|route| route.component == validated.entry_component)
            .ok_or_else(|| RouteGraphError {
                code: "PSROUTE1004_ENTRY_ROUTE_MISSING",
                message: "each route publication entry must declare one matching @route path"
                    .into(),
            })?;
        let root = route_artifact_root(&route.path);
        if routes
            .iter()
            .any(|entry: &RouteManifestEntryV1| entry.path == route.path)
        {
            return Err(RouteGraphError {
                code: "PSROUTE1002_DUPLICATE_ROUTE_PATH",
                message: "each static route path must identify exactly one component".into(),
            });
        }
        let product =
            build_application_publication_product_v1(validated).map_err(route_product_error)?;
        insert_route_product(&mut artifacts, &root, product)?;
        routes.push(RouteManifestEntryV1 {
            path: route.path.clone(),
            component_id: route.component.to_string(),
            artifact_root: root,
        });
    }
    routes.sort_by(|left, right| left.path.cmp(&right.path));
    let manifest = RouteManifestV1 {
        schema_version: 1,
        routes,
    };
    artifacts.insert(
        std::path::PathBuf::from("routes.manifest.json"),
        route_manifest_json_v1(&manifest).into_bytes(),
    );
    Ok((manifest, artifacts))
}

fn insert_route_product(
    artifacts: &mut BTreeMap<std::path::PathBuf, Vec<u8>>,
    root: &str,
    product: ApplicationPublicationProductV1,
) -> Result<(), RouteGraphError> {
    for (path, bytes) in product.artifacts {
        let path = std::path::PathBuf::from(root).join(path);
        if artifacts.insert(path.clone(), bytes).is_some() {
            return Err(RouteGraphError {
                code: "PSROUTE1005_ARTIFACT_PATH_COLLISION",
                message: format!(
                    "route publication generated colliding artifact {}",
                    path.display()
                ),
            });
        }
    }
    Ok(())
}

fn route_request_error(error: crate::ApplicationPublicationRequestErrorV1) -> RouteGraphError {
    RouteGraphError {
        code: error.code,
        message: error.message,
    }
}
fn route_product_error(error: ApplicationPublicationErrorV1) -> RouteGraphError {
    RouteGraphError {
        code: error.code,
        message: error.message,
    }
}
fn route_artifact_root(path: &str) -> String {
    if path == "/" {
        return "routes/root".into();
    }
    format!("routes/{}", path.trim_matches('/').replace('/', "__"))
}

#[must_use]
pub fn route_manifest_json_v1(manifest: &RouteManifestV1) -> String {
    serde_json::to_string_pretty(manifest).expect("route manifest is serializable") + "\n"
}

fn is_static_route_path(path: &str) -> bool {
    path.starts_with('/')
        && !path.contains("//")
        && !path.contains('*')
        && !path.contains(':')
        && !path.contains('{')
        && !path.contains('}')
        && path.split('/').all(|segment| !segment.contains(".."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_application_semantic_model;
    use presolve_parser::parse_file;

    #[test]
    fn validates_and_sorts_static_route_identity() {
        let source = r#"@route("/about") @component("x-about") class About extends Component { render() { return <main />; } } @route("/") @component("x-home") class Home extends Component { render() { return <main />; } }"#;
        let model = build_application_semantic_model(&parse_file("src/routes.tsx", source));
        let graph = build_validated_route_graph_v1(&model).unwrap();
        assert_eq!(
            graph
                .routes
                .iter()
                .map(|route| route.path.as_str())
                .collect::<Vec<_>>(),
            vec!["/", "/about"]
        );
        assert!(route_manifest_json_v1(&route_manifest_v1(&graph)).contains("schema_version"));
    }
}
