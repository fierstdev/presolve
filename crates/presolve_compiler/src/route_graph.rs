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

/// Compiler-derived route topology for the ergonomic `app/routes` convention.
///
/// This is intentionally distinct from the frozen explicit static-route
/// product. It records layout ownership before a later publication product
/// decides how a route page is emitted; it does not introduce a router runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRouteGraphV1 {
    pub routes: Vec<FileRouteNodeV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRouteNodeV1 {
    pub path: String,
    pub component: SemanticId,
    /// Ordered from the application shell to the nearest route layout.
    pub layouts: Vec<SemanticId>,
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
    pub parent_path: Option<String>,
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

/// Builds the ergonomic route graph: explicit `@route()` wins, otherwise an
/// `app/routes/**/*.tsx` component derives its path from its module path.
#[must_use]
pub fn build_file_route_graph_v1(model: &ApplicationSemanticModel) -> RouteGraph {
    RouteGraph {
        routes: model
            .components
            .iter()
            .filter_map(|component| {
                component
                    .route_path
                    .clone()
                    .or_else(|| file_route_path(&component.module_path))
                    .map(|path| RouteNode {
                        path,
                        component: component.id.clone(),
                    })
            })
            .collect(),
    }
}

/// Builds ergonomic file-route topology directly from compiler component facts.
/// This is used during file-route application-model assembly before the full
/// application model has derived instance/runtime products.
#[must_use]
pub fn build_file_route_graph_from_components_v1(
    components: &[crate::ComponentNode],
) -> RouteGraph {
    RouteGraph {
        routes: components
            .iter()
            .filter_map(|component| {
                component
                    .route_path
                    .clone()
                    .or_else(|| file_route_path(&component.module_path))
                    .map(|path| RouteNode {
                        path,
                        component: component.id.clone(),
                    })
            })
            .collect(),
    }
}

/// Builds and validates the complete `app/routes` topology, including
/// conventional `layout.tsx` files. A layout file must declare exactly one
/// component and cannot also claim a route through `@route()`.
///
/// # Errors
///
/// Returns stable diagnostics for ambiguous layouts and route patterns. Dynamic
/// parameter names are intentionally not identity: `/posts/:id` and
/// `/posts/:slug` conflict because they match the same requests.
pub fn build_validated_file_route_graph_v1(
    model: &ApplicationSemanticModel,
) -> Result<FileRouteGraphV1, RouteGraphError> {
    let mut layouts = BTreeMap::<Vec<String>, SemanticId>::new();
    for component in &model.components {
        let Some(scope) = file_layout_scope(&component.module_path) else {
            continue;
        };
        let scope = normalize_route_scope(scope);
        if component.route_path.is_some() {
            return Err(RouteGraphError {
                code: "PSROUTE1010_LAYOUT_CANNOT_DECLARE_ROUTE",
                message: format!(
                    "layout component `{}` must not declare @route()",
                    component.id
                ),
            });
        }
        if layouts
            .insert(scope.clone(), component.id.clone())
            .is_some()
        {
            return Err(RouteGraphError {
                code: "PSROUTE1011_LAYOUT_COMPONENT_AMBIGUOUS",
                message: format!(
                    "layout scope `{}` must declare exactly one component",
                    route_scope_display(&scope)
                ),
            });
        }
    }

    let mut routes = Vec::new();
    for component in &model.components {
        if file_layout_scope(&component.module_path).is_some() {
            continue;
        }
        let Some(file_path) = file_route_path(&component.module_path) else {
            continue;
        };
        let path = component.route_path.clone().unwrap_or(file_path);
        if !is_file_route_path(&path) {
            return Err(RouteGraphError {
                code: "PSROUTE1012_INVALID_FILE_ROUTE_PATH",
                message: format!("route path `{path}` is not a supported file-route path"),
            });
        }
        let layouts = layout_chain_for_route(&path, &layouts);
        routes.push(FileRouteNodeV1 {
            path,
            component: component.id.clone(),
            layouts,
        });
    }
    routes.sort_by(|left, right| {
        route_match_shape(&left.path)
            .cmp(&route_match_shape(&right.path))
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.component.cmp(&right.component))
    });
    for pair in routes.windows(2) {
        if route_match_shape(&pair[0].path) == route_match_shape(&pair[1].path) {
            return Err(RouteGraphError {
                code: "PSROUTE1013_FILE_ROUTE_CONFLICT",
                message: format!(
                    "routes `{}` and `{}` match the same request path",
                    pair[0].path, pair[1].path
                ),
            });
        }
    }
    Ok(FileRouteGraphV1 { routes })
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
                parent_path: route_parent_path(&route.path),
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
            parent_path: route_parent_path(&route.path),
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
fn route_parent_path(path: &str) -> Option<String> {
    if path == "/" {
        return None;
    }
    let trimmed = path.trim_matches('/');
    let parent = trimmed.rsplit_once('/').map_or("", |(parent, _)| parent);
    Some(if parent.is_empty() {
        "/".into()
    } else {
        format!("/{parent}")
    })
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

fn is_file_route_path(path: &str) -> bool {
    path.starts_with('/')
        && !path.contains("//")
        && path.split('/').all(|segment| {
            segment.is_empty()
                || (segment.starts_with(':')
                    && segment.len() > 1
                    && segment[1..]
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '_'))
                || (segment != "."
                    && segment != ".."
                    && !segment.contains('*')
                    && !segment.contains('{')
                    && !segment.contains('}'))
        })
}

fn file_route_path(path: &std::path::Path) -> Option<String> {
    let values = path
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()?;
    let index = values.iter().position(|value| *value == "routes")?;
    if values.get(index.checked_sub(1)?)? != &"app" {
        return None;
    }
    let mut segments = values[index + 1..]
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    let filename = segments.pop()?;
    let stem = filename
        .strip_suffix(".tsx")
        .or_else(|| filename.strip_suffix(".ts"))?;
    if stem != "index" {
        segments.push(route_segment(stem)?);
    }
    for segment in &mut segments {
        *segment = route_segment(segment)?;
    }
    Some(if segments.is_empty() {
        "/".into()
    } else {
        format!("/{}", segments.join("/"))
    })
}

fn file_layout_scope(path: &std::path::Path) -> Option<Vec<String>> {
    let values = path
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()?;
    let filename = values.last()?;
    if !matches!(*filename, "layout.ts" | "layout.tsx") {
        return None;
    }
    match values.as_slice() {
        ["app", _] => Some(Vec::new()),
        ["app", "routes", rest @ .., _] => {
            rest.iter().map(|segment| route_segment(segment)).collect()
        }
        _ => None,
    }
}

fn layout_chain_for_route(
    path: &str,
    layouts: &BTreeMap<Vec<String>, SemanticId>,
) -> Vec<SemanticId> {
    let segments = path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(route_match_segment)
        .collect::<Vec<_>>();
    (0..=segments.len())
        .filter_map(|length| layouts.get(&segments[..length]).cloned())
        .collect()
}

fn route_match_shape(path: &str) -> String {
    path.split('/')
        .map(route_match_segment)
        .collect::<Vec<_>>()
        .join("/")
}

fn route_match_segment(segment: &str) -> String {
    if segment.starts_with(':') {
        ":".into()
    } else {
        segment.into()
    }
}

fn normalize_route_scope(scope: Vec<String>) -> Vec<String> {
    scope
        .into_iter()
        .map(|segment| route_match_segment(&segment))
        .collect()
}

fn route_scope_display(scope: &[String]) -> String {
    if scope.is_empty() {
        "/".into()
    } else {
        format!("/{}", scope.join("/"))
    }
}

fn route_segment(segment: &str) -> Option<String> {
    if let Some(parameter) = segment
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    {
        if parameter.is_empty()
            || !parameter
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            return None;
        }
        return Some(format!(":{parameter}"));
    }
    (!segment.is_empty()).then(|| segment.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_application_semantic_model, build_application_semantic_model_for_unit,
        CompilationUnit,
    };
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

    #[test]
    fn derives_file_routes_and_keeps_explicit_route_override() {
        let source =
            r#"@component() class Home extends Component { render() { return <main />; } }"#;
        let model = build_application_semantic_model(&parse_file("app/routes/index.tsx", source));
        assert_eq!(build_file_route_graph_v1(&model).routes[0].path, "/");
        let source = r#"@route("/welcome") @component() class Home extends Component { render() { return <main />; } }"#;
        let model = build_application_semantic_model(&parse_file("app/routes/index.tsx", source));
        assert_eq!(build_file_route_graph_v1(&model).routes[0].path, "/welcome");
    }

    #[test]
    fn derives_typed_parameter_segment_from_file_name() {
        let source =
            r#"@component() class Post extends Component { render() { return <main />; } }"#;
        let model =
            build_application_semantic_model(&parse_file("app/routes/blog/[slug].tsx", source));
        assert_eq!(
            build_file_route_graph_v1(&model).routes[0].path,
            "/blog/:slug"
        );
    }

    #[test]
    fn derives_nested_layout_chains_for_file_routes() {
        let model = build_application_semantic_model_for_unit(&CompilationUnit::parse_sources([
            (
                "app/layout.tsx",
                r#"@component() class AppLayout extends Component { render() { return <main />; } }"#,
            ),
            (
                "app/routes/blog/layout.tsx",
                r#"@component() class BlogLayout extends Component { render() { return <section />; } }"#,
            ),
            (
                "app/routes/blog/[slug].tsx",
                r#"@component() class Post extends Component { render() { return <article />; } }"#,
            ),
        ]));

        let graph = build_validated_file_route_graph_v1(&model).unwrap();

        assert_eq!(graph.routes.len(), 1);
        assert_eq!(graph.routes[0].path, "/blog/:slug");
        assert_eq!(
            graph.routes[0]
                .layouts
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec![
                "module:app/layout.tsx/component:presolve-app-layout",
                "module:app/routes/blog/layout.tsx/component:presolve-blog-layout"
            ]
        );
    }

    #[test]
    fn rejects_file_routes_that_differ_only_by_parameter_name() {
        let model = build_application_semantic_model_for_unit(&CompilationUnit::parse_sources([
            (
                "app/routes/blog/[id].tsx",
                r#"@component() class ById extends Component { render() { return <article />; } }"#,
            ),
            (
                "app/routes/blog/[slug].tsx",
                r#"@component() class BySlug extends Component { render() { return <article />; } }"#,
            ),
        ]));

        assert_eq!(
            build_validated_file_route_graph_v1(&model)
                .unwrap_err()
                .code,
            "PSROUTE1013_FILE_ROUTE_CONFLICT"
        );
    }
}
