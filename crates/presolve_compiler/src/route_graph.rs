use crate::application_semantic_model::ApplicationSemanticModel;
use crate::semantic_id::SemanticId;
use serde::Serialize;

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
            })
            .collect(),
    }
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
