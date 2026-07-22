import { readFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { initializeLsp } from "../src/index.js";

const product = await readFile(fileURLToPath(new URL("../../../crates/ezc_core/fixtures/tooling/query-snapshot-v1.json", import.meta.url)));
const snapshot = JSON.parse(product);
const fixtures = JSON.parse(await readFile(fileURLToPath(new URL("../../../crates/ezc_core/fixtures/tooling/lsp-v1.json", import.meta.url))));
const wasm = await readFile(fileURLToPath(new URL("../../compiler-wasm/dist/presolve_compiler_wasm_bg.wasm", import.meta.url)));
const lsp = await initializeLsp(wasm);
const target = snapshot.references[0].targetQuerySemanticId;
const unit = snapshot.sourceUnits[0].sourceUnitId;
const cases = { definition:{jsonrpc:"2.0",id:1,method:"textDocument/definition",params:{querySemanticId:target}}, references:{jsonrpc:"2.0",id:2,method:"textDocument/references",params:{querySemanticId:target}}, symbols:{jsonrpc:"2.0",id:3,method:"textDocument/documentSymbol",params:{sourceUnitId:unit}}, diagnostics:{jsonrpc:"2.0",id:4,method:"textDocument/publishDiagnostics",params:{sourceUnitId:unit}}, position:{jsonrpc:"2.0",id:5,method:"presolve/position",params:{sourceUnitId:unit,offset:114}}, unsupported:{jsonrpc:"2.0",id:6,method:"textDocument/hover"}, unknown:{jsonrpc:"2.0",id:7,method:"textDocument/definition",params:{querySemanticId:"query-semantic:missing"}}, invalid:{id:8} };
for (const [name, request] of Object.entries(cases)) if (createHash("sha256").update(JSON.stringify(lsp.dispatch(product, request))).digest("hex") !== fixtures.responseSha256[name]) throw new Error(`L12-D fixture drift: ${name}`);
