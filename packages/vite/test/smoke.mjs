import { createHash } from "node:crypto";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  buildPresolveProduction,
  createPresolveVitePlugin,
  createPresolveVirtualModuleRegistry,
  composeDevelopmentDiagnostics,
  PRESOLVE_VITE_ADAPTER_SCHEMA_VERSION,
  PRESOLVE_VIRTUAL_MODULE_PREFIX,
  startPresolveDevServer,
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

const diagnostics = composeDevelopmentDiagnostics({
  typescript: [{ code: 2322, message: "Type mismatch", file: "src/App.tsx", start: 12 }],
  presolve: [{ code: "PSV1001", message: "Unsupported construct", file: "src/App.tsx", start: 3 }],
});
if (diagnostics.diagnostics.map(diagnostic => diagnostic.authority).join(",") !== "presolve,typescript") {
  throw new Error("development diagnostics must compose and order both authorities");
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
