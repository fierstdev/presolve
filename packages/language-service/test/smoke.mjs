import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { initializeLanguageService } from "../src/index.js";

const product = await readFile(fileURLToPath(new URL("../../../crates/ezc_core/fixtures/tooling/query-snapshot-v1.json", import.meta.url)));
const snapshot = JSON.parse(product);
const wasm = await readFile(fileURLToPath(new URL("../../compiler-wasm/dist/presolve_compiler_wasm_bg.wasm", import.meta.url)));
const service = await initializeLanguageService(wasm);
const result = service.query(product, { schema: "presolve.language-service-wasm-request", version: 1, operation: "hover" });
if (result.status !== "unsupported" || result.capability !== "hover") throw new Error("language service must preserve the WASM result");
const position = service.query(product, { schema: "presolve.language-service-wasm-request", version: 1, operation: "position", sourceUnitId: snapshot.sourceUnits[0].sourceUnitId, offset: 114 });
if (position.status !== "ok" || position.result.records.length !== 6) throw new Error("language service must preserve WASM position records");
