//! Cloudflare Workers Static Assets projection over compiler-published routes.
//!
//! The adapter validates compiler artifacts and executes only a compiler-issued
//! route table. It never reads application source or adds a framework runtime.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use presolve_compiler::{ApplicationPublicationArtifactV1, FileRoutePublicationManifestV1};
use serde::Serialize;
use sha2::{Digest as _, Sha256};

pub const CLOUDFLARE_WORKERS_STATIC_DEPLOYMENT_SCHEMA_VERSION: u32 = 1;
/// Tested Workers compatibility target for the current Presolve release train.
pub const CLOUDFLARE_WORKERS_COMPATIBILITY_DATE_V1: &str = "2026-07-23";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudflareWorkersDeploymentOptionsV1 {
    pub worker_name: String,
    pub compatibility_date: String,
    pub required_secrets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudflareWorkersDeploymentErrorV1 {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CloudflareWorkersStaticDeploymentPlanV1 {
    pub schema_version: u32,
    pub provider: String,
    pub worker_name: String,
    pub compatibility_date: String,
    pub release_id: String,
    pub required_secrets: Vec<String>,
    pub assets_binding: String,
    pub routes: Vec<CloudflareWorkersStaticRouteV1>,
    pub artifacts: Vec<CloudflareWorkersArtifactV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CloudflareWorkersStaticRouteV1 {
    pub path: String,
    pub artifact_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CloudflareWorkersArtifactV1 {
    pub path: String,
    pub digest: String,
}

/// Projects one compiler file-route manifest into immutable Cloudflare facts.
///
/// # Errors
///
/// Returns stable errors for invalid public configuration or unsafe compiler
/// artifact paths. This does not read application source.
pub fn build_cloudflare_workers_static_deployment_plan_v1(
    manifest: &FileRoutePublicationManifestV1,
    options: &CloudflareWorkersDeploymentOptionsV1,
) -> Result<CloudflareWorkersStaticDeploymentPlanV1, CloudflareWorkersDeploymentErrorV1> {
    if !is_worker_name(&options.worker_name) {
        return Err(CloudflareWorkersDeploymentErrorV1 {
            code: "PSCFL1001_WORKER_NAME_INVALID",
            message: options.worker_name.clone(),
        });
    }
    if !is_compatibility_date(&options.compatibility_date) {
        return Err(CloudflareWorkersDeploymentErrorV1 {
            code: "PSCFL1002_COMPATIBILITY_DATE_INVALID",
            message: options.compatibility_date.clone(),
        });
    }
    let required_secrets = canonical_secret_names(&options.required_secrets)?;
    let mut artifacts = manifest
        .artifacts
        .iter()
        .map(cloudflare_artifact)
        .collect::<Result<Vec<_>, _>>()?;
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    let routes = manifest
        .routes
        .iter()
        .map(|route| CloudflareWorkersStaticRouteV1 {
            path: route.path.clone(),
            artifact_root: route.artifact_root.clone(),
        })
        .collect::<Vec<_>>();
    if routes.is_empty() {
        return Err(CloudflareWorkersDeploymentErrorV1 {
            code: "PSCFL1003_ROUTE_SET_EMPTY",
            message: "Cloudflare deployment requires at least one compiler route".into(),
        });
    }
    if routes
        .iter()
        .any(|route| !is_route_path(&route.path) || !is_artifact_root(&route.artifact_root))
    {
        return Err(CloudflareWorkersDeploymentErrorV1 {
            code: "PSCFL1004_ROUTE_PROJECTION_INVALID",
            message: "compiler route manifest contains an unsafe Cloudflare projection".into(),
        });
    }
    let release_id = format!(
        "sha256:{:x}",
        Sha256::digest(
            artifacts
                .iter()
                .map(|artifact| format!("{}:{}\n", artifact.path, artifact.digest))
                .collect::<String>(),
        )
    );
    Ok(CloudflareWorkersStaticDeploymentPlanV1 {
        schema_version: CLOUDFLARE_WORKERS_STATIC_DEPLOYMENT_SCHEMA_VERSION,
        provider: "cloudflare_workers_static_assets".into(),
        worker_name: options.worker_name.clone(),
        compatibility_date: options.compatibility_date.clone(),
        release_id,
        required_secrets,
        assets_binding: "ASSETS".into(),
        routes,
        artifacts,
    })
}

/// Confirms that provider-upload bytes still equal the compiler inventory.
///
/// # Errors
///
/// Returns an error for a missing or modified artifact.
pub fn validate_cloudflare_workers_artifacts_v1(
    output_root: &Path,
    plan: &CloudflareWorkersStaticDeploymentPlanV1,
) -> Result<(), CloudflareWorkersDeploymentErrorV1> {
    for artifact in &plan.artifacts {
        let path = output_root.join(&artifact.path);
        let bytes = fs::read(&path).map_err(|error| CloudflareWorkersDeploymentErrorV1 {
            code: "PSCFL1005_ARTIFACT_READ_FAILED",
            message: format!("{}: {error}", path.display()),
        })?;
        if format!("sha256:{:x}", Sha256::digest(&bytes)) != artifact.digest {
            return Err(CloudflareWorkersDeploymentErrorV1 {
                code: "PSCFL1006_ARTIFACT_INTEGRITY_MISMATCH",
                message: artifact.path.clone(),
            });
        }
    }
    Ok(())
}

#[must_use]
pub fn cloudflare_workers_static_plan_json_v1(
    plan: &CloudflareWorkersStaticDeploymentPlanV1,
) -> String {
    serde_json::to_string_pretty(plan).expect("Cloudflare static plan serializes") + "\n"
}

/// Generates a Worker that consumes only the canonical route table. It is a
/// static asset executor, not a generic server or client router.
#[must_use]
pub fn cloudflare_workers_static_worker_module_v1(
    plan: &CloudflareWorkersStaticDeploymentPlanV1,
) -> String {
    let routes = serde_json::to_string(&plan.routes).expect("Cloudflare routes serialize");
    format!(
        "// Generated by Presolve. Do not edit.\n\
const routes = {routes};\n\
function segments(pathname) {{ return pathname.split('/').filter(Boolean); }}\n\
function score(parts) {{ return [parts.filter((value) => !value.startsWith(':')).length, parts.length]; }}\n\
function routeFor(requested) {{\n\
  let selected;\n\
  for (const route of routes) {{\n\
    const parts = segments(route.path);\n\
    if (requested.length < parts.length) continue;\n\
    if (!parts.every((part, index) => part.startsWith(':') ? requested[index].length > 0 : part === requested[index])) continue;\n\
    const candidate = score(parts);\n\
    if (!selected || candidate[0] > selected.score[0] || (candidate[0] === selected.score[0] && candidate[1] > selected.score[1])) selected = {{ route, parts, score: candidate }};\n\
  }}\n\
  return selected;\n\
}}\n\
function safeAssetSegments(values) {{ return values.every((value) => value && value !== '.' && value !== '..' && !value.includes('\\\\')); }}\n\
async function fetchCompilerAsset(request, env, artifactUrl) {{\n\
  const response = await env.ASSETS.fetch(new Request(artifactUrl, request));\n\
  if (response.status < 300 || response.status > 399) return response;\n\
  const location = response.headers.get('Location');\n\
  if (!location) return response;\n\
  const redirected = new URL(location, artifactUrl);\n\
  const internal = new URL(artifactUrl);\n\
  if (redirected.origin !== internal.origin || !redirected.pathname.startsWith('/routes/')) return response;\n\
  return env.ASSETS.fetch(new Request(redirected, request));\n\
}}\n\
export default {{ async fetch(request, env) {{\n\
  if (request.method !== 'GET' && request.method !== 'HEAD') return new Response('Method Not Allowed\\n', {{ status: 405, headers: {{ Allow: 'GET, HEAD' }} }});\n\
  const url = new URL(request.url);\n\
  const direct = await env.ASSETS.fetch(request);\n\
  if (direct.status !== 404) {{\n\
    const location = direct.headers.get('Location');\n\
    if (location) {{\n\
      const redirected = new URL(location, url);\n\
      if (redirected.origin === url.origin && redirected.pathname.startsWith('/routes/')) return fetchCompilerAsset(request, env, redirected);\n\
    }}\n\
    return direct;\n\
  }}\n\
  const requested = segments(url.pathname);\n\
  const selected = routeFor(requested);\n\
  if (!selected) return new Response('Not Found\\n', {{ status: 404 }});\n\
  if (requested.length === selected.parts.length) {{\n\
    if (selected.route.path !== '/' && !url.pathname.endsWith('/')) return Response.redirect(new URL(`${{url.pathname}}/`, url).toString(), 308);\n\
    return fetchCompilerAsset(request, env, new URL(`/${{selected.route.artifact_root}}/index.html`, url));\n\
  }}\n\
  const suffix = requested.slice(selected.parts.length);\n\
  if (!safeAssetSegments(suffix)) return new Response('Not Found\\n', {{ status: 404 }});\n\
  return fetchCompilerAsset(request, env, new URL(`/${{selected.route.artifact_root}}/${{suffix.join('/')}}`, url));\n\
}} }};\n"
    )
}

#[must_use]
pub fn cloudflare_workers_wrangler_jsonc_v1(
    plan: &CloudflareWorkersStaticDeploymentPlanV1,
    assets_directory: &str,
) -> String {
    let mut value = serde_json::json!({
        "name": plan.worker_name,
        "main": "worker.mjs",
        "compatibility_date": plan.compatibility_date,
        "assets": {
            "directory": assets_directory,
            "binding": plan.assets_binding,
            "run_worker_first": true,
        },
    });
    if !plan.required_secrets.is_empty() {
        value["secrets"] = serde_json::json!({ "required": plan.required_secrets });
    }
    serde_json::to_string_pretty(&value).expect("Wrangler configuration serializes") + "\n"
}

fn cloudflare_artifact(
    artifact: &ApplicationPublicationArtifactV1,
) -> Result<CloudflareWorkersArtifactV1, CloudflareWorkersDeploymentErrorV1> {
    if !is_artifact_path(&artifact.path) || !artifact.digest.starts_with("sha256:") {
        return Err(CloudflareWorkersDeploymentErrorV1 {
            code: "PSCFL1007_ARTIFACT_PROJECTION_INVALID",
            message: artifact.path.clone(),
        });
    }
    Ok(CloudflareWorkersArtifactV1 {
        path: artifact.path.clone(),
        digest: artifact.digest.clone(),
    })
}

fn canonical_secret_names(
    values: &[String],
) -> Result<Vec<String>, CloudflareWorkersDeploymentErrorV1> {
    let mut names = BTreeSet::new();
    for value in values {
        if !is_secret_name(value) {
            return Err(CloudflareWorkersDeploymentErrorV1 {
                code: "PSCFL1008_SECRET_NAME_INVALID",
                message: value.clone(),
            });
        }
        names.insert(value.clone());
    }
    Ok(names.into_iter().collect())
}

fn is_worker_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
}

fn is_compatibility_date(value: &str) -> bool {
    value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .bytes()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}

fn is_secret_name(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase() || character == '_')
        && value.chars().all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        })
}

fn is_route_path(value: &str) -> bool {
    value.starts_with('/') && !value.contains('?') && !value.contains('#') && !value.contains('\\')
}

fn is_artifact_root(value: &str) -> bool {
    is_artifact_path(value) && value.starts_with("routes/")
}

fn is_artifact_path(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && value.split('/').all(|segment| {
            !segment.is_empty() && segment != "." && segment != ".." && !segment.contains('\\')
        })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use presolve_compiler::{
        ApplicationPublicationArtifactV1, FileRoutePublicationManifestV1,
        FileRoutePublicationRouteV1,
    };
    use sha2::{Digest as _, Sha256};

    use super::{
        build_cloudflare_workers_static_deployment_plan_v1,
        cloudflare_workers_static_worker_module_v1, cloudflare_workers_wrangler_jsonc_v1,
        validate_cloudflare_workers_artifacts_v1, CloudflareWorkersDeploymentOptionsV1,
        CLOUDFLARE_WORKERS_COMPATIBILITY_DATE_V1,
    };

    fn manifest(digest: String) -> FileRoutePublicationManifestV1 {
        FileRoutePublicationManifestV1 {
            schema_version: 1,
            compiler_contract: "presolve-file-route-publication:1".into(),
            profile: "production".into(),
            routes: vec![FileRoutePublicationRouteV1 {
                path: "/posts/:slug".into(),
                entry_component_id: "component:post".into(),
                artifact_root: "routes/segment-posts/parameter-slug".into(),
                layout_component_ids: Vec::new(),
            }],
            artifacts: vec![ApplicationPublicationArtifactV1 {
                path: "routes/segment-posts/parameter-slug/index.html".into(),
                digest,
            }],
        }
    }

    #[test]
    fn projects_only_compiler_owned_routes_artifacts_and_public_bindings() {
        let plan = build_cloudflare_workers_static_deployment_plan_v1(
            &manifest(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            ),
            &CloudflareWorkersDeploymentOptionsV1 {
                worker_name: "presolve-docs".into(),
                compatibility_date: CLOUDFLARE_WORKERS_COMPATIBILITY_DATE_V1.into(),
                required_secrets: vec!["DOCS_TOKEN".into()],
            },
        )
        .unwrap();
        assert_eq!(plan.routes[0].path, "/posts/:slug");
        assert_eq!(plan.required_secrets, ["DOCS_TOKEN"]);
        let worker = cloudflare_workers_static_worker_module_v1(&plan);
        assert!(worker.contains("routeFor"));
        assert!(worker.contains("routes/segment-posts/parameter-slug"));
        assert!(worker.contains("fetchCompilerAsset"));
        assert!(worker.contains("redirected.pathname.startsWith('/routes/')"));
        assert!(worker.contains("if (location)"));
        assert!(!worker.contains("url.pathname === '/' && location"));
        assert!(!worker.contains("component:post"));
        let config = cloudflare_workers_wrangler_jsonc_v1(&plan, "../../dist");
        assert!(config.contains("run_worker_first"));
        assert!(config.contains("DOCS_TOKEN"));
    }

    #[test]
    fn rejects_mutated_compiler_artifacts_before_upload() {
        let bytes = b"<main>post</main>".to_vec();
        let digest = format!("sha256:{:x}", Sha256::digest(&bytes));
        let plan = build_cloudflare_workers_static_deployment_plan_v1(
            &manifest(digest),
            &CloudflareWorkersDeploymentOptionsV1 {
                worker_name: "presolve-docs".into(),
                compatibility_date: CLOUDFLARE_WORKERS_COMPATIBILITY_DATE_V1.into(),
                required_secrets: Vec::new(),
            },
        )
        .unwrap();
        let root =
            std::env::temp_dir().join(format!("presolve-cloudflare-plan-{}", std::process::id()));
        let path = root.join("routes/segment-posts/parameter-slug/index.html");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &bytes).unwrap();
        assert!(validate_cloudflare_workers_artifacts_v1(&root, &plan).is_ok());
        fs::write(&path, b"tampered").unwrap();
        assert_eq!(
            validate_cloudflare_workers_artifacts_v1(&root, &plan)
                .unwrap_err()
                .code,
            "PSCFL1006_ARTIFACT_INTEGRITY_MISMATCH"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
