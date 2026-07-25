import { createHash } from "node:crypto";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  buildPresolveProduction,
  createPresolveHmrTransport,
  createPresolveVitePlugin,
  createPresolveVirtualModuleRegistry,
  composeDevelopmentDiagnostics,
  PRESOLVE_HMR_EVENT,
  readPresolveProductionAudit,
  translatePresolveSourceMap,
  PRESOLVE_VITE_ADAPTER_SCHEMA_VERSION,
  PRESOLVE_VIRTUAL_MODULE_PREFIX,
  startPresolveDevServer,
} from "../src/index.js";

const runtime = "export const runtime = 1;\n";
const digest = createHash("sha256").update(runtime).digest("hex");
const auditJson = JSON.stringify({
  schemaVersion: 1,
  buildId: "resume-build:fixture",
  optimizationReportSchemaVersion: 1,
  runtimeCostReportSchemaVersion: 1,
  runtimeTableCount: 0,
  authorityCount: 8,
  invariantCount: 13,
  checks: ["report-schema"],
  status: "passed",
});
const auditDigest = createHash("sha256").update(auditJson).digest("hex");
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

const audit = await readPresolveProductionAudit({
  compilerProduct: {
    manifest: {
      schema_version: 1,
      compiler_contract: "presolve-application-publication:1",
      workspace_snapshot_id: "fixture-snapshot",
      artifacts: [{ path: "production-audit.json", digest: auditDigest }],
    },
  },
  readArtifact: () => auditJson,
});
if (audit.status !== "passed" || audit.buildId !== "resume-build:fixture") {
  throw new Error("Vite must expose the digest-verified compiler production audit");
}
await assertAsyncRejects(
  () => readPresolveProductionAudit({
    compilerProduct: {
      manifest: {
        schema_version: 1,
        compiler_contract: "presolve-application-publication:1",
        workspace_snapshot_id: "fixture-snapshot",
        artifacts: [{ path: "production-audit.json", digest: "0".repeat(64) }],
      },
    },
    readArtifact: () => auditJson,
  }),
  "Vite must reject a production audit whose bytes differ from the compiler manifest",
);

const sourceMapTranslation = translatePresolveSourceMap({
  compilerProduct: {
    manifest: {
      schema_version: 1,
      compiler_contract: "presolve-application-publication:1",
      workspace_snapshot_id: "fixture-snapshot",
      artifacts: [{ path: "runtime.js", digest }],
    },
  },
  sourceMap: { version: 3, sources: [`\0${PRESOLVE_VIRTUAL_MODULE_PREFIX}runtime.js`, "node_modules/dependency.js"] },
});
if (sourceMapTranslation.sources[0].compilerArtifactPath !== "runtime.js"
  || sourceMapTranslation.sources[1].compilerArtifactPath !== undefined) {
  throw new Error("source-map translation must retain only manifest-bound compiler identities");
}

const diagnostics = composeDevelopmentDiagnostics({
  typescript: [{ code: 2322, message: "Type mismatch", file: "src/App.tsx", start: 12 }],
  presolve: [{ code: "PSV1001", message: "Unsupported construct", file: "src/App.tsx", start: 3 }],
});
if (diagnostics.diagnostics.map(diagnostic => diagnostic.authority).join(",") !== "presolve,typescript") {
  throw new Error("development diagnostics must compose and order both authorities");
}

const hmrMessages = [];
const hmr = createPresolveHmrTransport({
  workspaceSnapshotId: "fixture-snapshot",
  send: message => hmrMessages.push(message),
});
const actionUpdate = {
  schemaVersion: 1,
  workspaceSnapshotId: "fixture-snapshot",
  updateId: "hmr-action-1",
  messageClass: "action-update",
  affectedModuleIds: ["virtual:presolve/v1/runtime.js"],
  stateCompatibility: "proven-compatible",
  preserveState: true,
};
if (hmr.publish(actionUpdate, [{ id: "vite-runtime" }]).length !== 0) {
  throw new Error("semantic HMR must suppress Vite module replacement");
}
if (hmrMessages.length !== 1 || hmrMessages[0].event !== PRESOLVE_HMR_EVENT
  || hmrMessages[0].data.preserveState !== true) {
  throw new Error("semantic HMR must transport the compiler-selected update unchanged");
}
const viteModules = [{ id: "vite-style" }];
if (hmr.publish({ ...actionUpdate, updateId: "hmr-style-1", messageClass: "style-update" }, viteModules) !== viteModules) {
  throw new Error("style updates must remain under Vite native CSS HMR");
}
hmr.publish({
  ...actionUpdate,
  updateId: "hmr-full-1",
  messageClass: "full-reload",
  stateCompatibility: "reload-required",
  preserveState: false,
});
if (hmrMessages.at(-1).type !== "full-reload") {
  throw new Error("full reload must use Vite's native transport");
}
assertRejects(
  () => hmr.publish({ ...actionUpdate, updateId: "hmr-unsafe-1", preserveState: false }),
  "the adapter must reject state preservation that was not compiler-proven",
);
const hmrPlugin = createPresolveVitePlugin({
  compilerProduct: {
    manifest: {
      schema_version: 1,
      compiler_contract: "presolve-application-publication:1",
      workspace_snapshot_id: "fixture-snapshot",
      artifacts: [{ path: "runtime.js", digest }],
    },
  },
  hmr: observation => ({ ...actionUpdate, updateId: `hmr-hook-${observation.timestamp}` }),
});
const hmrPluginMessages = [];
hmrPlugin.configureServer({ ws: { send: message => hmrPluginMessages.push(message) }, middlewares: { use() {} } });
const hmrResult = await hmrPlugin.handleHotUpdate({ file: "/work/runtime.ts", timestamp: 10, modules: viteModules });
if (hmrResult.length !== 0 || hmrPluginMessages[0].data.updateId !== "hmr-hook-10") {
  throw new Error("Vite's hot-update hook must only forward the compiler HMR product");
}

const dev = await startPresolveDevServer({
  compilerProduct: {
    manifest: {
      schema_version: 1,
      compiler_contract: "presolve-application-publication:1",
      workspace_snapshot_id: "fixture-snapshot",
      artifacts: [{ path: "runtime.js", digest }],
    },
  },
  readArtifact: () => runtime,
  requestHost: request => request.url === "/route"
    ? { status: 200, headers: { "content-type": "text/plain" }, body: "Presolve route" }
    : undefined,
  diagnostics: () => ({ typescript: [{ code: 2322, message: "Type mismatch" }], presolve: [] }),
  vite: { logLevel: "silent", server: { host: "127.0.0.1", port: 0 } },
});
try {
  const address = dev.server.httpServer.address();
  const response = await fetch(`http://127.0.0.1:${address.port}/route`);
  if (response.status !== 200 || await response.text() !== "Presolve route") {
    throw new Error("presolve dev must route compiler-owned requests without restarting Vite");
  }
  const viteAsset = await fetch(`http://127.0.0.1:${address.port}/@vite/client`);
  if (viteAsset.status !== 200 || !(await viteAsset.text()).includes("createHotContext")) {
    throw new Error("unclaimed JS assets must continue through Vite middleware");
  }
  const published = await dev.publishDiagnostics();
  if (published.diagnostics.length !== 1 || published.diagnostics[0].authority !== "typescript") {
    throw new Error("presolve dev must republish the composed diagnostics product");
  }
} finally {
  await dev.close();
}

const outputDirectory = await mkdtemp(join(tmpdir(), "presolve-vite-build-"));
try {
  const production = await buildPresolveProduction({
    compilerProduct: {
      manifest: {
        schema_version: 1,
        compiler_contract: "presolve-application-publication:1",
        workspace_snapshot_id: "fixture-snapshot",
        entry_component_id: "component:x-app",
        artifacts: [{ path: "runtime.js", digest }],
      },
    },
    readArtifact: () => runtime,
    entryArtifactPath: "runtime.js",
    vite: { logLevel: "silent", build: { outDir: outputDirectory } },
  });
  if (production.entryComponentId !== "component:x-app" || production.entries.length !== 1) {
    throw new Error("production build must map the Vite entry back to the compiler component");
  }
  if (production.sourceMaps.length !== 1 || !production.sourceMaps[0].mapPath.endsWith(".map")) {
    throw new Error("production builds must emit and report Vite source maps");
  }
  const entry = production.entries[0];
  if (entry.compilerArtifactPath !== "runtime.js" || entry.componentId !== "component:x-app") {
    throw new Error("production entry mapping must retain compiler identities");
  }
  const physicalManifest = JSON.parse(await readFile(production.viteManifestPath, "utf8"));
  if (!Object.values(physicalManifest).some(output => output.file === entry.file)) {
    throw new Error("production product must describe a file from Vite's written manifest");
  }
} finally {
  await rm(outputDirectory, { recursive: true, force: true });
}

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
