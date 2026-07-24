import initializeWasm, { query_snapshot_v1 } from "@presolve/compiler-wasm";

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/**
 * Initializes the compiler-owned WASM artifact. The caller supplies its module
 * bytes/path and keeps every query-snapshot byte sequence outside this package.
 */
export async function initializeLanguageService(moduleOrPath) {
  await initializeWasm({ module_or_path: moduleOrPath });
  return Object.freeze({ query });
}

/**
 * Projects one caller-owned compiler product through the sole WASM authority.
 */
export function query(productBytes, request) {
  const requestBytes = encoder.encode(`${JSON.stringify(request)}\n`);
  return JSON.parse(decoder.decode(query_snapshot_v1(productBytes, requestBytes)));
}
