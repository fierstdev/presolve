#!/usr/bin/env node

import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";

const root = resolve(import.meta.dirname, "..");
const temporary = mkdtempSync(join(tmpdir(), "presolve-release-"));
const packages = join(temporary, "packages");
const app = join(temporary, "app");
mkdirSync(packages);

try {
  run("pnpm", ["run", "release:prepare"]);
  const tarballs = {
    framework: pack("framework/packages/presolve"),
    cli: pack("packages/cli"),
    platform: packNative(`packages/${platformPackage()}`),
    create: pack("packages/create-presolve"),
  };

  run("pnpm", ["dlx", "--package", tarballs.create, "create-presolve", app]);
  const overrides = {
    presolve: `file:${tarballs.framework}`,
    "@presolve/cli": `file:${tarballs.cli}`,
    [`@presolve/${platformPackage()}`]: `file:${tarballs.platform}`,
  };
  const workspaceConfiguration = [
    "overrides:",
    ...Object.entries(overrides).map(
      ([packageName, specifier]) =>
        `  ${JSON.stringify(packageName)}: ${JSON.stringify(specifier)}`,
    ),
    "",
  ].join("\n");
  writeFileSync(join(app, "pnpm-workspace.yaml"), workspaceConfiguration);

  run("pnpm", ["install", "--ignore-scripts"], app);
  run("pnpm", ["check"], app);
  run("pnpm", ["build"], app);
  run("pnpm", ["deploy:prepare"], app);
  console.log("Verified a newly scaffolded Presolve application from packed packages.");
} finally {
  rmSync(temporary, { recursive: true, force: true });
  rmSync(join(root, "packages", platformPackage(), "bin"), { recursive: true, force: true });
}

function pack(directory) {
  const result = run("pnpm", ["--dir", directory, "pack", "--json", "--pack-destination", packages]);
  const output = JSON.parse(result.stdout);
  return output.tarball ?? output.filename;
}

function packNative(directory) {
  const result = run(
    "npm",
    ["pack", "--json", "--pack-destination", packages],
    resolve(root, directory),
  );
  const [output] = JSON.parse(result.stdout);
  return join(packages, output.filename);
}

function platformPackage() {
  const values = {
    "darwin-arm64": "cli-darwin-arm64",
    "darwin-x64": "cli-darwin-x64",
    "linux-x64": "cli-linux-x64",
    "win32-x64": "cli-win32-x64",
  };
  const name = values[`${process.platform}-${process.arch}`];
  if (!name) throw new Error(`No local platform package for ${process.platform}/${process.arch}.`);
  return name;
}

function run(command, argumentsList, cwd = root) {
  const result = spawnSync(command, argumentsList, { cwd, encoding: "utf8" });
  if (result.status !== 0) {
    process.stderr.write(result.stdout);
    process.stderr.write(result.stderr);
    throw new Error(`${command} ${argumentsList.join(" ")} failed`);
  }
  return result;
}
