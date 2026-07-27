#!/usr/bin/env node

import { existsSync, rmSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";

const root = resolve(import.meta.dirname, "..");
const corpus = resolve(root, "tests/typescript-compatibility");
const generated = resolve(corpus, ".generated");
const pnpm = process.platform === "win32" ? "pnpm.cmd" : "pnpm";

const cases = [
  {
    name: "compatibility",
    arguments: ["-p", "tests/typescript-compatibility/tsconfig.json", "--pretty", "false"],
    expectedStatus: 0,
  },
  {
    name: "project references and source maps",
    arguments: ["-b", "tests/typescript-compatibility/tsconfig.references.json", "--pretty", "false"],
    expectedStatus: 0,
    output: resolve(generated, "project-references/app/main.js.map"),
  },
  {
    name: "diagnostics",
    arguments: ["-p", "tests/typescript-compatibility/diagnostics/tsconfig.json", "--pretty", "false"],
    expectedStatus: 1,
    diagnostics: ["TS2322", "TS2345"],
  },
];

try {
  const version = run(["--version"]);
  if (version.status !== 0 || !version.output.includes("Version 7.0.2")) {
    throw new Error(`expected primary TypeScript 7.0.2, received:\n${version.output}`);
  }

  for (const testCase of cases) {
    const result = run(testCase.arguments);
    if (result.status !== testCase.expectedStatus) {
      throw new Error(
        `${testCase.name} expected exit ${testCase.expectedStatus}, received ${result.status}:\n${result.output}`,
      );
    }
    for (const diagnostic of testCase.diagnostics ?? []) {
      if (!result.output.includes(diagnostic)) {
        throw new Error(`${testCase.name} did not report ${diagnostic}:\n${result.output}`);
      }
    }
    if (testCase.output && !existsSync(testCase.output)) {
      throw new Error(`${testCase.name} did not emit ${testCase.output}`);
    }
  }

  console.log("Validated TypeScript 7.0.2 compatibility corpus.");
} finally {
  rmSync(generated, { recursive: true, force: true });
}

function run(argumentsList) {
  const result = spawnSync(pnpm, ["exec", "tsc", ...argumentsList], {
    cwd: root,
    encoding: "utf8",
  });
  return {
    status: result.status,
    output: `${result.stdout ?? ""}${result.stderr ?? ""}`,
  };
}
