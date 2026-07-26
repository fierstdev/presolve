import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { join, resolve } from "node:path";

/** V1 shape of the compiler manifest consumed by this external adapter. */
export const PRESOLVE_VITE_ADAPTER_SCHEMA_VERSION = 1;
export const PRESOLVE_APPLICATION_PUBLICATION_CONTRACT_V1 = "presolve-application-publication:1";
export const PRESOLVE_VIRTUAL_MODULE_SCHEMA_VERSION = 1;
export const PRESOLVE_VIRTUAL_MODULE_PREFIX = "virtual:presolve/v1/";
export const PRESOLVE_ENVIRONMENT_PUBLICATION_ARTIFACT = "environment.browser.json";
export const PRESOLVE_ENVIRONMENT_PUBLICATION_SCHEMA_VERSION = 1;
export const PRESOLVE_DEVELOPMENT_DIAGNOSTICS_SCHEMA_VERSION = 1;
export const PRESOLVE_VITE_PRODUCTION_SCHEMA_VERSION = 1;
export const PRESOLVE_HMR_UPDATE_SCHEMA_VERSION = 1;
export const PRESOLVE_HMR_EVENT = "presolve:hmr";
export const PRESOLVE_PRODUCTION_AUDIT_SCHEMA_VERSION = 1;
export const PRESOLVE_PRODUCTION_AUDIT_ARTIFACT = "production-audit.json";
export const PRESOLVE_SOURCE_MAP_TRANSLATION_SCHEMA_VERSION = 1;

const PRESOLVE_HMR_MESSAGE_CLASSES = new Set([
  "template-update",
  "action-update",
  "computed-update",
  "style-update",
  "server-only-update",
  "component-instance-reload",
  "route-reload",
  "full-reload",
]);

/**
 * Creates the Vite transport boundary over an already-produced compiler product.
 *
 * The compiler owns parsing, TypeScript semantics, lowering, and artifact
 * contents. This package owns only Vite integration and its transport hooks.
 */
export function createPresolveVitePlugin({ compilerProduct, readArtifact, requestHost, hmr } = {}) {
  const manifest = validateCompilerProduct(compilerProduct);
  let hmrTransport;
  const plugin = {
    name: "presolve:compiler-products",
    enforce: "pre",
    api: Object.freeze({
      schemaVersion: PRESOLVE_VITE_ADAPTER_SCHEMA_VERSION,
      compilerContract: manifest.compiler_contract,
      workspaceSnapshotId: manifest.workspace_snapshot_id,
    }),
  };
  if (readArtifact) {
    const registry = createPresolveVirtualModuleRegistry({ compilerProduct, readArtifact });
    plugin.resolveId = registry.resolveId;
    plugin.load = async resolvedId => {
      const code = await registry.load(resolvedId);
      if (code === undefined) return undefined;
      return {
        code,
        // This is a transport map for the virtual compiler artifact, not an
        // authored-source map. Vite may retain it in physical output maps.
        map: {
          version: 3,
          sources: [resolvedId.replace(/^\0/, "")],
          sourcesContent: [code],
          names: [],
          mappings: "AAAA",
        },
      };
    };
  }
  if (requestHost !== undefined && typeof requestHost !== "function") {
    throw new TypeError("@presolve/vite requestHost must be a function");
  }
  if (hmr !== undefined && typeof hmr !== "function") {
    throw new TypeError("@presolve/vite hmr must be a compiler-owned function");
  }
  if (hmr) {
    plugin.handleHotUpdate = async context => {
      if (!hmrTransport) {
        throw new Error("Presolve HMR transport is unavailable before Vite configures the server");
      }
      return hmrTransport.publish(
        await hmr(Object.freeze({ file: context.file, timestamp: context.timestamp })),
        context.modules,
      );
    };
  }
  if (requestHost || hmr) {
    plugin.configureServer = server => {
      if (requestHost) {
        server.middlewares.use((request, response, next) => {
          Promise.resolve(requestHost(request)).then(hostResponse => {
            if (hostResponse === undefined) {
              next();
              return;
            }
            writeHostResponse(response, hostResponse);
          }).catch(next);
        });
      }
      if (hmr) {
        hmrTransport = createPresolveHmrTransport({
          workspaceSnapshotId: manifest.workspace_snapshot_id,
          send: message => server.ws.send(message),
        });
      }
    };
  }
  return Object.freeze(plugin);
}

/**
 * Validates and transports one compiler-selected HMR update without deriving
 * semantics from Vite modules, source text, or filenames.
 */
export function createPresolveHmrTransport({ workspaceSnapshotId, send } = {}) {
  if (typeof workspaceSnapshotId !== "string" || !workspaceSnapshotId) {
    throw new TypeError("Presolve HMR transport requires a workspaceSnapshotId");
  }
  if (typeof send !== "function") {
    throw new TypeError("Presolve HMR transport requires a Vite send function");
  }
  return Object.freeze({
    schemaVersion: PRESOLVE_HMR_UPDATE_SCHEMA_VERSION,
    publish(update, viteModules = []) {
      const canonical = validateHmrUpdate(update, workspaceSnapshotId);
      if (!Array.isArray(viteModules)) {
        throw new TypeError("Vite HMR modules must be an array");
      }
      if (canonical.messageClass === "style-update") return viteModules;
      if (canonical.messageClass === "full-reload") {
        send({ type: "full-reload", path: "*" });
        return [];
      }
      send({ type: "custom", event: PRESOLVE_HMR_EVENT, data: canonical });
      return [];
    },
  });
}

/**
 * Starts the Vite development server owned by the `presolve dev` boundary.
 *
 * The request host decides document, route, loader, and action routing from
 * compiler products; returning `undefined` delegates JS, CSS, and assets to
 * Vite middleware. Vite types are intentionally not exposed from this API.
 */
export async function startPresolveDevServer({
  compilerProduct,
  readArtifact,
  requestHost,
  hmr,
  diagnostics = () => ({}),
  vite = {},
} = {}) {
  if (typeof requestHost !== "function") {
    throw new TypeError("presolve dev requires a compiler-owned requestHost");
  }
  if (typeof diagnostics !== "function") {
    throw new TypeError("presolve dev diagnostics must be a function");
  }
  const { createServer } = await import("vite");
  const plugin = createPresolveVitePlugin({ compilerProduct, readArtifact, requestHost, hmr });
  const configuredPlugins = vite.plugins === undefined
    ? []
    : Array.isArray(vite.plugins) ? vite.plugins : [vite.plugins];
  const server = await createServer({
    ...vite,
    appType: "custom",
    plugins: [...configuredPlugins, plugin],
  });
  let currentDiagnostics = composeDevelopmentDiagnostics(await diagnostics());
  const publishDiagnostics = async () => {
    currentDiagnostics = composeDevelopmentDiagnostics(await diagnostics());
    server.ws.send({
      type: "custom",
      event: "presolve:diagnostics",
      data: currentDiagnostics,
    });
    return currentDiagnostics;
  };
  await server.listen();
  await publishDiagnostics();
  return Object.freeze({
    server,
    diagnostics: () => currentDiagnostics,
    publishDiagnostics,
    close: () => server.close(),
  });
}

/** Merges TypeScript and Presolve diagnostics without changing their meaning. */
export function composeDevelopmentDiagnostics({ typescript = [], presolve = [] } = {}) {
  const diagnostics = [
    ...normalizeDiagnostics("typescript", typescript),
    ...normalizeDiagnostics("presolve", presolve),
  ].sort(compareDevelopmentDiagnostics);
  return Object.freeze({
    schemaVersion: PRESOLVE_DEVELOPMENT_DIAGNOSTICS_SCHEMA_VERSION,
    diagnostics: Object.freeze(diagnostics),
  });
}

/**
 * Runs Vite's production build for one compiler-selected logical entry.
 *
 * The Vite manifest is read after physical output is written and then mapped
 * back to the stable compiler entry-component ID. No Vite filename becomes a
 * compiler semantic identity.
 */
export async function buildPresolveProduction({
  compilerProduct,
  readArtifact,
  entryArtifactPath,
  viteEntries = [],
  vite = {},
} = {}) {
  const manifest = validateCompilerProduct(compilerProduct);
  if (typeof entryArtifactPath !== "string" || !entryArtifactPath) {
    throw new TypeError("Presolve production build requires an entryArtifactPath");
  }
  if (!manifest.artifacts.some(artifact => artifact.path === entryArtifactPath)) {
    throw new TypeError(`Presolve production entry is not a compiler artifact: ${entryArtifactPath}`);
  }
  if (typeof manifest.entry_component_id !== "string" || !manifest.entry_component_id) {
    throw new TypeError("Presolve production build requires manifest.entry_component_id");
  }
  if (!Array.isArray(viteEntries)) {
    throw new TypeError("Presolve Vite entries must be an array");
  }
  const outDir = vite.build?.outDir;
  if (typeof outDir !== "string" || !outDir) {
    throw new TypeError("Presolve production build requires an explicit Vite build.outDir");
  }
  const manifestName = "presolve-vite-manifest.json";
  const virtualEntryId = `${PRESOLVE_VIRTUAL_MODULE_PREFIX}${entryArtifactPath}`;
  const viteRoot = typeof vite.root === "string" && vite.root ? vite.root : process.cwd();
  const additionalInputs = viteEntries
    .map(validateViteEntry)
    .map(entry => Object.freeze({ ...entry, path: resolve(viteRoot, entry.path) }));
  if (new Set(additionalInputs.map(entry => entry.name)).size !== additionalInputs.length) {
    throw new TypeError("Presolve Vite entry names must be unique");
  }
  const input = Object.fromEntries([
    ["presolve-compiler-entry", virtualEntryId],
    ...additionalInputs.map(entry => [entry.name, entry.path]),
  ]);
  const plugin = createPresolveVitePlugin({ compilerProduct, readArtifact });
  const configuredPlugins = vite.plugins === undefined
    ? []
    : Array.isArray(vite.plugins) ? vite.plugins : [vite.plugins];
  const { build } = await import("vite");
  await build({
    ...vite,
    configFile: false,
    plugins: [...configuredPlugins, plugin],
    build: {
      ...vite.build,
      outDir,
      emptyOutDir: false,
      manifest: manifestName,
      sourcemap: true,
      rollupOptions: {
        ...vite.build?.rollupOptions,
        input,
      },
    },
  });
  const viteManifestPath = join(outDir, manifestName);
  const viteManifest = JSON.parse(await readFile(viteManifestPath, "utf8"));
  const isCompilerEntry = input => input === virtualEntryId
    || input === "presolve-compiler-entry"
    || input.endsWith(`${PRESOLVE_VIRTUAL_MODULE_PREFIX}${entryArtifactPath}`);
  const entries = Object.entries(viteManifest)
    .filter(([, output]) => output.isEntry)
    .map(([input, output]) => Object.freeze({
      input,
      file: output.file,
      css: Object.freeze([...(output.css ?? [])].sort()),
      assets: Object.freeze([...(output.assets ?? [])].sort()),
      imports: Object.freeze([...(output.imports ?? [])].sort()),
      compilerArtifactPath: isCompilerEntry(input)
        ? entryArtifactPath
        : undefined,
      componentId: isCompilerEntry(input)
        ? manifest.entry_component_id
        : undefined,
    }))
    .sort((left, right) => left.input.localeCompare(right.input));
  if (!entries.some(entry => entry.compilerArtifactPath === entryArtifactPath)) {
    throw new Error("Vite production manifest did not retain the compiler-selected virtual entry");
  }
  const sourceMaps = await Promise.all(entries
    .filter(entry => entry.compilerArtifactPath !== undefined)
    .map(async entry => {
    const mapPath = join(outDir, `${entry.file}.map`);
    let map;
    try {
      map = JSON.parse(await readFile(mapPath, "utf8"));
    } catch {
      throw new Error(`Vite did not emit a source map for ${entry.file}`);
    }
    return Object.freeze({
      file: entry.file,
      mapPath,
      translation: translatePresolveSourceMap({ compilerProduct, sourceMap: map }),
    });
    }));
  return Object.freeze({
    schemaVersion: PRESOLVE_VITE_PRODUCTION_SCHEMA_VERSION,
    compilerContract: manifest.compiler_contract,
    workspaceSnapshotId: manifest.workspace_snapshot_id,
    entryComponentId: manifest.entry_component_id,
    viteManifestPath,
    entries: Object.freeze(entries),
    sourceMaps: Object.freeze(sourceMaps),
  });
}

function validateViteEntry(entry) {
  if (!entry || typeof entry !== "object" || Array.isArray(entry)
    || typeof entry.name !== "string" || !entry.name
    || typeof entry.path !== "string" || !entry.path
    || entry.name === "presolve-compiler-entry") {
    throw new TypeError("Presolve Vite entries require a non-empty name and path");
  }
  return Object.freeze({ name: entry.name, path: entry.path });
}

/**
 * Associates Vite map sources with exact compiler publication artifacts. This
 * does not decode mappings or fabricate authored locations: Vite owns map
 * generation and the compiler manifest owns the logical source identity.
 */
export function translatePresolveSourceMap({ compilerProduct, sourceMap } = {}) {
  const manifest = validateCompilerProduct(compilerProduct);
  if (!sourceMap || typeof sourceMap !== "object" || !Array.isArray(sourceMap.sources)) {
    throw new TypeError("Vite source map requires a sources array");
  }
  const artifactPaths = new Set(manifest.artifacts.map(artifact => {
    validateArtifact(artifact);
    return artifact.path;
  }));
  const sources = sourceMap.sources.map(source => {
    if (typeof source !== "string" || !source) {
      throw new TypeError("Vite source map sources must be non-empty strings");
    }
    const index = source.indexOf(PRESOLVE_VIRTUAL_MODULE_PREFIX);
    const artifactPath = index < 0 ? undefined : source.slice(index + PRESOLVE_VIRTUAL_MODULE_PREFIX.length);
    return Object.freeze({
      viteSource: source,
      compilerArtifactPath: artifactPaths.has(artifactPath) ? artifactPath : undefined,
    });
  });
  return Object.freeze({
    schemaVersion: PRESOLVE_SOURCE_MAP_TRANSLATION_SCHEMA_VERSION,
    workspaceSnapshotId: manifest.workspace_snapshot_id,
    sources: Object.freeze(sources),
  });
}

/**
 * Reads the compiler-produced production audit after verifying its publication
 * digest. The adapter validates transport shape only; audit policy remains in
 * the compiler artifact.
 */
export async function readPresolveProductionAudit({ compilerProduct, readArtifact } = {}) {
  const manifest = validateCompilerProduct(compilerProduct);
  if (typeof readArtifact !== "function") {
    throw new TypeError("Presolve production audit requires a readArtifact function");
  }
  const artifact = manifest.artifacts.find(candidate => candidate.path === PRESOLVE_PRODUCTION_AUDIT_ARTIFACT);
  if (!artifact) {
    throw new TypeError("compiler product does not publish a production audit artifact");
  }
  validateArtifact(artifact);
  const bytes = toBytes(await readArtifact(artifact.path));
  const digest = createHash("sha256").update(bytes).digest("hex");
  if (digest !== artifact.digest) {
    throw new Error("compiler production audit digest mismatch");
  }
  let audit;
  try {
    audit = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
  } catch {
    throw new TypeError("compiler production audit must be UTF-8 JSON");
  }
  if (!audit || typeof audit !== "object" || Array.isArray(audit)
    || audit.schemaVersion !== PRESOLVE_PRODUCTION_AUDIT_SCHEMA_VERSION
    || audit.status !== "passed"
    || typeof audit.buildId !== "string" || !audit.buildId
    || !Array.isArray(audit.checks) || audit.checks.some(check => typeof check !== "string" || !check)) {
    throw new TypeError("compiler production audit is not a passing schema-v1 product");
  }
  return Object.freeze({
    schemaVersion: audit.schemaVersion,
    buildId: audit.buildId,
    optimizationReportSchemaVersion: audit.optimizationReportSchemaVersion,
    runtimeCostReportSchemaVersion: audit.runtimeCostReportSchemaVersion,
    runtimeTableCount: audit.runtimeTableCount,
    authorityCount: audit.authorityCount,
    invariantCount: audit.invariantCount,
    checks: Object.freeze([...audit.checks]),
    status: audit.status,
  });
}

/**
 * Builds versioned Vite module names from one digest-bound compiler product.
 *
 * `readArtifact` is a host bridge over exact compiler bytes. Content is
 * re-digested before Vite sees it, so the manifest remains the source of truth.
 */
export function createPresolveVirtualModuleRegistry({ compilerProduct, readArtifact }) {
  const manifest = validateCompilerProduct(compilerProduct);
  if (typeof readArtifact !== "function") {
    throw new TypeError("@presolve/vite virtual modules require a readArtifact function");
  }
  const modules = new Map();
  for (const artifact of manifest.artifacts) {
    validateArtifact(artifact);
    const id = `${PRESOLVE_VIRTUAL_MODULE_PREFIX}${artifact.path}`;
    if (modules.has(id)) {
      throw new TypeError(`duplicate Presolve publication artifact ${artifact.path}`);
    }
    modules.set(id, Object.freeze({
      id,
      resolvedId: `\0${id}`,
      artifactPath: artifact.path,
      digest: artifact.digest,
    }));
  }
  const entries = Object.freeze([...modules.values()].sort((left, right) => left.id.localeCompare(right.id)));
  return Object.freeze({
    schemaVersion: PRESOLVE_VIRTUAL_MODULE_SCHEMA_VERSION,
    entries,
    resolveId(id) {
      return modules.get(id)?.resolvedId;
    },
    async load(resolvedId) {
      const entry = entries.find(candidate => candidate.resolvedId === resolvedId);
      if (!entry) return undefined;
      const bytes = toBytes(await readArtifact(entry.artifactPath));
      const digest = createHash("sha256").update(bytes).digest("hex");
      if (digest !== entry.digest) {
        throw new Error(`compiler artifact digest mismatch for ${entry.artifactPath}`);
      }
      return virtualModuleSource(entry.artifactPath, new TextDecoder("utf-8", { fatal: true }).decode(bytes));
    },
  });
}

function validateCompilerProduct(product) {
  if (!product || typeof product !== "object") {
    throw new TypeError("@presolve/vite requires a compiler product object");
  }
  const { manifest } = product;
  if (!manifest || typeof manifest !== "object") {
    throw new TypeError("@presolve/vite requires compilerProduct.manifest");
  }
  if (manifest.schema_version !== 1) {
    throw new TypeError(`unsupported Presolve publication manifest schema ${manifest.schema_version}`);
  }
  if (manifest.compiler_contract !== PRESOLVE_APPLICATION_PUBLICATION_CONTRACT_V1) {
    throw new TypeError(`unsupported Presolve compiler contract ${manifest.compiler_contract}`);
  }
  if (!Array.isArray(manifest.artifacts)) {
    throw new TypeError("Presolve publication manifest must contain artifacts");
  }
  return manifest;
}

function validateHmrUpdate(update, workspaceSnapshotId) {
  if (!update || typeof update !== "object" || Array.isArray(update)) {
    throw new TypeError("Presolve HMR update must be an object");
  }
  if (update.schemaVersion !== PRESOLVE_HMR_UPDATE_SCHEMA_VERSION) {
    throw new TypeError(`unsupported Presolve HMR update schema ${update.schemaVersion}`);
  }
  if (update.workspaceSnapshotId !== workspaceSnapshotId) {
    throw new TypeError("Presolve HMR update workspace snapshot does not match the compiler product");
  }
  if (typeof update.updateId !== "string" || !update.updateId) {
    throw new TypeError("Presolve HMR update requires a stable updateId");
  }
  if (!PRESOLVE_HMR_MESSAGE_CLASSES.has(update.messageClass)) {
    throw new TypeError("Presolve HMR update has an unsupported messageClass");
  }
  if (!Array.isArray(update.affectedModuleIds)
    || update.affectedModuleIds.some(id => typeof id !== "string" || !id)
    || [...new Set(update.affectedModuleIds)].length !== update.affectedModuleIds.length
    || [...update.affectedModuleIds].sort().some((id, index) => id !== update.affectedModuleIds[index])) {
    throw new TypeError("Presolve HMR update requires sorted unique affectedModuleIds");
  }
  if (!matchesStateCompatibility(update)) {
    throw new TypeError("Presolve HMR state preservation must be explicitly compiler-proven");
  }
  return Object.freeze({
    schemaVersion: update.schemaVersion,
    workspaceSnapshotId: update.workspaceSnapshotId,
    updateId: update.updateId,
    messageClass: update.messageClass,
    affectedModuleIds: Object.freeze([...update.affectedModuleIds]),
    stateCompatibility: update.stateCompatibility,
    preserveState: update.preserveState,
  });
}

function matchesStateCompatibility(update) {
  return (update.stateCompatibility === "proven-compatible" && update.preserveState === true)
    || (update.stateCompatibility === "reload-required" && update.preserveState === false);
}

function validateArtifact(artifact) {
  if (!artifact || typeof artifact.path !== "string" || !artifact.path || artifact.path.startsWith("/")
    || artifact.path.split("/").includes("..")) {
    throw new TypeError("Presolve publication artifact paths must be non-escaping relative paths");
  }
  if (!/^[a-f0-9]{64}$/.test(artifact.digest)) {
    throw new TypeError(`Presolve publication artifact ${artifact.path} requires a SHA-256 digest`);
  }
}

function toBytes(value) {
  if (value instanceof Uint8Array) return value;
  if (typeof value === "string") return new TextEncoder().encode(value);
  throw new TypeError("readArtifact must return a string or Uint8Array");
}

function virtualModuleSource(artifactPath, content) {
  if (artifactPath === PRESOLVE_ENVIRONMENT_PUBLICATION_ARTIFACT) {
    return environmentVirtualModuleSource(artifactPath, content);
  }
  return [
    `export const artifactPath = ${JSON.stringify(artifactPath)};`,
    `export const content = ${JSON.stringify(content)};`,
    "export default content;",
    "",
  ].join("\n");
}

/**
 * Projects the compiler-published browser environment artifact into one Vite
 * virtual module. This validates only the immutable compiler product: it never
 * reads dotenv files, process state, or Vite's environment object.
 */
function environmentVirtualModuleSource(artifactPath, content) {
  let artifact;
  try {
    artifact = JSON.parse(content);
  } catch {
    throw new TypeError("Presolve browser environment artifact must be UTF-8 JSON");
  }
  if (!artifact || typeof artifact !== "object" || Array.isArray(artifact)
    || artifact.schemaVersion !== PRESOLVE_ENVIRONMENT_PUBLICATION_SCHEMA_VERSION
    || !artifact.browserValues || typeof artifact.browserValues !== "object"
    || Array.isArray(artifact.browserValues)) {
    throw new TypeError("Presolve browser environment artifact must be a schema-v1 compiler product");
  }
  for (const [name, value] of Object.entries(artifact.browserValues)) {
    if (!name.startsWith("PRESOLVE_PUBLIC_") || name.length === "PRESOLVE_PUBLIC_".length
      || typeof value !== "string" || value.includes("\0")) {
      throw new TypeError("Presolve browser environment artifact contained an invalid public value");
    }
  }
  const browserValues = Object.fromEntries(Object.entries(artifact.browserValues).sort(
    ([left], [right]) => left.localeCompare(right),
  ));
  return [
    `export const artifactPath = ${JSON.stringify(artifactPath)};`,
    `export const schemaVersion = ${PRESOLVE_ENVIRONMENT_PUBLICATION_SCHEMA_VERSION};`,
    `export const browserValues = Object.freeze(${JSON.stringify(browserValues)});`,
    "export default browserValues;",
    "",
  ].join("\n");
}

function writeHostResponse(response, hostResponse) {
  if (!hostResponse || typeof hostResponse !== "object") {
    throw new TypeError("Presolve requestHost must return undefined or a response object");
  }
  const { status = 200, headers = {}, body = "" } = hostResponse;
  if (!Number.isInteger(status) || status < 100 || status > 599) {
    throw new TypeError("Presolve requestHost response status must be an HTTP status code");
  }
  if (!headers || typeof headers !== "object" || Array.isArray(headers)) {
    throw new TypeError("Presolve requestHost response headers must be an object");
  }
  response.statusCode = status;
  for (const [name, value] of Object.entries(headers)) response.setHeader(name, value);
  response.end(toBytes(body));
}

function normalizeDiagnostics(authority, diagnostics) {
  if (!Array.isArray(diagnostics)) {
    throw new TypeError(`${authority} diagnostics must be an array`);
  }
  return diagnostics.map(diagnostic => {
    if (!diagnostic || typeof diagnostic !== "object" || diagnostic.code === undefined
      || typeof diagnostic.message !== "string") {
      throw new TypeError(`${authority} diagnostics require code and message`);
    }
    return Object.freeze({ authority, ...diagnostic });
  });
}

function compareDevelopmentDiagnostics(left, right) {
  return String(left.file ?? "").localeCompare(String(right.file ?? ""))
    || (left.start ?? -1) - (right.start ?? -1)
    || String(left.code).localeCompare(String(right.code))
    || left.authority.localeCompare(right.authority);
}
