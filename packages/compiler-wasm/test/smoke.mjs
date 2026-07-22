import { readFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import init, { query_snapshot_v1 } from "../dist/presolve_compiler_wasm.js";

const product = await readFile(
  fileURLToPath(new URL("../../../crates/presolve_compiler/fixtures/tooling/query-snapshot-v1.json", import.meta.url)),
);
const fixture = JSON.parse(await readFile(fileURLToPath(new URL("../../../crates/presolve_compiler/fixtures/tooling/language-service-wasm-v1.json", import.meta.url))));
const snapshot = JSON.parse(product);
const wasm = await readFile(
  fileURLToPath(new URL("../dist/presolve_compiler_wasm_bg.wasm", import.meta.url)),
);
await init({ module_or_path: wasm });
const unit = snapshot.sourceUnits[0].sourceUnitId;
const target = snapshot.references[0].targetQuerySemanticId;
const cases = {
  position: { schema: "presolve.language-service-wasm-request", version: 1, operation: "position", sourceUnitId: unit, offset: 114 },
  definition: { schema: "presolve.language-service-wasm-request", version: 1, operation: "definition", querySemanticId: target },
  references: { schema: "presolve.language-service-wasm-request", version: 1, operation: "references", querySemanticId: target },
  symbols: { schema: "presolve.language-service-wasm-request", version: 1, operation: "documentSymbols", sourceUnitId: unit },
  diagnostics: { schema: "presolve.language-service-wasm-request", version: 1, operation: "diagnostics", sourceUnitId: unit },
  emptyPosition: { schema: "presolve.language-service-wasm-request", version: 1, operation: "position", sourceUnitId: unit, offset: 139 },
  hover: { schema: "presolve.language-service-wasm-request", version: 1, operation: "hover" },
  unknownSource: { schema: "presolve.language-service-wasm-request", version: 1, operation: "position", sourceUnitId: "source:missing", offset: 0 },
  outOfRange: { schema: "presolve.language-service-wasm-request", version: 1, operation: "position", sourceUnitId: unit, offset: 140 },
  unknownId: { schema: "presolve.language-service-wasm-request", version: 1, operation: "definition", querySemanticId: "query-semantic:missing" },
  invalidRequest: { schema: "wrong", version: 1, operation: "hover" },
};
for (const [name, request] of Object.entries(cases)) {
  const response = query_snapshot_v1(product, new TextEncoder().encode(`${JSON.stringify(request)}\n`));
  if (createHash("sha256").update(response).digest("hex") !== fixture.responseSha256[name]) throw new Error(`L12-C-3 fixture drift: ${name}`);
}
const invalidProduct = query_snapshot_v1(new TextEncoder().encode("{}\n"), new TextEncoder().encode("{\"schema\":\"wrong\"}\n"));
if (createHash("sha256").update(invalidProduct).digest("hex") !== fixture.responseSha256.invalidProduct) throw new Error("L12-C-3 fixture drift: invalidProduct");
