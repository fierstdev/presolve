#!/usr/bin/env node

import {
  existsSync,
  mkdirSync,
  readFileSync,
  statSync,
} from "node:fs";
import { resolve } from "node:path";
import { arch, platform } from "node:process";
import { spawnSync } from "node:child_process";

const root = resolve(import.meta.dirname, "..");
const packageDefinitions = {
  "cli-darwin-arm64": {
    name: "@presolve/cli-darwin-arm64",
    os: "darwin",
    cpu: "arm64",
    executable: "presolve",
  },
  "cli-darwin-x64": {
    name: "@presolve/cli-darwin-x64",
    os: "darwin",
    cpu: "x64",
    executable: "presolve",
  },
  "cli-linux-x64": {
    name: "@presolve/cli-linux-x64",
    os: "linux",
    cpu: "x64",
    executable: "presolve",
  },
  "cli-win32-x64": {
    name: "@presolve/cli-win32-x64",
    os: "win32",
    cpu: "x64",
    executable: "presolve.exe",
  },
};

const requestedPackage = process.argv[2];
const destinationArgument = process.argv[3];
if (requestedPackage === undefined || destinationArgument === undefined) {
  fail("Usage: node scripts/pack-native-cli.mjs <package|--host> <destination>");
}

const packageName =
  requestedPackage === "--host"
    ? hostPackage()
    : requestedPackage;
const definition = packageDefinitions[packageName];
if (definition === undefined) {
  fail(`Unsupported native CLI package: ${packageName}`);
}

const packageRoot = resolve(root, "packages", packageName);
const destination = resolve(root, destinationArgument);
const manifestPath = resolve(packageRoot, "package.json");
const executablePath = resolve(packageRoot, "bin", definition.executable);
const rootManifest = readJson(resolve(root, "package.json"));
const manifest = readJson(manifestPath);

assert(manifest.name === definition.name, `${manifestPath} has an unexpected package name.`);
assert(
  manifest.version === rootManifest.version,
  `${manifestPath} has version ${manifest.version}; expected ${rootManifest.version}.`,
);
assert(
  Array.isArray(manifest.os) && manifest.os.length === 1 && manifest.os[0] === definition.os,
  `${manifestPath} has an unexpected operating-system constraint.`,
);
assert(
  Array.isArray(manifest.cpu) &&
    manifest.cpu.length === 1 &&
    manifest.cpu[0] === definition.cpu,
  `${manifestPath} has an unexpected CPU constraint.`,
);
assert(
  Array.isArray(manifest.files) && manifest.files.includes("bin"),
  `${manifestPath} must publish the bin directory.`,
);
assert(
  existsSync(executablePath) && statSync(executablePath).isFile(),
  `Expected staged native executable at ${executablePath}.`,
);

mkdirSync(destination, { recursive: true });
const npm = platform === "win32" ? "npm.cmd" : "npm";
const result = spawnSync(
  npm,
  ["pack", packageRoot, "--pack-destination", destination, "--json"],
  {
    cwd: root,
    encoding: "utf8",
  },
);
if (result.status !== 0) {
  if (result.stdout) process.stderr.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);
  process.exit(result.status ?? 1);
}

let packed;
try {
  const parsed = JSON.parse(result.stdout);
  packed = Array.isArray(parsed) ? parsed[0] : parsed;
} catch {
  fail(`npm pack returned invalid JSON:\n${result.stdout}`);
}

assert(packed?.name === definition.name, "npm packed an unexpected package.");
assert(packed?.version === rootManifest.version, "npm packed an unexpected version.");
assert(
  packed?.files?.some((file) => file.path === `bin/${definition.executable}`),
  `Packed tarball does not contain bin/${definition.executable}.`,
);

const tarball = resolve(destination, packed.filename);
assert(existsSync(tarball) && statSync(tarball).isFile(), `Missing packed tarball ${tarball}.`);

process.stdout.write(
  `${JSON.stringify({
    name: packed.name,
    version: packed.version,
    filename: packed.filename,
    tarball,
  })}\n`,
);

function hostPackage() {
  const match = Object.entries(packageDefinitions).find(
    ([, value]) => value.os === platform && value.cpu === arch,
  );
  if (match === undefined) {
    fail(`No Presolve native CLI package exists for ${platform}/${arch}.`);
  }
  return match[0];
}

function readJson(path) {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    fail(`Unable to read ${path}: ${error.message}`);
  }
}

function assert(condition, message) {
  if (!condition) fail(message);
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
