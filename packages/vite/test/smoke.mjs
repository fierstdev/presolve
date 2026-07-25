import { createHash } from "node:crypto";
import {
  createPresolveVitePlugin,
  createPresolveVirtualModuleRegistry,
  PRESOLVE_VITE_ADAPTER_SCHEMA_VERSION,
  PRESOLVE_VIRTUAL_MODULE_PREFIX,
} from "../src/index.js";

const runtime = "export const runtime = 1;\n";
const digest = createHash("sha256").update(runtime).digest("hex");
const plugin = createPresolveVitePlugin({
  compilerProduct: {
    manifest: {
      schema_version: 1,
      compiler_contract: "presolve-application-publication:1",
      workspace_snapshot_id: "fixture-snapshot",
      artifacts: [{ path: "runtime.js", digest }],
    },
  },
});

if (plugin.name !== "presolve:compiler-products" || plugin.enforce !== "pre") {
  throw new Error("adapter must expose its stable Vite plugin identity");
}
if (plugin.api.schemaVersion !== PRESOLVE_VITE_ADAPTER_SCHEMA_VERSION) {
  throw new Error("adapter must expose its schema version");
}
if ("resolveId" in plugin || "load" in plugin || "configureServer" in plugin) {
  throw new Error("the skeleton must not publish virtual modules or dev-server hooks");
}
assertRejects(
  () => createPresolveVitePlugin({ compilerProduct: { manifest: { schema_version: 2 } } }),
  "unsupported manifest schemas must not enter Vite",
);

const registry = createPresolveVirtualModuleRegistry({
  compilerProduct: {
    manifest: {
      schema_version: 1,
      compiler_contract: "presolve-application-publication:1",
      workspace_snapshot_id: "fixture-snapshot",
      artifacts: [{ path: "runtime.js", digest }],
    },
  },
  readArtifact: path => path === "runtime.js" ? runtime : undefined,
});
const virtualId = `${PRESOLVE_VIRTUAL_MODULE_PREFIX}runtime.js`;
const resolvedId = registry.resolveId(virtualId);
if (resolvedId !== `\0${virtualId}`) throw new Error("registry must resolve a versioned virtual module id");
const source = await registry.load(resolvedId);
const expected = [
  "export const artifactPath = \"runtime.js\";",
  "export const content = \"export const runtime = 1;\\n\";",
  "export default content;",
  "",
].join("\n");
if (source !== expected) throw new Error("registry must expose the golden compiler artifact content");
await assertAsyncRejects(
  () => createPresolveVirtualModuleRegistry({
    compilerProduct: { manifest: { schema_version: 1, compiler_contract: "presolve-application-publication:1", artifacts: [{ path: "runtime.js", digest: "0".repeat(64) }] } },
    readArtifact: () => runtime,
  }).load(`\0${virtualId}`),
  "registry must reject artifact content that differs from its compiler digest",
);

function assertRejects(action, message) {
  try {
    action();
  } catch {
    return;
  }
  throw new Error(message);
}

async function assertAsyncRejects(action, message) {
  try {
    await action();
  } catch {
    return;
  }
  throw new Error(message);
}
