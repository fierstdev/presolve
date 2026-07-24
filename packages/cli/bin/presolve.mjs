#!/usr/bin/env node
import { createRequire } from "node:module";
import { existsSync } from "node:fs";
import { spawnSync } from "node:child_process";

const require = createRequire(import.meta.url);
const platformPackage = new Map([
  ["darwin-arm64", "@presolve/cli-darwin-arm64"],
  ["darwin-x64", "@presolve/cli-darwin-x64"],
  ["linux-x64", "@presolve/cli-linux-x64"],
  ["win32-x64", "@presolve/cli-win32-x64"],
]).get(`${process.platform}-${process.arch}`);

const binary = process.env.PRESOLVE_BINARY || resolvePlatformBinary(platformPackage);
if (!binary) {
  console.error(
    `Presolve 0.1 alpha does not include a CLI binary for ${process.platform}-${process.arch}. ` +
      "See https://github.com/fierstdev/presolve#supported-platforms."
  );
  process.exit(1);
}

const child = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });
if (child.error) {
  console.error(`Unable to start Presolve: ${child.error.message}`);
  process.exit(1);
}
process.exit(child.status ?? 1);

function resolvePlatformBinary(packageName) {
  if (!packageName) return null;
  const executable = process.platform === "win32" ? "presolve.exe" : "presolve";
  try {
    const manifest = require.resolve(`${packageName}/package.json`);
    const binaryPath = new URL(`./bin/${executable}`, `file://${manifest}`).pathname;
    return existsSync(binaryPath) ? binaryPath : null;
  } catch {
    return null;
  }
}
