import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { join } from "node:path";

/** V1 shape of the compiler manifest consumed by this external adapter. */
export const PRESOLVE_VITE_ADAPTER_SCHEMA_VERSION = 1;
export const PRESOLVE_APPLICATION_PUBLICATION_CONTRACT_V1 = "presolve-application-publication:1";
export const PRESOLVE_VIRTUAL_MODULE_SCHEMA_VERSION = 1;
export const PRESOLVE_VIRTUAL_MODULE_PREFIX = "virtual:presolve/v1/";
export const PRESOLVE_DEVELOPMENT_DIAGNOSTICS_SCHEMA_VERSION = 1;
export const PRESOLVE_VITE_PRODUCTION_SCHEMA_VERSION = 1;

/**
 * Creates the empty Vite boundary over an already-produced compiler product.
 *
 * The compiler owns parsing, TypeScript semantics, lowering, and artifact
 * contents. This package owns only Vite integration and its transport hooks.
 */
export function createPresolveVitePlugin({ compilerProduct, readArtifact, requestHost } = {}) {
  const manifest = validateCompilerProduct(compilerProduct);
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
    plugin.load = registry.load;
  }
  if (requestHost) {
    if (typeof requestHost !== "function") {
      throw new TypeError("@presolve/vite requestHost must be a function");
    }
    plugin.configureServer = server => {
      server.middlewares.use((request, response, next) => {
        Promise.resolve(requestHost(request)).then(hostResponse => {
          if (hostResponse === undefined) {
            next();
            return;
          }
          writeHostResponse(response, hostResponse);
        }).catch(next);
      });
    };
  }
  return Object.freeze(plugin);
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
  const plugin = createPresolveVitePlugin({ compilerProduct, readArtifact, requestHost });
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
  const outDir = vite.build?.outDir;
  if (typeof outDir !== "string" || !outDir) {
    throw new TypeError("Presolve production build requires an explicit Vite build.outDir");
  }
  const manifestName = "presolve-vite-manifest.json";
  const virtualEntryId = `${PRESOLVE_VIRTUAL_MODULE_PREFIX}${entryArtifactPath}`;
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
      rollupOptions: {
        ...vite.build?.rollupOptions,
        input: virtualEntryId,
      },
    },
  });
  const viteManifestPath = join(outDir, manifestName);
  const viteManifest = JSON.parse(await readFile(viteManifestPath, "utf8"));
  const entries = Object.entries(viteManifest)
    .filter(([, output]) => output.isEntry)
    .map(([input, output]) => Object.freeze({
      input,
      file: output.file,
      css: Object.freeze([...(output.css ?? [])].sort()),
      assets: Object.freeze([...(output.assets ?? [])].sort()),
      imports: Object.freeze([...(output.imports ?? [])].sort()),
      compilerArtifactPath: input === virtualEntryId ? entryArtifactPath : undefined,
      componentId: input === virtualEntryId ? manifest.entry_component_id : undefined,
    }))
    .sort((left, right) => left.input.localeCompare(right.input));
  if (!entries.some(entry => entry.compilerArtifactPath === entryArtifactPath)) {
    throw new Error("Vite production manifest did not retain the compiler-selected virtual entry");
  }
  return Object.freeze({
    schemaVersion: PRESOLVE_VITE_PRODUCTION_SCHEMA_VERSION,
    compilerContract: manifest.compiler_contract,
    workspaceSnapshotId: manifest.workspace_snapshot_id,
    entryComponentId: manifest.entry_component_id,
    viteManifestPath,
    entries: Object.freeze(entries),
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
  return [
    `export const artifactPath = ${JSON.stringify(artifactPath)};`,
    `export const content = ${JSON.stringify(content)};`,
    "export default content;",
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
