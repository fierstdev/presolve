#!/usr/bin/env node

import { chmodSync, copyFileSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { platform, arch } from "node:process";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";

const root = resolve(import.meta.dirname, "..");
const target = targetName();
const packageName = packageFor(target);
const executable = platform === "win32" ? "presolve.exe" : "presolve";
const destination = resolve(root, "packages", packageName, "bin", executable);

run("cargo", ["build", "--release", "--locked", "-p", "presolve-cli", "--target", target]);
mkdirSync(resolve(destination, ".."), { recursive: true });
copyFileSync(resolve(root, "target", target, "release", executable), destination);
if (platform !== "win32") chmodSync(destination, 0o755);
console.log(`Staged ${packageName}/${executable} for packaging.`);

function targetName() {
  if (platform === "darwin" && arch === "arm64") return "aarch64-apple-darwin";
  if (platform === "darwin" && arch === "x64") return "x86_64-apple-darwin";
  if (platform === "linux" && arch === "x64") return "x86_64-unknown-linux-gnu";
  if (platform === "win32" && arch === "x64") return "x86_64-pc-windows-msvc";
  throw new Error(`No Presolve CLI package exists for ${platform}/${arch}.`);
}

function packageFor(target) {
  const values = {
    "aarch64-apple-darwin": "cli-darwin-arm64",
    "x86_64-apple-darwin": "cli-darwin-x64",
    "x86_64-unknown-linux-gnu": "cli-linux-x64",
    "x86_64-pc-windows-msvc": "cli-win32-x64",
  };
  return values[target];
}

function run(command, argumentsList) {
  const result = spawnSync(command, argumentsList, { cwd: root, stdio: "inherit" });
  if (result.status !== 0) process.exit(result.status ?? 1);
}
