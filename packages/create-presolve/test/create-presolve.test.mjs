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
assert.ok(existsSync(join(target, "app/app.tsx")));
assert.ok(existsSync(join(target, "app/app.css")));
assert.ok(existsSync(join(target, "app/index.html")));
assert.ok(existsSync(join(target, "app/routes/index.tsx")));
assert.ok(existsSync(join(target, "app/routes/docs/getting-started.tsx")));
for (const route of ["app/app.tsx", "app/routes/index.tsx", "app/routes/docs/index.tsx", "app/routes/docs/getting-started.tsx"]) {
  const source = readFileSync(join(target, route), "utf8");
  assert.match(source, /extends Component/);
  assert.doesNotMatch(source, /@component|@action|@computed|@form|@resource|@slot|@loader|@serverAction/);
}
for (const path of ["app/components/README.md", "server/README.md", "app/app.css", "app/index.html", "assets/README.md", "public/robots.txt", "tests/README.md", ".env.example"]) {
  assert.ok(existsSync(join(target, path)), `missing conventional platform path ${path}`);
}
assert.match(readFileSync(join(target, ".env.example"), "utf8"), /PRESOLVE_PUBLIC_APP_NAME/);
const manifest = JSON.parse(readFileSync(join(target, "package.json"), "utf8"));
assert.equal(manifest.packageManager, "pnpm@11.17.0");
assert.equal(manifest.dependencies.presolve, "npm:@presolve/framework@0.2.0-beta.6");
assert.equal(manifest.devDependencies["@presolve/typescript-authority"], "0.2.0-beta.6");
assert.ok(manifest.scripts["deploy:prepare"]);
assert.equal(manifest.scripts["deploy:node:prepare"], "presolve deploy node --prepare");
assert.match(readFileSync(join(target, "README.md"), "utf8"), /deploy:node:prepare/);
assert.match(readFileSync(join(target, "README.md"), "utf8"), /app\/app\.css/);
assert.match(readFileSync(join(target, "app/index.html"), "utf8"), /\{\{ head \}\}/);
assert.doesNotMatch(readFileSync(join(target, "app/app.tsx"), "utf8"), /<main>|stylesheet/);
const second = spawnSync(process.execPath, ["bin/create-presolve.mjs", target], {
  cwd: new URL("..", import.meta.url),
  encoding: "utf8",
});
assert.equal(second.status, 2);
rmSync(root, { recursive: true, force: true });
