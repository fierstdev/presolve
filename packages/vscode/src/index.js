import { initializeLsp } from "@presolve/lsp";

/**
 * Activates a product-free extension facade over the completed LSP authority.
 */
export async function activate(moduleOrPath) {
  const lsp = await initializeLsp(moduleOrPath);
  return Object.freeze({ dispatch: (productBytes, request) => lsp.dispatch(productBytes, request) });
}
