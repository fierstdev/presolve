#!/usr/bin/env node

import { readFileSync } from "node:fs";

const packageManifests = [
  "package.json",
  "framework/packages/presolve/package.json",
  "packages/cli/package.json",
  "packages/cli-darwin-arm64/package.json",
  "packages/cli-darwin-x64/package.json",
  "packages/cli-linux-x64/package.json",
  "packages/cli-win32-x64/package.json",
  "packages/compiler-wasm/package.json",
  "packages/create-presolve/package.json",
  "packages/language-service/package.json",
  "packages/lsp/package.json",
  "packages/testing/package.json",
  "packages/vscode/package.json",
  "metaframework/packages/application/package.json"
];

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

const root = readJson("package.json");
const requestedVersion = process.argv[2] ?? root.version;

if (!/^\d+\.\d+\.\d+-alpha\.\d+$/.test(requestedVersion)) {
  throw new Error(
    `Expected an alpha version such as 0.1.0-alpha.1; received ${requestedVersion}.`
  );
}

for (const manifestPath of packageManifests) {
  const manifest = readJson(manifestPath);
  if (manifest.version !== requestedVersion) {
    throw new Error(
      `${manifestPath} has ${manifest.version}; expected ${requestedVersion}.`
    );
  }
}

const cargoToml = readFileSync("Cargo.toml", "utf8");
const cargoVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"$/m)?.[1];
if (cargoVersion !== requestedVersion) {
  throw new Error(
    `Cargo.toml has ${cargoVersion ?? "no workspace version"}; expected ${requestedVersion}.`
  );
}

console.log(`Presolve release train is locked at ${requestedVersion}.`);
