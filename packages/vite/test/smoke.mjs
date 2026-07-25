import { createPresolveVitePlugin, PRESOLVE_VITE_ADAPTER_SCHEMA_VERSION } from "../src/index.js";

const plugin = createPresolveVitePlugin({
  compilerProduct: {
    manifest: {
      schema_version: 1,
      compiler_contract: "presolve-application-publication:1",
      workspace_snapshot_id: "fixture-snapshot",
      artifacts: [],
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

function assertRejects(action, message) {
  try {
    action();
  } catch {
    return;
  }
  throw new Error(message);
}
