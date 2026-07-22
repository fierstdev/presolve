import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { activate } from "../src/index.js";

const product = await readFile(fileURLToPath(new URL("../../../crates/ezc_core/fixtures/tooling/query-snapshot-v1.json", import.meta.url)));
const snapshot = JSON.parse(product);
const wasm = await readFile(fileURLToPath(new URL("../../compiler-wasm/dist/presolve_compiler_wasm_bg.wasm", import.meta.url)));
const extension = await activate(wasm);
const definition = extension.dispatch(product, { jsonrpc: "2.0", id: "fixture-definition", method: "textDocument/definition", params: { querySemanticId: snapshot.references[0].targetQuerySemanticId } });
if (definition.result?.status !== "ok" || definition.result.operation !== "definition") throw new Error("pinned extension definition fixture drift");
const unsupported = extension.dispatch(product, { jsonrpc: "2.0", id: "fixture-hover", method: "textDocument/hover" });
if (unsupported.result?.status !== "unsupported") throw new Error("pinned extension unsupported fixture drift");
