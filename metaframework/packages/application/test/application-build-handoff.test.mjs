import assert from "node:assert/strict";
import test from "node:test";
import { createApplicationBuildInvocation, invokeApplicationBuild } from "../src/index.js";

test("projects one explicit application entry and lexically ordered package mappings", () => {
  const invocation = createApplicationBuildInvocation({
    entryPath: "src/App.tsx",
    outputDirectory: "dist",
    packageContracts: {
      "z-service": "contracts/z-service.json",
      "a-service": "contracts/a-service.json",
    },
    packageRuntimeModules: {
      "z-service": "./runtime/z-service.js",
      "a-service": "./runtime/a-service.js",
    },
    production: true,
  });

  assert.deepEqual(invocation, {
    executable: "presolve",
    arguments: [
      "build", "src/App.tsx", "--out", "dist",
      "--package-contract", "a-service=contracts/a-service.json",
      "--package-contract", "z-service=contracts/z-service.json",
      "--package-runtime", "a-service=./runtime/a-service.js",
      "--package-runtime", "z-service=./runtime/z-service.js",
      "--production",
    ],
  });
  assert.ok(Object.isFrozen(invocation));
  assert.ok(Object.isFrozen(invocation.arguments));
});

test("does not translate executor results", () => {
  const result = Object.freeze({ exitCode: 2, stderr: new Uint8Array([1]) });
  assert.equal(
    invokeApplicationBuild(
      { entryPath: "src/App.tsx", outputDirectory: "dist" },
      () => result,
    ),
    result,
  );
});

test("rejects malformed caller-owned requests", () => {
  assert.throws(
    () => createApplicationBuildInvocation({ entryPath: "", outputDirectory: "dist" }),
    /entryPath must be a non-empty string/,
  );
  assert.throws(
    () => createApplicationBuildInvocation({ entryPath: "src/App.tsx", outputDirectory: "dist", packageContracts: [] }),
    /packageContracts must be an object when provided/,
  );
  assert.throws(
    () => createApplicationBuildInvocation({ entryPath: "src/App.tsx", outputDirectory: "dist", production: "yes" }),
    /production must be a boolean when provided/,
  );
});
