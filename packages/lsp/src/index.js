import { initializeLanguageService } from "@presolve/language-service";

const requestSchema = "presolve.language-service-wasm-request";
const supported = new Map([
  ["textDocument/definition", (params) => ({ operation: "definition", querySemanticId: params.querySemanticId })],
  ["textDocument/references", (params) => ({ operation: "references", querySemanticId: params.querySemanticId })],
  ["textDocument/documentSymbol", (params) => ({ operation: "documentSymbols", sourceUnitId: params.sourceUnitId })],
  ["textDocument/publishDiagnostics", (params) => ({ operation: "diagnostics", sourceUnitId: params.sourceUnitId })],
  ["presolve/position", (params) => ({ operation: "position", sourceUnitId: params.sourceUnitId, offset: params.offset })],
]);

export async function initializeLsp(moduleOrPath) {
  const languageService = await initializeLanguageService(moduleOrPath);
  return Object.freeze({ dispatch: (productBytes, request) => dispatch(languageService, productBytes, request) });
}

function dispatch(languageService, productBytes, request) {
  const id = request?.id ?? null;
  if (request?.jsonrpc !== "2.0" || typeof request?.method !== "string") return error(id, "invalid_request");
  const project = supported.get(request.method);
  if (!project) return { jsonrpc: "2.0", id, result: { status: "unsupported", capability: request.method } };
  const response = languageService.query(productBytes, { schema: requestSchema, version: 1, ...project(request.params ?? {}) });
  return response.status === "error" ? error(id, response.code) : { jsonrpc: "2.0", id, result: response };
}

function error(id, code) {
  return { jsonrpc: "2.0", id, error: { code } };
}
