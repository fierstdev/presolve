import { createHash } from "node:crypto";

/** V1 shape of the compiler manifest consumed by this external adapter. */
export const PRESOLVE_VITE_ADAPTER_SCHEMA_VERSION = 1;
export const PRESOLVE_APPLICATION_PUBLICATION_CONTRACT_V1 = "presolve-application-publication:1";
export const PRESOLVE_VIRTUAL_MODULE_SCHEMA_VERSION = 1;
export const PRESOLVE_VIRTUAL_MODULE_PREFIX = "virtual:presolve/v1/";

/**
 * Creates the empty Vite boundary over an already-produced compiler product.
 *
 * The compiler owns parsing, TypeScript semantics, lowering, and artifact
 * contents. This package owns only Vite integration, beginning with this
 * contract check. Virtual modules and dev-server hooks are intentionally
 * deferred to their own versioned products.
 */
export function createPresolveVitePlugin({ compilerProduct, readArtifact } = {}) {
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
  return Object.freeze(plugin);
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
