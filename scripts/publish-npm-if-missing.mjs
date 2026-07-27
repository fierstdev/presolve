#!/usr/bin/env node

import { readFileSync, statSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";

const [publisher, packagePath, ...publishArguments] = process.argv.slice(2);
if (!["npm", "pnpm"].includes(publisher) || !packagePath) {
  console.error(
    "Usage: publish-npm-if-missing.mjs <npm|pnpm> <package-directory|tarball> [publish arguments...]",
  );
  process.exit(2);
}

const manifest = readManifest(packagePath);
const packageIdentity = `${manifest.name}@${manifest.version}`;
const lookup = spawnSync(
  "npm",
  ["view", packageIdentity, "version", "--json"],
  { encoding: "utf8" },
);

if (lookup.status === 0 && JSON.parse(lookup.stdout) === manifest.version) {
  console.log(`Already published ${packageIdentity}; skipping.`);
  process.exit(0);
}

const command =
  publisher === "npm"
    ? ["npm", ["publish", packagePath, ...publishArguments]]
    : ["pnpm", ["--dir", packagePath, "publish", ...publishArguments]];
const publication = spawnSync(command[0], command[1], { stdio: "inherit" });
if (publication.error) throw publication.error;
process.exit(publication.status ?? 1);

function readManifest(path) {
  const absolutePath = resolve(path);
  if (statSync(absolutePath).isDirectory()) {
    return JSON.parse(readFileSync(resolve(absolutePath, "package.json"), "utf8"));
  }

  const extraction = spawnSync(
    "tar",
    ["-xOf", absolutePath, "package/package.json"],
    { encoding: "utf8" },
  );
  if (extraction.error) throw extraction.error;
  if (extraction.status !== 0) {
    process.stderr.write(extraction.stderr);
    process.exit(extraction.status ?? 1);
  }
  return JSON.parse(extraction.stdout);
}
