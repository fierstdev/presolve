#!/usr/bin/env node

import {
  existsSync,
  readdirSync,
  readFileSync,
  statSync,
} from "node:fs";
import { basename, resolve } from "node:path";
import { runNpm } from "./npm-command.mjs";

const root = resolve(import.meta.dirname, "..");
const packageDirectories = [
  "cli-darwin-arm64",
  "cli-darwin-x64",
  "cli-linux-x64",
  "cli-win32-x64",
];
const arguments_ = process.argv.slice(2);
const dryRun = takeFlag("--dry-run");
const requireAll = takeFlag("--require-all");

if (arguments_.length !== 1) {
  fail(
    "Usage: node scripts/publish-native-cli.mjs [--dry-run] [--require-all] <artifact-directory>",
  );
}

const artifactDirectory = resolve(root, arguments_[0]);
assert(
  existsSync(artifactDirectory) && statSync(artifactDirectory).isDirectory(),
  `Native CLI artifact directory does not exist: ${artifactDirectory}`,
);

const expectedArtifacts = new Map(
  packageDirectories.map((directory) => {
    const manifest = readJson(
      resolve(root, "packages", directory, "package.json"),
    );
    const filename = `${manifest.name
      .replace(/^@/, "")
      .replace("/", "-")}-${manifest.version}.tgz`;
    return [filename, manifest.name];
  }),
);
const artifacts = readdirSync(artifactDirectory)
  .filter((entry) => entry.endsWith(".tgz"))
  .sort()
  .map((entry) => resolve(artifactDirectory, entry));

assert(artifacts.length > 0, `No native CLI tarballs found in ${artifactDirectory}.`);
for (const artifact of artifacts) {
  assert(
    expectedArtifacts.has(basename(artifact)),
    `Unexpected native CLI artifact: ${basename(artifact)}`,
  );
}
if (requireAll) {
  assert(
    artifacts.length === expectedArtifacts.size &&
      artifacts.every((artifact) => expectedArtifacts.has(basename(artifact))),
    `Expected all ${expectedArtifacts.size} native CLI artifacts; found ${artifacts.length}.`,
  );
}

for (const artifact of artifacts) {
  publish(artifact, true);
}
if (!dryRun) {
  for (const artifact of artifacts) {
    publish(artifact, false);
  }
}

process.stdout.write(
  `${dryRun ? "Validated" : "Published"} ${artifacts.length} native CLI package(s).\n`,
);

function publish(artifact, preflight) {
  const npmArguments = [
    "publish",
    artifact,
    "--tag",
    "alpha",
    "--access",
    "public",
  ];
  if (preflight) {
    npmArguments.push("--dry-run");
  } else {
    npmArguments.push("--provenance");
  }

  const result = runNpm(npmArguments, {
    cwd: root,
    env: process.env,
  });
  if (result.error) {
    fail(`Unable to execute npm publish: ${result.error.message}`);
  }
  if (result.status !== 0) {
    if (result.stdout) process.stderr.write(result.stdout);
    if (result.stderr) process.stderr.write(result.stderr);
    fail(
      `npm publish ${preflight ? "preflight" : "publication"} failed for ${basename(artifact)}.`,
    );
  }
  if (result.stdout) process.stdout.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);
}

function takeFlag(flag) {
  const index = arguments_.indexOf(flag);
  if (index === -1) return false;
  arguments_.splice(index, 1);
  return true;
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

