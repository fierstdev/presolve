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
  "packages/typescript-authority/package.json",
  "packages/vite/package.json"
];

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

const root = readJson("package.json");
const requestedVersion = process.argv[2] ?? root.version;

const prereleaseVersion = requestedVersion.match(
  /^(\d+)\.(\d+)\.(\d+)-(alpha|beta)\.(\d+)$/
);

if (prereleaseVersion === null) {
  throw new Error(
    `Expected an alpha or beta version such as 0.2.0-beta.1; received ${requestedVersion}.`
  );
}

const [, major, minor, patch, channel, prerelease] = prereleaseVersion;
const prereleaseNumber = Number(prerelease);
if (!Number.isSafeInteger(prereleaseNumber) || prereleaseNumber < 1) {
  throw new Error(`Expected a positive ${channel} number; received ${prerelease}.`);
}

for (const manifestPath of packageManifests) {
  const manifest = readJson(manifestPath);
  if (manifest.version !== requestedVersion) {
    throw new Error(
      `${manifestPath} has ${manifest.version}; expected ${requestedVersion}.`
    );
  }
}

for (const manifestPath of [
  "Cargo.toml",
  "crates/presolve_parser/Cargo.toml",
  "crates/presolve_compiler/Cargo.toml",
  "crates/presolve_cli/Cargo.toml"
]) {
  const cargoToml = readFileSync(manifestPath, "utf8");
  if (!cargoToml.includes("version.workspace = true") && !cargoToml.includes(`version = "${requestedVersion}"`)) {
    throw new Error(`${manifestPath} is not aligned to ${requestedVersion}.`);
  }
}

const marketplaceVersion = `${major}.${minor}.${Number(patch) + prereleaseNumber}`;
const vscodeManifest = readJson("packages/vscode/package.json");
if (vscodeManifest.version !== marketplaceVersion) {
  throw new Error(
    `packages/vscode/package.json has ${vscodeManifest.version}; expected Marketplace version ${marketplaceVersion} for ${requestedVersion}.`
  );
}

console.log(
  `Presolve ${channel} release train is locked at ${requestedVersion}; VS Code Marketplace prerelease ${marketplaceVersion}.`
);
