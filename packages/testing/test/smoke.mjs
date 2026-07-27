import { createHash } from "node:crypto";
import {
  createPresolvePlaywrightProject,
  createPresolveVitestConfig,
  declaredTest,
  equalCanonicalBytes,
} from "../src/index.js";

if (!equalCanonicalBytes(new Uint8Array([1, 2]), new Uint8Array([1, 2]))) throw new Error("equal bytes must compare");
if (equalCanonicalBytes(new Uint8Array([1]), new Uint8Array([2]))) throw new Error("different bytes must not compare");
const test = declaredTest({ name: "compiler", command: "cargo test -p presolve-compiler --lib", lane: "deterministic" });
if (test.lane !== "deterministic" || !Object.isFrozen(test)) throw new Error("declared test metadata must be immutable");

const runtime = "export const runtime = 1;\n";
const vitest = createPresolveVitestConfig({
  compilerProduct: {
    manifest: {
      schema_version: 1,
      compiler_contract: "presolve-application-publication:1",
      workspace_snapshot_id: "testing-fixture",
      artifacts: [{ path: "runtime.js", digest: createHash("sha256").update(runtime).digest("hex") }],
    },
  },
  readArtifact: () => runtime,
  fixtures: [{ name: "home", route: "/" }],
});
if (vitest.runner !== "vitest" || vitest.vite.plugins.length !== 1 || vitest.fixtures[0].route !== "/") {
  throw new Error("Vitest integration must preserve the compiler Vite plugin and declared route fixture");
}
const playwright = createPresolvePlaywrightProject({
  compilerProduct: {
    manifest: {
      schema_version: 1,
      compiler_contract: "presolve-application-publication:1",
      workspace_snapshot_id: "testing-fixture",
      artifacts: [{ path: "runtime.js", digest: createHash("sha256").update(runtime).digest("hex") }],
    },
  },
  readArtifact: () => runtime,
  baseURL: "http://127.0.0.1:4173/",
  fixtures: [{ name: "home", route: "/" }],
});
if (playwright.runner !== "playwright" || playwright.use.baseURL !== "http://127.0.0.1:4173") {
  throw new Error("Playwright integration must retain the caller-owned test origin");
}
