//! Node release inventory projected from compiler-published file routes.
//!
//! This adapter consumes only the immutable route manifest and the compiler's
//! loader/server-action handoffs. It can host compiler-proven static routes,
//! while routes with a server handoff remain explicitly Node-required until a
//! capability-specific executor is published.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use presolve_compiler::{ApplicationPublicationArtifactV1, FileRoutePublicationManifestV1};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

pub const NODE_DEPLOYMENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeDeploymentOptionsV1 {
    pub application_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeDeploymentErrorV1 {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeDeploymentPlanV1 {
    pub schema_version: u32,
    pub provider: String,
    pub application_name: String,
    pub release_id: String,
    pub routes: Vec<NodeDeploymentRouteV1>,
    pub artifacts: Vec<NodeDeploymentArtifactV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeDeploymentRouteV1 {
    pub path: String,
    pub artifact_root: String,
    /// `static` is compiler-proven exportable. `node` requires a future
    /// capability-specific loader or server-action executor.
    pub execution: String,
    pub loader_count: usize,
    pub server_action_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeDeploymentArtifactV1 {
    pub path: String,
    pub digest: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoaderPlanV1 {
    #[serde(alias = "schema_version")]
    schema_version: u32,
    routes: Vec<LoaderPlanRouteV1>,
}

#[derive(Debug, Deserialize)]
struct LoaderPlanRouteV1 {
    path: String,
    loaders: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerActionPlanV1 {
    #[serde(alias = "schema_version")]
    schema_version: u32,
    routes: Vec<ServerActionPlanRouteV1>,
}

#[derive(Debug, Deserialize)]
struct ServerActionPlanRouteV1 {
    path: String,
    actions: Vec<serde_json::Value>,
}

/// Produces a Node inventory without interpreting application source.
///
/// A route is statically exportable only when both compiler-issued handoff
/// plans contain no work for that exact route. The adapter rejects missing,
/// duplicate, or foreign handoff routes instead of guessing their meaning.
pub fn build_node_deployment_plan_v1(
    manifest: &FileRoutePublicationManifestV1,
    loader_plan_source: &str,
    server_action_plan_source: &str,
    options: &NodeDeploymentOptionsV1,
) -> Result<NodeDeploymentPlanV1, NodeDeploymentErrorV1> {
    if !is_application_name(&options.application_name) {
        return Err(NodeDeploymentErrorV1 {
            code: "PSNODE1001_APPLICATION_NAME_INVALID",
            message: options.application_name.clone(),
        });
    }
    if manifest.schema_version != 1
        || manifest.compiler_contract != "presolve-file-route-publication:1"
    {
        return Err(NodeDeploymentErrorV1 {
            code: "PSNODE1002_ROUTE_MANIFEST_UNSUPPORTED",
            message: format!(
                "schema {} contract {}",
                manifest.schema_version, manifest.compiler_contract
            ),
        });
    }
    let loaders: LoaderPlanV1 =
        serde_json::from_str(loader_plan_source).map_err(|error| NodeDeploymentErrorV1 {
            code: "PSNODE1003_LOADER_HANDOFF_INVALID",
            message: error.to_string(),
        })?;
    let actions: ServerActionPlanV1 =
        serde_json::from_str(server_action_plan_source).map_err(|error| NodeDeploymentErrorV1 {
            code: "PSNODE1004_SERVER_ACTION_HANDOFF_INVALID",
            message: error.to_string(),
        })?;
    if loaders.schema_version != 1 || actions.schema_version != 1 {
        return Err(NodeDeploymentErrorV1 {
            code: "PSNODE1005_HANDOFF_SCHEMA_UNSUPPORTED",
            message: format!(
                "loader {} server-action {}",
                loaders.schema_version, actions.schema_version
            ),
        });
    }
    let route_paths = manifest
        .routes
        .iter()
        .map(|route| route.path.clone())
        .collect::<BTreeSet<_>>();
    if route_paths.len() != manifest.routes.len()
        || route_paths.is_empty()
        || manifest.routes.iter().any(|route| {
            !is_route_path(&route.path)
                || !is_artifact_path(&route.artifact_root)
                || !route.artifact_root.starts_with("routes/")
        })
    {
        return Err(NodeDeploymentErrorV1 {
            code: "PSNODE1006_ROUTE_MANIFEST_INVALID",
            message: "route manifest must contain one or more unique routes".into(),
        });
    }
    let loader_counts = handoff_counts(
        loaders
            .routes
            .into_iter()
            .map(|route| (route.path, route.loaders.len())),
        &route_paths,
        "loader",
    )?;
    let action_counts = handoff_counts(
        actions
            .routes
            .into_iter()
            .map(|route| (route.path, route.actions.len())),
        &route_paths,
        "server-action",
    )?;
    let mut routes = manifest
        .routes
        .iter()
        .map(|route| {
            let loader_count = loader_counts[&route.path];
            let server_action_count = action_counts[&route.path];
            NodeDeploymentRouteV1 {
                path: route.path.clone(),
                artifact_root: route.artifact_root.clone(),
                execution: if loader_count == 0 && server_action_count == 0 {
                    "static".into()
                } else {
                    "node".into()
                },
                loader_count,
                server_action_count,
            }
        })
        .collect::<Vec<_>>();
    routes.sort_by(|left, right| left.path.cmp(&right.path));
    let mut artifacts = manifest
        .artifacts
        .iter()
        .map(node_artifact)
        .collect::<Result<Vec<_>, _>>()?;
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    if artifacts
        .windows(2)
        .any(|pair| pair[0].path == pair[1].path)
    {
        return Err(NodeDeploymentErrorV1 {
            code: "PSNODE1010_ARTIFACT_PROJECTION_INVALID",
            message: "compiler artifact inventory contains duplicate paths".into(),
        });
    }
    let release_input = routes
        .iter()
        .map(|route| {
            format!(
                "{}:{}:{}:{}:{}\n",
                route.path,
                route.artifact_root,
                route.execution,
                route.loader_count,
                route.server_action_count
            )
        })
        .chain(
            artifacts
                .iter()
                .map(|artifact| format!("{}:{}\n", artifact.path, artifact.digest)),
        )
        .collect::<String>();
    Ok(NodeDeploymentPlanV1 {
        schema_version: NODE_DEPLOYMENT_SCHEMA_VERSION,
        provider: "node".into(),
        application_name: options.application_name.clone(),
        release_id: format!("sha256:{:x}", Sha256::digest(release_input)),
        routes,
        artifacts,
    })
}

/// Verifies the output directory against the compiler-authored release inventory.
pub fn validate_node_deployment_artifacts_v1(
    output_root: &Path,
    plan: &NodeDeploymentPlanV1,
) -> Result<(), NodeDeploymentErrorV1> {
    for artifact in &plan.artifacts {
        let path = output_root.join(&artifact.path);
        let bytes = fs::read(&path).map_err(|error| NodeDeploymentErrorV1 {
            code: "PSNODE1008_ARTIFACT_READ_FAILED",
            message: format!("{}: {error}", path.display()),
        })?;
        if format!("sha256:{:x}", Sha256::digest(&bytes)) != artifact.digest {
            return Err(NodeDeploymentErrorV1 {
                code: "PSNODE1009_ARTIFACT_INTEGRITY_MISMATCH",
                message: artifact.path.clone(),
            });
        }
    }
    Ok(())
}

#[must_use]
pub fn node_deployment_plan_json_v1(plan: &NodeDeploymentPlanV1) -> String {
    serde_json::to_string_pretty(plan).expect("Node deployment plan serializes") + "\n"
}

/// Emits a small Node static host that consumes the release plan verbatim.
/// Server-bound routes fail loudly rather than pretending that loaders or
/// server actions were executed.
#[must_use]
pub fn node_static_host_module_v1(plan: &NodeDeploymentPlanV1) -> String {
    let routes = serde_json::to_string(&plan.routes).expect("Node routes serialize");
    format!(
        "// Generated by Presolve. Do not edit.\n\
import {{ createServer }} from 'node:http';\n\
import {{ readFile }} from 'node:fs/promises';\n\
import {{ resolve }} from 'node:path';\n\
import {{ fileURLToPath }} from 'node:url';\n\
const routes = {routes};\n\
const outputRoot = fileURLToPath(new URL('../../dist/', import.meta.url));\n\
const port = Number(process.env.PORT || '3000');\n\
function segments(pathname) {{ return pathname.split('/').filter(Boolean); }}\n\
function score(parts) {{ return [parts.filter((part) => !part.startsWith(':')).length, parts.length]; }}\n\
function routeFor(requested) {{\n\
  let selected;\n\
  for (const route of routes) {{\n\
    const parts = segments(route.path);\n\
    if (requested.length < parts.length) continue;\n\
    if (!parts.every((part, index) => part.startsWith(':') ? requested[index]?.length > 0 : part === requested[index])) continue;\n\
    const candidate = score(parts);\n\
    if (!selected || candidate[0] > selected.score[0] || (candidate[0] === selected.score[0] && candidate[1] > selected.score[1])) selected = {{ route, parts, score: candidate }};\n\
  }}\n\
  return selected;\n\
}}\n\
function safeSegments(values) {{ return values.every((value) => value && value !== '.' && value !== '..' && !value.includes('\\\\')); }}\n\
createServer(async (request, response) => {{\n\
  if (request.method !== 'GET' && request.method !== 'HEAD') {{ response.writeHead(405, {{ Allow: 'GET, HEAD' }}); response.end(); return; }}\n\
  const url = new URL(request.url, 'http://presolve.local');\n\
  const requested = segments(url.pathname);\n\
  const selected = routeFor(requested);\n\
  if (!selected) {{\n\
    if (!safeSegments(requested) || requested.length === 0) {{ response.writeHead(404); response.end('Not Found\\n'); return; }}\n\
    try {{ const bytes = await readFile(resolve(outputRoot, requested.join('/'))); response.writeHead(200); response.end(request.method === 'HEAD' ? undefined : bytes); return; }}\n\
    catch {{ response.writeHead(404); response.end('Not Found\\n'); return; }}\n\
  }}\n\
  if (selected.route.execution !== 'static') {{ response.writeHead(501); response.end('Presolve Node executor required for this route\\n'); return; }}\n\
  const suffix = requested.slice(selected.parts.length);\n\
  if (!safeSegments(suffix)) {{ response.writeHead(404); response.end('Not Found\\n'); return; }}\n\
  const relative = suffix.length === 0 ? `${{selected.route.artifactRoot}}/index.html` : `${{selected.route.artifactRoot}}/${{suffix.join('/')}}`;\n\
  try {{ const bytes = await readFile(resolve(outputRoot, relative)); response.writeHead(200); response.end(request.method === 'HEAD' ? undefined : bytes); }}\n\
  catch {{\n\
    if (!safeSegments(requested) || requested.length === 0) {{ response.writeHead(404); response.end('Not Found\\n'); return; }}\n\
    try {{ const bytes = await readFile(resolve(outputRoot, requested.join('/'))); response.writeHead(200); response.end(request.method === 'HEAD' ? undefined : bytes); }}\n\
    catch {{ response.writeHead(404); response.end('Not Found\\n'); }}\n\
  }}\n\
}}).listen(port, '0.0.0.0', () => console.log(`Presolve Node release listening on ${{port}}`));\n"
    )
}

fn handoff_counts(
    values: impl IntoIterator<Item = (String, usize)>,
    expected_paths: &BTreeSet<String>,
    kind: &str,
) -> Result<BTreeMap<String, usize>, NodeDeploymentErrorV1> {
    let values = values.into_iter().collect::<Vec<_>>();
    let counts = values.iter().cloned().collect::<BTreeMap<_, _>>();
    if values.len() != counts.len()
        || counts.len() != expected_paths.len()
        || counts.keys().collect::<BTreeSet<_>>() != expected_paths.iter().collect::<BTreeSet<_>>()
    {
        return Err(NodeDeploymentErrorV1 {
            code: "PSNODE1007_HANDOFF_ROUTE_SET_MISMATCH",
            message: format!(
                "{kind} handoff routes do not exactly match the compiler route manifest"
            ),
        });
    }
    Ok(counts)
}

fn node_artifact(
    artifact: &ApplicationPublicationArtifactV1,
) -> Result<NodeDeploymentArtifactV1, NodeDeploymentErrorV1> {
    if !is_artifact_path(&artifact.path) || !artifact.digest.starts_with("sha256:") {
        return Err(NodeDeploymentErrorV1 {
            code: "PSNODE1010_ARTIFACT_PROJECTION_INVALID",
            message: artifact.path.clone(),
        });
    }
    Ok(NodeDeploymentArtifactV1 {
        path: artifact.path.clone(),
        digest: artifact.digest.clone(),
    })
}

fn is_application_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
}

fn is_artifact_path(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && value.split('/').all(|segment| {
            !segment.is_empty() && segment != "." && segment != ".." && !segment.contains('\\')
        })
}

fn is_route_path(value: &str) -> bool {
    value.starts_with('/') && !value.contains('?') && !value.contains('#') && !value.contains('\\')
}

#[cfg(test)]
mod tests {
    use presolve_compiler::{
        ApplicationPublicationArtifactV1, FileRoutePublicationManifestV1,
        FileRoutePublicationRouteV1,
    };

    use super::{
        build_node_deployment_plan_v1, node_static_host_module_v1, NodeDeploymentOptionsV1,
    };

    fn manifest() -> FileRoutePublicationManifestV1 {
        FileRoutePublicationManifestV1 {
            schema_version: 1,
            compiler_contract: "presolve-file-route-publication:1".into(),
            profile: "production".into(),
            routes: vec![
                FileRoutePublicationRouteV1 {
                    path: "/".into(),
                    entry_component_id: "component:home".into(),
                    artifact_root: "routes/root".into(),
                    layout_component_ids: Vec::new(),
                },
                FileRoutePublicationRouteV1 {
                    path: "/posts/:slug".into(),
                    entry_component_id: "component:post".into(),
                    artifact_root: "routes/segment-posts/parameter-slug".into(),
                    layout_component_ids: Vec::new(),
                },
            ],
            artifacts: vec![ApplicationPublicationArtifactV1 {
                path: "routes/root/index.html".into(),
                digest: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .into(),
            }],
        }
    }

    #[test]
    fn marks_only_compiler_proven_handoff_free_routes_static() {
        let plan = build_node_deployment_plan_v1(
            &manifest(),
            r#"{"schemaVersion":1,"routes":[{"path":"/","loaders":[]},{"path":"/posts/:slug","loaders":[{}]}]}"#,
            r#"{"schemaVersion":1,"routes":[{"path":"/","actions":[]},{"path":"/posts/:slug","actions":[]}]}"#,
            &NodeDeploymentOptionsV1 {
                application_name: "presolve-docs".into(),
            },
        )
        .unwrap();
        assert_eq!(plan.routes[0].execution, "static");
        assert_eq!(plan.routes[1].execution, "node");
        let host = node_static_host_module_v1(&plan);
        assert!(host.contains("executor required"));
        assert!(host.contains("routes/segment-posts/parameter-slug"));
    }

    #[test]
    fn rejects_handoff_route_sets_that_do_not_equal_the_manifest() {
        let error = build_node_deployment_plan_v1(
            &manifest(),
            r#"{"schemaVersion":1,"routes":[{"path":"/","loaders":[]}]}"#,
            r#"{"schemaVersion":1,"routes":[{"path":"/","actions":[]},{"path":"/posts/:slug","actions":[]}]}"#,
            &NodeDeploymentOptionsV1 {
                application_name: "presolve-docs".into(),
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "PSNODE1007_HANDOFF_ROUTE_SET_MISMATCH");
    }
}
