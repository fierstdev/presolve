import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";

const packageRoot = new URL("..", import.meta.url);
const help = spawnSync(process.execPath, ["bin/create-presolve.mjs", "--help"], {
  cwd: packageRoot,
  encoding: "utf8",
});
assert.equal(help.status, 0, help.stderr);
assert.match(help.stdout, /Create a Presolve application/);
assert.match(help.stdout, /never overwrites an existing path/);
const version = spawnSync(process.execPath, ["bin/create-presolve.mjs", "--version"], {
  cwd: packageRoot,
  encoding: "utf8",
});
assert.equal(version.status, 0, version.stderr);
assert.match(version.stdout, /^0\.2\.0-beta\.\d+\s*$/);
const unknownOption = spawnSync(process.execPath, ["bin/create-presolve.mjs", "--unknown"], {
  cwd: packageRoot,
  encoding: "utf8",
});
assert.equal(unknownOption.status, 2);
assert.match(unknownOption.stderr, /Unknown option/);

const root = mkdtempSync(join(tmpdir(), "create-presolve-"));
const target = join(root, "docs-site");
const result = spawnSync(process.execPath, ["bin/create-presolve.mjs", target], {
  cwd: packageRoot,
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
for (const path of ["app/components/README.md", "server/README.md", "app/app.css", "app/index.html", "assets/README.md", "public/favicon.svg", "public/robots.txt", "tests/README.md", ".env.example"]) {
  assert.ok(existsSync(join(target, path)), `missing conventional platform path ${path}`);
}
assert.match(readFileSync(join(target, ".env.example"), "utf8"), /PRESOLVE_PUBLIC_APP_NAME/);
assert.match(readFileSync(join(target, ".gitignore"), "utf8"), /!\.env\.example/);
const manifest = JSON.parse(readFileSync(join(target, "package.json"), "utf8"));
assert.equal(manifest.packageManager, "pnpm@11.17.0");
assert.equal(manifest.dependencies.presolve, "npm:@presolve/framework@0.2.0-beta.21");
assert.equal(manifest.devDependencies["@presolve/cli"], "0.2.0-beta.21");
assert.equal(manifest.devDependencies["@presolve/typescript-authority"], "0.2.0-beta.21");
assert.equal(manifest.devDependencies.vite, "^7.0.0");
assert.ok(manifest.scripts["deploy:prepare"]);
assert.equal(manifest.scripts["deploy:node:prepare"], "presolve deploy node --prepare");
assert.match(readFileSync(join(target, "README.md"), "utf8"), /deploy:node:prepare/);
assert.match(readFileSync(join(target, "README.md"), "utf8"), /app\/app\.css/);
assert.match(readFileSync(join(target, "README.md"), "utf8"), /Vite has a separate, bounded role/);
assert.match(readFileSync(join(target, "README.md"), "utf8"), /exact bytes/);
const documentTemplate = readFileSync(join(target, "app/index.html"), "utf8");
assert.match(documentTemplate, /\{\{ head \}\}/);
assert.match(documentTemplate, /width=device-width/);
assert.match(documentTemplate, /\/favicon\.svg/);
const appShell = readFileSync(join(target, "app/app.tsx"), "utf8");
assert.match(appShell, /<slot \/>/);
assert.doesNotMatch(appShell, /<main>|stylesheet|<html>|<head>/);
const homeRoute = readFileSync(join(target, "app/routes/index.tsx"), "utf8");
assert.match(homeRoute, /state\(0\)/);
assert.match(homeRoute, /get nextCount\(\)/);
assert.match(homeRoute, /Next increment:/);
assert.match(homeRoute, /action\(\(\) =>/);
assert.match(homeRoute, /<main/);
assert.doesNotMatch(homeRoute, /@component|@action/);
const globalStyles = readFileSync(join(target, "app/app.css"), "utf8");
assert.match(globalStyles, /@media \(min-width: 48rem\)/);
assert.match(globalStyles, /prefers-reduced-motion/);
assert.match(globalStyles, /:focus-visible/);
const second = spawnSync(process.execPath, ["bin/create-presolve.mjs", target], {
  cwd: packageRoot,
  encoding: "utf8",
});
assert.equal(second.status, 2);
rmSync(root, { recursive: true, force: true });
