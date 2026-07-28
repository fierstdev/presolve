import assert from "node:assert/strict";
import { chmodSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const {
  byteOffsetToUtf16,
  extractDiagnostics,
  inspectWorkspace,
  parseJsonOutput,
  readPresolveVersion,
  resolveCompiler,
  summarizeDiagnostics,
} = require("../extension.cjs");

const root = mkdtempSync(join(tmpdir(), "presolve-vscode-"));
const vscode = { workspace: { workspaceFolders: [{ uri: { fsPath: root } }] } };

assert.equal(inspectWorkspace(vscode).configured, false);
writeFileSync(join(root, "tsconfig.json"), "{}\n");
assert.match(inspectWorkspace(vscode).message, /canonical Presolve app/);
mkdirSync(join(root, "app"));
writeFileSync(join(root, "package.json"), JSON.stringify({
  dependencies: { presolve: "npm:@presolve/framework@0.2.0-beta.12" },
}));
assert.match(inspectWorkspace(vscode).message, /@presolve\/cli/);
mkdirSync(join(root, "node_modules", ".bin"), { recursive: true });
const executable = join(root, "node_modules", ".bin", process.platform === "win32" ? "presolve.cmd" : "presolve");
writeFileSync(executable, process.platform === "win32" ? "@echo off\r\n" : "#!/bin/sh\n");
if (process.platform !== "win32") chmodSync(executable, 0o755);

assert.equal(resolveCompiler(root), executable);
assert.equal(readPresolveVersion(root), "0.2.0-beta.12");
assert.deepEqual(inspectWorkspace(vscode), {
  configured: true,
  root,
  cli: executable,
  version: "0.2.0-beta.12",
  message: "Presolve 0.2.0-beta.12 · compiler-owned diagnostics and explanations are available.",
});

assert.deepEqual(parseJsonOutput('{"ok":true}'), { ok: true });
assert.equal(parseJsonOutput("not json"), null);
assert.equal(byteOffsetToUtf16("aβc", 3), 2);

const source = join(root, "app", "routes", "index.tsx");
const diagnostics = extractDiagnostics({
  parser_diagnostics: [],
  compiler_diagnostics: [{
    code: "PSC1020",
    message: "invalid state write",
    severity: "error",
    primary_provenance: {
      path: "app/routes/index.tsx",
      start: 4,
      end: 12,
      line: 2,
      column: 3,
    },
  }],
  production_diagnostics: [{
    code: "PSC1112",
    message: "invalid root",
    primary_provenance: {
      path: "app/routes/other.tsx",
      start: 0,
      end: 1,
      line: 1,
      column: 1,
    },
  }],
}, source);
assert.deepEqual(diagnostics, [{
  code: "PSC1020",
  message: "invalid state write",
  severity: "error",
  start: 4,
  end: 12,
  line: 2,
  column: 3,
}]);
assert.deepEqual(summarizeDiagnostics(diagnostics), { errors: 1, warnings: 0 });

rmSync(root, { recursive: true, force: true });
