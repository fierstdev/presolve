import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import init, { query_snapshot_v1 } from "../dist/presolve_compiler_wasm.js";

const product = await readFile(
  fileURLToPath(new URL("../../../crates/ezc_core/fixtures/tooling/query-snapshot-v1.json", import.meta.url)),
);
const request = new TextEncoder().encode(
  "{\"schema\":\"presolve.language-service-wasm-request\",\"version\":1,\"operation\":\"hover\"}\n",
);
const wasm = await readFile(
  fileURLToPath(new URL("../dist/presolve_compiler_wasm_bg.wasm", import.meta.url)),
);
await init({ module_or_path: wasm });
const response = JSON.parse(new TextDecoder().decode(query_snapshot_v1(product, request)));
if (response.status !== "unsupported" || response.capability !== "hover") {
  throw new Error("L12-C-3 WASM binding did not preserve the Rust unsupported response");
}
