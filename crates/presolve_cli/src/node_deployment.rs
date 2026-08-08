//! Node release inventory projected from compiler-published file routes.
//!
//! This adapter consumes only the immutable route manifest and the compiler's
//! loader/server-action handoffs. It hosts compiler-proven static routes and
//! canonical Form-bound server actions and route loaders.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use presolve_compiler::{
    ApplicationPublicationArtifactV1, FileRoutePublicationManifestV1, ResumeValueCodec,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

pub const NODE_DEPLOYMENT_SCHEMA_VERSION: u32 = 3;

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
    pub server_actions: Vec<NodeServerActionV1>,
    pub route_loaders: Vec<NodeRouteLoaderV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_action_registry: Option<NodeDeploymentArtifactV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_loader_registry: Option<NodeDeploymentArtifactV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeDeploymentRouteV1 {
    pub path: String,
    pub artifact_root: String,
    /// `static` is compiler-proven exportable. `node` executes an admitted
    /// capability-specific route loader or server action.
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NodeServerActionV1 {
    pub id: String,
    pub route_path: String,
    pub component_id: String,
    pub form: String,
    pub request_path: String,
    pub package: String,
    pub version: String,
    pub integrity: String,
    pub export: String,
    pub runtime_module: String,
    pub input: String,
    pub response: String,
    pub failure: String,
    pub cancellation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NodeRouteLoaderV1 {
    pub id: String,
    pub route_path: String,
    pub component_id: String,
    pub field: String,
    pub package: String,
    pub version: String,
    pub integrity: String,
    pub export: String,
    pub runtime_module: String,
    pub cache_scope: String,
    pub max_age_seconds: Option<u64>,
    pub failure: String,
    pub data_codec: ResumeValueCodec,
    pub error_codec: ResumeValueCodec,
    pub resource_activation_id: String,
    pub bootstrap_key: String,
    pub parameters: Vec<NodeRouteLoaderParameterV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NodeRouteLoaderParameterV1 {
    pub name: String,
    pub segment_index: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoaderPlanV2 {
    #[serde(alias = "schema_version")]
    schema_version: u32,
    routes: Vec<LoaderPlanRouteV2>,
}

#[derive(Debug, Deserialize)]
struct LoaderPlanRouteV2 {
    path: String,
    loaders: Vec<LoaderPlanBindingV2>,
}

#[derive(Debug, Deserialize)]
struct LoaderPlanBindingV2 {
    id: String,
    component_id: String,
    field: String,
    package: String,
    version: String,
    integrity: String,
    export: String,
    runtime_module: String,
    cache_scope: String,
    max_age_seconds: Option<u64>,
    failure: String,
    data_codec: ResumeValueCodec,
    error_codec: ResumeValueCodec,
    resource_activation_id: String,
    parameters: Vec<LoaderPlanParameterV2>,
}

#[derive(Debug, Deserialize)]
struct LoaderPlanParameterV2 {
    name: String,
    segment_index: usize,
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
    actions: Vec<ServerActionPlanBindingV2>,
}

#[derive(Debug, Deserialize)]
struct ServerActionPlanBindingV2 {
    id: String,
    source: String,
    component_id: String,
    form: Option<String>,
    request_path: Option<String>,
    package: String,
    version: String,
    integrity: String,
    export: String,
    runtime_module: String,
    input: String,
    response: String,
    failure: String,
    cancellation: String,
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
    let loaders: LoaderPlanV2 =
        serde_json::from_str(loader_plan_source).map_err(|error| NodeDeploymentErrorV1 {
            code: "PSNODE1003_LOADER_HANDOFF_INVALID",
            message: error.to_string(),
        })?;
    let actions: ServerActionPlanV1 =
        serde_json::from_str(server_action_plan_source).map_err(|error| NodeDeploymentErrorV1 {
            code: "PSNODE1004_SERVER_ACTION_HANDOFF_INVALID",
            message: error.to_string(),
        })?;
    if loaders.schema_version != 2 || actions.schema_version != 2 {
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
    let mut executable_loaders = Vec::new();
    let loader_counts = handoff_counts(
        loaders.routes.into_iter().map(|route| {
            let count = route.loaders.len();
            for loader in route.loaders {
                executable_loaders.push(NodeRouteLoaderV1 {
                    id: loader.id,
                    route_path: route.path.clone(),
                    component_id: loader.component_id,
                    field: loader.field,
                    package: loader.package,
                    version: loader.version,
                    integrity: loader.integrity,
                    export: loader.export,
                    runtime_module: loader.runtime_module,
                    cache_scope: loader.cache_scope,
                    max_age_seconds: loader.max_age_seconds,
                    failure: loader.failure,
                    data_codec: loader.data_codec,
                    error_codec: loader.error_codec,
                    bootstrap_key: loader.resource_activation_id.clone(),
                    resource_activation_id: loader.resource_activation_id,
                    parameters: loader
                        .parameters
                        .into_iter()
                        .map(|parameter| NodeRouteLoaderParameterV1 {
                            name: parameter.name,
                            segment_index: parameter.segment_index,
                        })
                        .collect(),
                });
            }
            (route.path, count)
        }),
        &route_paths,
        "loader",
    )?;
    executable_loaders.sort_by(|left, right| left.id.cmp(&right.id));
    validate_route_loaders(&executable_loaders, &route_paths)?;
    let mut executable_actions = Vec::new();
    let action_counts = handoff_counts(
        actions.routes.into_iter().map(|route| {
            let count = route.actions.len();
            for action in route.actions {
                if action.source != "canonical_form" {
                    continue;
                }
                let (Some(form), Some(request_path)) = (action.form, action.request_path) else {
                    continue;
                };
                executable_actions.push(NodeServerActionV1 {
                    id: action.id,
                    route_path: route.path.clone(),
                    component_id: action.component_id,
                    form,
                    request_path,
                    package: action.package,
                    version: action.version,
                    integrity: action.integrity,
                    export: action.export,
                    runtime_module: action.runtime_module,
                    input: action.input,
                    response: action.response,
                    failure: action.failure,
                    cancellation: action.cancellation,
                });
            }
            (route.path, count)
        }),
        &route_paths,
        "server-action",
    )?;
    executable_actions.sort_by(|left, right| left.id.cmp(&right.id));
    validate_server_actions(&executable_actions, &route_paths)?;
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
    let mut plan = NodeDeploymentPlanV1 {
        schema_version: NODE_DEPLOYMENT_SCHEMA_VERSION,
        provider: "node".into(),
        application_name: options.application_name.clone(),
        release_id: String::new(),
        routes,
        artifacts,
        server_actions: executable_actions,
        route_loaders: executable_loaders,
        server_action_registry: None,
        route_loader_registry: None,
    };
    plan.release_id = release_id(&plan);
    Ok(plan)
}

/// Adds the digest-bound server registry to a fully validated plan.
pub fn attach_node_server_action_registry_v1(
    plan: &mut NodeDeploymentPlanV1,
    digest: String,
) -> Result<(), NodeDeploymentErrorV1> {
    if plan.server_actions.is_empty() {
        return Err(NodeDeploymentErrorV1 {
            code: "PSNODE1020_SERVER_ACTION_REGISTRY_UNUSED",
            message: "cannot attach a server-action registry to a handoff-free release".into(),
        });
    }
    if !is_sha256_digest(&digest) {
        return Err(NodeDeploymentErrorV1 {
            code: "PSNODE1021_SERVER_ACTION_REGISTRY_INVALID",
            message: digest,
        });
    }
    plan.server_action_registry = Some(NodeDeploymentArtifactV1 {
        path: "presolve.server-actions.mjs".into(),
        digest,
    });
    plan.release_id = release_id(plan);
    Ok(())
}

/// Adds the digest-bound route-loader registry to a validated Node plan.
pub fn attach_node_route_loader_registry_v1(
    plan: &mut NodeDeploymentPlanV1,
    digest: String,
) -> Result<(), NodeDeploymentErrorV1> {
    if plan.route_loaders.is_empty() {
        return Err(NodeDeploymentErrorV1 {
            code: "PSNODE1022_ROUTE_LOADER_REGISTRY_UNUSED",
            message: "cannot attach a route-loader registry to a loader-free release".into(),
        });
    }
    if !is_sha256_digest(&digest) {
        return Err(NodeDeploymentErrorV1 {
            code: "PSNODE1023_ROUTE_LOADER_REGISTRY_INVALID",
            message: digest,
        });
    }
    plan.route_loader_registry = Some(NodeDeploymentArtifactV1 {
        path: "presolve.route-loaders.mjs".into(),
        digest,
    });
    plan.release_id = release_id(plan);
    Ok(())
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

/// Emits the Node host from compiler-issued route-loader and server-action records.
#[must_use]
pub fn node_static_host_module_v1(plan: &NodeDeploymentPlanV1) -> String {
    let routes = serde_json::to_string(&plan.routes).expect("Node routes serialize");
    let actions = serde_json::to_string(&plan.server_actions).expect("Node actions serialize");
    let loaders = serde_json::to_string(&plan.route_loaders).expect("Node loaders serialize");
    let registry = serde_json::to_string(&plan.server_action_registry)
        .expect("Node server registry serializes");
    let loader_registry = serde_json::to_string(&plan.route_loader_registry)
        .expect("Node loader registry serializes");
    r#"// Generated by Presolve. Do not edit.
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
const routes = __PRESOLVE_ROUTES__;
const actions = __PRESOLVE_ACTIONS__;
const loaders = __PRESOLVE_LOADERS__;
const registryArtifact = __PRESOLVE_REGISTRY__;
const loaderRegistryArtifact = __PRESOLVE_LOADER_REGISTRY__;
const outputRoot = fileURLToPath(new URL('../../dist/', import.meta.url));
const adapterRoot = fileURLToPath(new URL('./', import.meta.url));
const port = Number(process.env.PORT || '3000');
const bodyLimit = 8 * 1024 * 1024;
function segments(pathname) { return pathname.split('/').filter(Boolean); }
function decodedSegments(pathname) {
  try {
    return segments(pathname).map((segment) => {
      const decoded = decodeURIComponent(segment);
      if (!decoded || decoded === '.' || decoded === '..' || decoded.includes('/') || decoded.includes('\\') || decoded.includes('\0')) throw new Error('invalid route segment');
      return decoded;
    });
  } catch { return null; }
}
function score(parts) { return [parts.filter((part) => !part.startsWith(':')).length, parts.length]; }
function routeFor(requested) {
  let selected;
  for (const route of routes) {
    const parts = segments(route.path);
    if (requested.length < parts.length) continue;
    if (!parts.every((part, index) => part.startsWith(':') ? requested[index]?.length > 0 : part === requested[index])) continue;
    const candidate = score(parts);
    if (!selected || candidate[0] > selected.score[0] || (candidate[0] === selected.score[0] && candidate[1] > selected.score[1])) selected = { route, parts, score: candidate };
  }
  return selected;
}
function safeSegments(values) { return values.every((value) => value && value !== '.' && value !== '..' && !value.includes('\\')); }
function json(response, status, value) {
  const body = JSON.stringify(value);
  response.writeHead(status, { 'Content-Type': 'application/json; charset=utf-8', 'Cache-Control': 'no-store', 'Content-Length': Buffer.byteLength(body) });
  response.end(body);
}
function jsonValue(value, seen = new Set(), depth = 0) {
  if (depth > 64 || value === undefined || typeof value === 'function' || typeof value === 'symbol' || typeof value === 'bigint') return false;
  if (value === null || typeof value === 'string' || typeof value === 'boolean') return true;
  if (typeof value === 'number') return Number.isFinite(value);
  if (typeof value !== 'object' || seen.has(value)) return false;
  seen.add(value);
  const valid = Array.isArray(value)
    ? value.every((entry) => jsonValue(entry, seen, depth + 1))
    : Object.getPrototypeOf(value) === Object.prototype && Object.entries(value).every(([key, entry]) => key.length > 0 && jsonValue(entry, seen, depth + 1));
  seen.delete(value);
  return valid;
}
async function requestBody(request) {
  const chunks = [];
  let size = 0;
  for await (const chunk of request) {
    size += chunk.length;
    if (size > bodyLimit) { const error = new Error('body limit'); error.status = 413; throw error; }
    chunks.push(chunk);
  }
  return Buffer.concat(chunks);
}
function effectiveOrigin(request) {
  const forwarded = String(request.headers['x-forwarded-proto'] ?? '').split(',')[0].trim();
  const protocol = forwarded === 'https' ? 'https' : 'http';
  const host = request.headers.host;
  return typeof host === 'string' && host.length > 0 ? `${protocol}://${host}` : null;
}
async function loadActionRegistry() {
  if (actions.length === 0) return Object.freeze(Object.create(null));
  if (registryArtifact === null) throw new Error('PSNODE2001_SERVER_ACTION_REGISTRY_MISSING');
  const path = resolve(adapterRoot, registryArtifact.path);
  const bytes = await readFile(path);
  const digest = `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
  if (digest !== registryArtifact.digest) throw new Error('PSNODE2002_SERVER_ACTION_REGISTRY_INTEGRITY');
  const imported = await import(new URL(`./${registryArtifact.path}`, import.meta.url));
  const registry = imported.presolveServerActions;
  if (registry === null || typeof registry !== 'object' || Array.isArray(registry)
    || !actions.every((action) => typeof registry[action.id] === 'function')) {
    throw new Error('PSNODE2003_SERVER_ACTION_REGISTRY_MISMATCH');
  }
  return registry;
}
async function loadLoaderRegistry() {
  if (loaders.length === 0) return Object.freeze(Object.create(null));
  if (loaderRegistryArtifact === null) throw new Error('PSNODE2010_ROUTE_LOADER_REGISTRY_MISSING');
  const path = resolve(adapterRoot, loaderRegistryArtifact.path);
  const bytes = await readFile(path);
  const digest = `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
  if (digest !== loaderRegistryArtifact.digest) throw new Error('PSNODE2011_ROUTE_LOADER_REGISTRY_INTEGRITY');
  const imported = await import(new URL(`./${loaderRegistryArtifact.path}`, import.meta.url));
  const registry = imported.presolveRouteLoaders;
  if (registry === null || typeof registry !== 'object' || Array.isArray(registry)
    || !loaders.every((loader) => typeof registry[loader.id] === 'function')) {
    throw new Error('PSNODE2012_ROUTE_LOADER_REGISTRY_MISMATCH');
  }
  return registry;
}
const actionRegistry = await loadActionRegistry();
const loaderRegistry = await loadLoaderRegistry();
const actionsByPath = new Map(actions.map((action) => [action.requestPath, action]));
const loadersByRoute = new Map(routes.map((route) => [route.path, loaders.filter((loader) => loader.routePath === route.path)]));
const activeActionControllers = new Set();
const activeLoaderControllers = new Set();
const loaderCache = new Map();

function decodeLoaderValue(value, codec, depth = 0) {
  if (depth > 32 || codec === null || typeof codec !== 'object' || Array.isArray(codec)) throw new Error('codec');
  if (codec.kind === 'null_codec') { if (value !== null) throw new Error('codec'); return null; }
  if (codec.kind === 'boolean_codec') { if (typeof value !== 'boolean') throw new Error('codec'); return value; }
  if (codec.kind === 'number_codec') { if (typeof value !== 'number' || !Number.isFinite(value)) throw new Error('codec'); return value; }
  if (codec.kind === 'string_codec') { if (typeof value !== 'string') throw new Error('codec'); return value; }
  if (codec.kind === 'nullable_codec') return value === null ? null : decodeLoaderValue(value, codec.value, depth + 1);
  if (codec.kind === 'array_codec') {
    if (!Array.isArray(value)) throw new Error('codec');
    return value.map((entry) => decodeLoaderValue(entry, codec.value, depth + 1));
  }
  if (codec.kind === 'object_codec') {
    if (value === null || typeof value !== 'object' || Array.isArray(value) || Object.getPrototypeOf(value) !== Object.prototype) throw new Error('codec');
    const expected = codec.value.map((property) => property.name).sort();
    const actual = Object.keys(value).sort();
    if (JSON.stringify(expected) !== JSON.stringify(actual)) throw new Error('codec');
    return Object.fromEntries(codec.value.map((property) => [property.name, decodeLoaderValue(value[property.name], property.codec, depth + 1)]));
  }
  throw new Error('codec');
}

function canonicalParams(loader, requested) {
  return Object.freeze(Object.fromEntries(loader.parameters.map((parameter) => [parameter.name, requested[parameter.segmentIndex]])));
}
function loaderCacheKey(loader, params, request) {
  const partition = loader.cacheScope === 'private'
    ? createHash('sha256').update(String(request.headers.authorization ?? '')).update('\0').update(String(request.headers.cookie ?? '')).digest('hex')
    : '';
  return createHash('sha256').update(loader.id).update('\0').update(JSON.stringify(params)).update('\0').update(partition).digest('hex');
}
function abortRace(promise, signal) {
  if (signal.aborted) return Promise.reject(new DOMException('Aborted', 'AbortError'));
  return Promise.race([promise, new Promise((_, reject) => signal.addEventListener('abort', () => reject(new DOMException('Aborted', 'AbortError')), { once: true }))]);
}
async function invokeLoader(loader, params, signal) {
  try {
    const result = await loaderRegistry[loader.id](params, signal);
    if (signal.aborted) throw new DOMException('Aborted', 'AbortError');
    return { key: loader.bootstrapKey, state: 'ready', generation: 1, data: decodeLoaderValue(result, loader.dataCodec), error: null };
  } catch (error) {
    if (signal.aborted || error?.name === 'AbortError') throw error;
    try {
      return { key: loader.bootstrapKey, state: 'failed', generation: 1, data: null, error: decodeLoaderValue(error, loader.errorCodec) };
    } catch {
      throw new Error('PSNODE2013_ROUTE_LOADER_RESULT_MISMATCH');
    }
  }
}
async function executeLoader(loader, params, request, requestSignal) {
  if (loader.cacheScope === 'no_store') return invokeLoader(loader, params, requestSignal);
  const key = loaderCacheKey(loader, params, request);
  const now = Date.now();
  let entry = loaderCache.get(key);
  if (entry !== undefined && entry.expiresAt <= now) { entry.controller.abort(); loaderCache.delete(key); entry = undefined; }
  if (entry === undefined) {
    const controller = new AbortController();
    entry = { controller, waiters: 0, settled: false, expiresAt: now + loader.maxAgeSeconds * 1000 };
    entry.promise = invokeLoader(loader, params, controller.signal).then((value) => { entry.settled = true; return value; }, (error) => { loaderCache.delete(key); throw error; });
    loaderCache.set(key, entry);
  }
  entry.waiters += 1;
  try { return await abortRace(entry.promise, requestSignal); }
  finally {
    entry.waiters -= 1;
    if (!entry.settled && entry.waiters === 0) { entry.controller.abort(); loaderCache.delete(key); }
  }
}
function routeCacheHeaders(routeLoaders) {
  if (routeLoaders.some((loader) => loader.cacheScope === 'no_store')) return { 'Cache-Control': 'no-store' };
  const maxAge = Math.min(...routeLoaders.map((loader) => loader.maxAgeSeconds));
  return routeLoaders.some((loader) => loader.cacheScope === 'private')
    ? { 'Cache-Control': `private, max-age=${maxAge}`, Vary: 'Authorization, Cookie' }
    : { 'Cache-Control': `public, max-age=${maxAge}` };
}
function resourceBootstrapScript(values) {
  const json = JSON.stringify({ schema_version: 1, values }).replaceAll('<', '\\u003c').replaceAll('\u2028', '\\u2028').replaceAll('\u2029', '\\u2029');
  return `    <script id="presolve-resource-bootstrap" type="application/json">${json}</script>\n`;
}
async function dispatchAction(request, response, url, action) {
  if (request.method !== 'POST') { response.writeHead(405, { Allow: 'POST', 'Cache-Control': 'no-store' }); response.end(); return; }
  const origin = request.headers.origin;
  const expectedOrigin = effectiveOrigin(request);
  if (typeof origin === 'string' && origin !== expectedOrigin) { json(response, 403, { error: { code: 'PSNODE2004_ACTION_ORIGIN_REJECTED', message: 'Cross-origin server action rejected' } }); return; }
  const mediaType = String(request.headers['content-type'] ?? '').split(';')[0].trim().toLowerCase();
  if (mediaType !== 'multipart/form-data' && mediaType !== 'application/x-www-form-urlencoded') { json(response, 415, { error: { code: 'PSNODE2005_ACTION_MEDIA_TYPE_UNSUPPORTED', message: 'Server action requires form data' } }); return; }
  const declaredLength = Number(request.headers['content-length'] ?? '0');
  if (!Number.isSafeInteger(declaredLength) || declaredLength < 0) { json(response, 400, { error: { code: 'PSNODE2006_ACTION_FORM_DATA_INVALID', message: 'Malformed form data' } }); return; }
  if (declaredLength > bodyLimit) { json(response, 413, { error: { code: 'PSNODE2008_ACTION_BODY_TOO_LARGE', message: 'Server action body exceeded 8 MiB' } }); return; }
  const controller = new AbortController();
  activeActionControllers.add(controller);
  request.once('aborted', () => controller.abort());
  response.once('close', () => { if (!response.writableEnded) controller.abort(); });
  try {
    const bytes = await requestBody(request);
    if (controller.signal.aborted) return;
    const headers = new Headers();
    for (const [name, value] of Object.entries(request.headers)) {
      if (Array.isArray(value)) for (const entry of value) headers.append(name, entry);
      else if (value !== undefined) headers.set(name, value);
    }
    let formData;
    try { formData = await new Request(url, { method: 'POST', headers, body: bytes }).formData(); }
    catch { json(response, 400, { error: { code: 'PSNODE2006_ACTION_FORM_DATA_INVALID', message: 'Malformed form data' } }); return; }
    const result = await actionRegistry[action.id](formData, controller.signal);
    if (controller.signal.aborted || response.writableEnded) return;
    if (action.response === 'redirect') {
      const location = result?.location;
      if (result === null || typeof result !== 'object' || Array.isArray(result)
        || Object.keys(result).length !== 1 || typeof location !== 'string'
        || !location.startsWith('/') || location.startsWith('//') || location.includes('\\')) {
        throw new Error('PSNODE2007_ACTION_RESPONSE_MISMATCH');
      }
      response.writeHead(303, { Location: location, 'Cache-Control': 'no-store' }); response.end(); return;
    }
    if (!jsonValue(result)) throw new Error('PSNODE2007_ACTION_RESPONSE_MISMATCH');
    json(response, 200, result);
  } catch (error) {
    if (controller.signal.aborted || response.writableEnded) return;
    if (error?.status === 413) { json(response, 413, { error: { code: 'PSNODE2008_ACTION_BODY_TOO_LARGE', message: 'Server action body exceeded 8 MiB' } }); return; }
    const typed = error !== null && typeof error === 'object' && !Array.isArray(error)
      && Number.isInteger(error.status) && error.status >= 400 && error.status <= 599
      && typeof error.code === 'string' && error.code.length > 0
      && typeof error.message === 'string' && error.message.length > 0
      && (error.issues === undefined || jsonValue(error.issues));
    if (typed) { json(response, error.status, { error: { code: error.code, message: error.message, ...(error.issues === undefined ? {} : { issues: error.issues }) } }); return; }
    console.error('Presolve server action failed', { id: action.id, message: error instanceof Error ? error.message : String(error) });
    json(response, 500, { error: { code: 'PSNODE2009_ACTION_EXECUTION_FAILED', message: 'Server action failed' } });
  } finally {
    activeActionControllers.delete(controller);
  }
}
const host = createServer(async (request, response) => {
  const base = effectiveOrigin(request) ?? 'http://presolve.local';
  const url = new URL(request.url, base);
  const action = actionsByPath.get(url.pathname);
  if (action !== undefined) { await dispatchAction(request, response, url, action); return; }
  if (url.pathname.startsWith('/_presolve/actions/')) { response.writeHead(404, { 'Cache-Control': 'no-store' }); response.end('Not Found\n'); return; }
  if (request.method !== 'GET' && request.method !== 'HEAD') { response.writeHead(405, { Allow: 'GET, HEAD' }); response.end(); return; }
  const requested = decodedSegments(url.pathname);
  if (requested === null) { response.writeHead(400, { 'Cache-Control': 'no-store' }); response.end('Bad Request\n'); return; }
  const selected = routeFor(requested);
  if (!selected) {
    if (!safeSegments(requested) || requested.length === 0) { response.writeHead(404); response.end('Not Found\n'); return; }
    try { const bytes = await readFile(resolve(outputRoot, requested.join('/'))); response.writeHead(200); response.end(request.method === 'HEAD' ? undefined : bytes); return; }
    catch { response.writeHead(404); response.end('Not Found\n'); return; }
  }
  const suffix = requested.slice(selected.parts.length);
  if (!safeSegments(suffix)) { response.writeHead(404); response.end('Not Found\n'); return; }
  if (selected.route.loaderCount > 0 && suffix.length === 0) {
    const controller = new AbortController();
    activeLoaderControllers.add(controller);
    request.once('aborted', () => controller.abort());
    response.once('close', () => { if (!response.writableEnded) controller.abort(); });
    try {
      const routeLoaders = loadersByRoute.get(selected.route.path) ?? [];
      if (routeLoaders.length !== selected.route.loaderCount) throw new Error('PSNODE2014_ROUTE_LOADER_PLAN_MISMATCH');
      const values = await Promise.all(routeLoaders.map((loader) => executeLoader(loader, canonicalParams(loader, requested), request, controller.signal)));
      if (controller.signal.aborted || response.writableEnded) return;
      const path = resolve(outputRoot, `${selected.route.artifactRoot}/index.html`);
      const page = await readFile(path, 'utf8');
      const runtimeMarkers = [...page.matchAll(/    <script src="\.\/runtime(?:\.[a-f0-9]{64})?\.js" defer><\/script>/g)];
      if (runtimeMarkers.length !== 1) throw new Error('PSNODE2015_RESOURCE_BOOTSTRAP_MARKER_MISSING');
      const marker = runtimeMarkers[0][0];
      const markerIndex = runtimeMarkers[0].index;
      const body = page.slice(0, markerIndex) + resourceBootstrapScript(values) + marker + page.slice(markerIndex + marker.length);
      response.writeHead(200, {
        'Content-Type': 'text/html; charset=utf-8',
        ...routeCacheHeaders(routeLoaders),
        'Content-Length': Buffer.byteLength(body)
      });
      response.end(request.method === 'HEAD' ? undefined : body);
    } catch (error) {
      if (controller.signal.aborted || response.writableEnded) return;
      console.error('Presolve route loader failed', { route: selected.route.path, message: error instanceof Error ? error.message : String(error) });
      json(response, 500, { error: { code: 'PSNODE2016_ROUTE_LOADER_EXECUTION_FAILED', message: 'Route loader failed' } });
    } finally { activeLoaderControllers.delete(controller); }
    return;
  }
  const relative = suffix.length === 0 ? `${selected.route.artifactRoot}/index.html` : `${selected.route.artifactRoot}/${suffix.join('/')}`;
  try { const bytes = await readFile(resolve(outputRoot, relative)); response.writeHead(200); response.end(request.method === 'HEAD' ? undefined : bytes); }
  catch {
    if (!safeSegments(requested) || requested.length === 0) { response.writeHead(404); response.end('Not Found\n'); return; }
    try { const bytes = await readFile(resolve(outputRoot, requested.join('/'))); response.writeHead(200); response.end(request.method === 'HEAD' ? undefined : bytes); }
    catch { response.writeHead(404); response.end('Not Found\n'); }
  }
});
for (const signal of ['SIGTERM', 'SIGINT']) {
  process.once(signal, () => {
    for (const controller of activeActionControllers) controller.abort();
    for (const controller of activeLoaderControllers) controller.abort();
    host.close(() => process.exit(0));
    setTimeout(() => process.exit(1), 5000).unref();
  });
}
host.listen(port, '0.0.0.0', () => console.log(`Presolve Node release listening on ${port}`));
"#
    .replace("__PRESOLVE_ROUTES__", &routes)
    .replace("__PRESOLVE_ACTIONS__", &actions)
    .replace("__PRESOLVE_LOADERS__", &loaders)
    .replace("__PRESOLVE_REGISTRY__", &registry)
    .replace("__PRESOLVE_LOADER_REGISTRY__", &loader_registry)
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

fn validate_server_actions(
    actions: &[NodeServerActionV1],
    route_paths: &BTreeSet<String>,
) -> Result<(), NodeDeploymentErrorV1> {
    let ids = actions
        .iter()
        .map(|action| action.id.as_str())
        .collect::<BTreeSet<_>>();
    let coordinates = actions
        .iter()
        .map(|action| action.request_path.as_str())
        .collect::<BTreeSet<_>>();
    if ids.len() != actions.len()
        || coordinates.len() != actions.len()
        || actions.iter().any(|action| {
            action.id.is_empty()
                || !route_paths.contains(&action.route_path)
                || action.component_id.is_empty()
                || action.form.is_empty()
                || !is_server_action_path(&action.request_path)
                || action.package.is_empty()
                || action.version.is_empty()
                || !is_sha256_digest(&action.integrity)
                || action.export.is_empty()
                || !is_artifact_path(&action.runtime_module)
                || action.input != "form_data"
                || !matches!(action.response.as_str(), "json" | "redirect")
                || action.failure != "typed"
                || action.cancellation != "abort"
        })
    {
        return Err(NodeDeploymentErrorV1 {
            code: "PSNODE1019_SERVER_ACTION_HANDOFF_INVALID",
            message: "server-action handoff records must be unique and compiler-canonical".into(),
        });
    }
    Ok(())
}

fn validate_route_loaders(
    loaders: &[NodeRouteLoaderV1],
    route_paths: &BTreeSet<String>,
) -> Result<(), NodeDeploymentErrorV1> {
    let ids = loaders
        .iter()
        .map(|loader| loader.id.as_str())
        .collect::<BTreeSet<_>>();
    let activations = loaders
        .iter()
        .map(|loader| loader.resource_activation_id.as_str())
        .collect::<BTreeSet<_>>();
    let invalid = ids.len() != loaders.len()
        || activations.len() != loaders.len()
        || loaders.iter().any(|loader| {
            let dynamic = loader
                .route_path
                .split('/')
                .filter(|segment| !segment.is_empty())
                .enumerate()
                .filter_map(|(index, segment)| {
                    segment
                        .strip_prefix(':')
                        .map(|name| (name.to_owned(), index))
                })
                .collect::<Vec<_>>();
            loader.id.is_empty()
                || !route_paths.contains(&loader.route_path)
                || loader.component_id.is_empty()
                || loader.field.is_empty()
                || loader.package.is_empty()
                || loader.version.is_empty()
                || !is_sha256_digest(&loader.integrity)
                || loader.export.is_empty()
                || !is_artifact_path(&loader.runtime_module)
                || loader.failure != "typed"
                || loader.bootstrap_key != loader.resource_activation_id
                || !valid_value_codec(&loader.data_codec)
                || !valid_value_codec(&loader.error_codec)
                || loader
                    .parameters
                    .iter()
                    .map(|parameter| (parameter.name.clone(), parameter.segment_index))
                    .collect::<Vec<_>>()
                    != dynamic
                || match loader.cache_scope.as_str() {
                    "no_store" => loader.max_age_seconds.is_some(),
                    "private" | "public" => loader.max_age_seconds.is_none_or(|age| age == 0),
                    _ => true,
                }
        });
    if invalid {
        return Err(NodeDeploymentErrorV1 {
            code: "PSNODE1024_ROUTE_LOADER_INVALID",
            message: "route loaders did not retain exact compiler execution facts".into(),
        });
    }
    Ok(())
}

fn valid_value_codec(codec: &ResumeValueCodec) -> bool {
    match codec {
        ResumeValueCodec::NullCodec
        | ResumeValueCodec::BooleanCodec
        | ResumeValueCodec::NumberCodec
        | ResumeValueCodec::StringCodec => true,
        ResumeValueCodec::ArrayCodec(value) | ResumeValueCodec::NullableCodec(value) => {
            valid_value_codec(value)
        }
        ResumeValueCodec::ObjectCodec(properties) => {
            let mut names = BTreeSet::new();
            properties.iter().all(|property| {
                !property.name.is_empty()
                    && names.insert(property.name.as_str())
                    && valid_value_codec(&property.codec)
            })
        }
    }
}

fn release_id(plan: &NodeDeploymentPlanV1) -> String {
    let input = plan
        .routes
        .iter()
        .map(|route| {
            format!(
                "route:{}:{}:{}:{}:{}\n",
                route.path,
                route.artifact_root,
                route.execution,
                route.loader_count,
                route.server_action_count
            )
        })
        .chain(
            plan.artifacts
                .iter()
                .map(|artifact| format!("artifact:{}:{}\n", artifact.path, artifact.digest)),
        )
        .chain(plan.server_actions.iter().map(|action| {
            format!(
                "action:{}:{}:{}:{}:{}:{}:{}\n",
                action.id,
                action.route_path,
                action.request_path,
                action.package,
                action.version,
                action.integrity,
                action.export
            )
        }))
        .chain(plan.route_loaders.iter().map(|loader| {
            format!(
                "loader:{}:{}:{}:{}:{}:{}:{}:{:?}\n",
                loader.id,
                loader.route_path,
                loader.package,
                loader.version,
                loader.integrity,
                loader.export,
                loader.cache_scope,
                loader.max_age_seconds
            )
        }))
        .chain(
            plan.server_action_registry
                .iter()
                .map(|artifact| format!("server-registry:{}:{}\n", artifact.path, artifact.digest)),
        )
        .chain(
            plan.route_loader_registry
                .iter()
                .map(|artifact| format!("loader-registry:{}:{}\n", artifact.path, artifact.digest)),
        )
        .collect::<String>();
    format!("sha256:{:x}", Sha256::digest(input))
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

fn is_sha256_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

fn is_server_action_path(value: &str) -> bool {
    let Some(digest) = value.strip_prefix("/_presolve/actions/") else {
        return false;
    };
    digest.len() == 64
        && digest
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use presolve_compiler::{
        ApplicationPublicationArtifactV1, FileRoutePublicationManifestV1,
        FileRoutePublicationRouteV1,
    };

    use super::{
        attach_node_route_loader_registry_v1, attach_node_server_action_registry_v1,
        build_node_deployment_plan_v1, node_static_host_module_v1, NodeDeploymentOptionsV1,
    };

    fn loader_plan(loader: &str) -> String {
        format!(
            r#"{{"schemaVersion":2,"routes":[{{"path":"/","loaders":[]}},{{"path":"/posts/:slug","loaders":[{loader}]}}]}}"#
        )
    }

    fn empty_loader_plan() -> &'static str {
        r#"{"schemaVersion":2,"routes":[{"path":"/","loaders":[]},{"path":"/posts/:slug","loaders":[]}]}"#
    }

    fn canonical_loader() -> &'static str {
        r#"{"id":"route-loader-capability:post","component_id":"component:post","component":"Post","field":"post","endpoint_semantic":"route_parameters","resource_type":"Resource<Post, NotFound>","data_type":{"kind":"string"},"error_type":{"kind":"string"},"data_codec":{"kind":"string_codec"},"error_codec":{"kind":"string_codec"},"package":"post-service","version":"1.0.0","integrity":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","export":"loadPost","runtime_module":"dist/load.js","type_signature":"(RouteParameters, AbortSignal) -> Promise<RouteLoaderResult>","cache_scope":"public","max_age_seconds":60,"failure":"typed","resume_policy":"reload","resource_declaration_id":"resource-declaration:post","resource_activation_id":"resource-activation:post","component_instance":"component-instance:post","resource_state_slot":"resource-state:post","resource_data_slot":"resource-data:post","resource_error_slot":"resource-error:post","parameters":[{"name":"slug","segment_index":1}],"normalization":{"percent_decoding":"strict_utf8","duplicate_parameters":"reject"},"cache_key":{"algorithm":"sha256","ingredients":["loader_id","parameter:slug"],"private_partition":false}}"#
    }

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
            &loader_plan(canonical_loader()),
            r#"{"schemaVersion":2,"routes":[{"path":"/","actions":[]},{"path":"/posts/:slug","actions":[]}]}"#,
            &NodeDeploymentOptionsV1 {
                application_name: "presolve-docs".into(),
            },
        )
        .unwrap();
        assert_eq!(plan.routes[0].execution, "static");
        assert_eq!(plan.routes[1].execution, "node");
        assert_eq!(plan.route_loaders.len(), 1);
        assert_eq!(plan.route_loaders[0].parameters[0].name, "slug");
        let host = node_static_host_module_v1(&plan);
        assert!(host.contains("executeLoader"));
        assert!(host.contains("presolve-resource-bootstrap"));
        assert!(host.contains("routes/segment-posts/parameter-slug"));
    }

    #[test]
    fn rejects_handoff_route_sets_that_do_not_equal_the_manifest() {
        let error = build_node_deployment_plan_v1(
            &manifest(),
            r#"{"schemaVersion":2,"routes":[{"path":"/","loaders":[]}]}"#,
            r#"{"schemaVersion":2,"routes":[{"path":"/","actions":[]},{"path":"/posts/:slug","actions":[]}]}"#,
            &NodeDeploymentOptionsV1 {
                application_name: "presolve-docs".into(),
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "PSNODE1007_HANDOFF_ROUTE_SET_MISMATCH");
    }

    #[test]
    fn retains_only_canonical_form_actions_and_binds_a_digest_registry() {
        let action = r#"{"id":"server-action-capability:contact","source":"canonical_form","component_id":"component:home","method":"__submit_contact","form":"contact","request_path":"/_presolve/actions/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","package":"contact-service","version":"1.0.0","integrity":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","export":"saveContact","runtime_module":"dist/save.js","type_signature":"(FormData, AbortSignal) -> Promise<ServerActionResult>","resume_policy":"cold_fallback","input":"form_data","response":"json","failure":"typed","cancellation":"abort"}"#;
        let actions = format!(
            r#"{{"schemaVersion":2,"routes":[{{"path":"/","actions":[{action}]}},{{"path":"/posts/:slug","actions":[]}}]}}"#
        );
        let mut plan = build_node_deployment_plan_v1(
            &manifest(),
            empty_loader_plan(),
            &actions,
            &NodeDeploymentOptionsV1 {
                application_name: "presolve-docs".into(),
            },
        )
        .unwrap();
        assert_eq!(plan.server_actions.len(), 1);
        assert_eq!(plan.server_actions[0].form, "contact");
        let before = plan.release_id.clone();
        attach_node_server_action_registry_v1(
            &mut plan,
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".into(),
        )
        .unwrap();
        assert_ne!(plan.release_id, before);
        assert_eq!(
            plan.server_action_registry.as_ref().unwrap().path,
            "presolve.server-actions.mjs"
        );
        let host = node_static_host_module_v1(&plan);
        assert!(host.contains("dispatchAction"));
        assert!(host.contains("PSNODE2004_ACTION_ORIGIN_REJECTED"));
        assert!(host.contains("presolve.server-actions.mjs"));
    }

    #[test]
    fn binds_a_digest_registry_for_canonical_route_loaders() {
        let mut plan = build_node_deployment_plan_v1(
            &manifest(),
            &loader_plan(canonical_loader()),
            r#"{"schemaVersion":2,"routes":[{"path":"/","actions":[]},{"path":"/posts/:slug","actions":[]}]}"#,
            &NodeDeploymentOptionsV1 {
                application_name: "presolve-docs".into(),
            },
        )
        .unwrap();
        let before = plan.release_id.clone();
        attach_node_route_loader_registry_v1(
            &mut plan,
            "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".into(),
        )
        .unwrap();
        assert_ne!(plan.release_id, before);
        assert_eq!(
            plan.route_loader_registry.as_ref().unwrap().path,
            "presolve.route-loaders.mjs"
        );
        let host = node_static_host_module_v1(&plan);
        assert!(host.contains("loadLoaderRegistry"));
        assert!(host.contains("presolve.route-loaders.mjs"));
        assert!(host.contains("PSNODE2013_ROUTE_LOADER_RESULT_MISMATCH"));
    }
}
