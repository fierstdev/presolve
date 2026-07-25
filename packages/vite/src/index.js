/** V1 shape of the compiler manifest consumed by this external adapter. */
export const PRESOLVE_VITE_ADAPTER_SCHEMA_VERSION = 1;
export const PRESOLVE_APPLICATION_PUBLICATION_CONTRACT_V1 = "presolve-application-publication:1";

/**
 * Creates the empty Vite boundary over an already-produced compiler product.
 *
 * The compiler owns parsing, TypeScript semantics, lowering, and artifact
 * contents. This package owns only Vite integration, beginning with this
 * contract check. Virtual modules and dev-server hooks are intentionally
 * deferred to their own versioned products.
 */
export function createPresolveVitePlugin({ compilerProduct }) {
  const manifest = validateCompilerProduct(compilerProduct);
  return Object.freeze({
    name: "presolve:compiler-products",
    enforce: "pre",
    api: Object.freeze({
      schemaVersion: PRESOLVE_VITE_ADAPTER_SCHEMA_VERSION,
      compilerContract: manifest.compiler_contract,
      workspaceSnapshotId: manifest.workspace_snapshot_id,
    }),
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
