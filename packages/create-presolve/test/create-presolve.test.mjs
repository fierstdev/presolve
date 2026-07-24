import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";

const root = mkdtempSync(join(tmpdir(), "create-presolve-"));
const target = join(root, "docs-site");
const result = spawnSync(process.execPath, ["bin/create-presolve.mjs", target], {
  cwd: new URL("..", import.meta.url),
  encoding: "utf8",
});
assert.equal(result.status, 0, result.stderr);
assert.ok(existsSync(join(target, "app/routes/index.tsx")));
assert.ok(existsSync(join(target, "app/routes/docs/getting-started.tsx")));
assert.match(readFileSync(join(target, "package.json"), "utf8"), /deploy:prepare/);
const second = spawnSync(process.execPath, ["bin/create-presolve.mjs", target], {
  cwd: new URL("..", import.meta.url),
  encoding: "utf8",
});
assert.equal(second.status, 2);
rmSync(root, { recursive: true, force: true });
