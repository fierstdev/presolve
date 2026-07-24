import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { inspectWorkspace } = require("../extension.cjs");
const root = mkdtempSync(join(tmpdir(), "presolve-vscode-"));
const vscode = { workspace: { workspaceFolders: [{ uri: { fsPath: root } }] } };

assert.equal(inspectWorkspace(vscode).configured, false);
writeFileSync(join(root, "tsconfig.json"), "{}\n");
assert.deepEqual(inspectWorkspace(vscode), {
  configured: true,
  message: "Presolve is using this workspace's TypeScript project configuration.",
});
rmSync(root, { recursive: true, force: true });
