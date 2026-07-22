import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { initializeLsp } from "../src/index.js";

const product = await readFile(fileURLToPath(new URL("../../../crates/ezc_core/fixtures/tooling/query-snapshot-v1.json", import.meta.url)));
const snapshot = JSON.parse(product);
const wasm = await readFile(fileURLToPath(new URL("../../compiler-wasm/dist/presolve_compiler_wasm_bg.wasm", import.meta.url)));
const lsp = await initializeLsp(wasm);
const definition = lsp.dispatch(product, { jsonrpc: "2.0", id: 1, method: "textDocument/definition", params: { querySemanticId: snapshot.references[0].targetQuerySemanticId } });
if (definition.result?.status !== "ok" || definition.result.operation !== "definition") throw new Error("definition translation drift");
const hover = lsp.dispatch(product, { jsonrpc: "2.0", id: 2, method: "textDocument/hover" });
if (hover.result?.status !== "unsupported") throw new Error("unsupported method drift");
const invalid = lsp.dispatch(product, { jsonrpc: "2.0", id: 3, method: "textDocument/definition", params: { querySemanticId: "query-semantic:missing" } });
if (invalid.error?.code !== "unknown_query_semantic_id") throw new Error("error mapping drift");
