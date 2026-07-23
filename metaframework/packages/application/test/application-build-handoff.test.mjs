import assert from "node:assert/strict";
import test from "node:test";
import {
  createApplicationBuildInvocation,
  createApplicationPublicationInvocation,
  createApplicationCommandInvocation,
  createApplicationWatchOnceInvocation,
  createApplicationWorkspaceInvocation,
  invokeApplicationBuild,
  invokeApplicationPublication,
  invokeApplicationCommand,
  invokeApplicationDevelopment,
} from "../src/index.js";

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

test("projects the canonical explicit multi-source application publication command", () => {
  const invocation = createApplicationPublicationInvocation({
    configurationPath: "presolve.json",
    sources: ["src/App.tsx=src/App.tsx", "src/Card.tsx=src/Card.tsx"],
    entryPath: "src/App.tsx",
    outputDirectory: "dist",
    packageContracts: { "z-service": "contracts/z.json", "a-service": "contracts/a.json" },
    packageRuntimeModules: { "z-service": "runtime/z.js", "a-service": "runtime/a.js" },
    production: true,
  });
  assert.deepEqual(invocation, {
    executable: "presolve",
    arguments: [
      "application", "build", "--config", "presolve.json",
      "--source", "src/App.tsx=src/App.tsx",
      "--source", "src/Card.tsx=src/Card.tsx",
      "--entry", "src/App.tsx", "--out", "dist",
      "--package-contract", "a-service=contracts/a.json",
      "--package-contract", "z-service=contracts/z.json",
      "--package-runtime", "a-service=runtime/a.js",
      "--package-runtime", "z-service=runtime/z.js",
      "--production",
    ],
  });
  const result = Object.freeze({ exitCode: 0 });
  assert.equal(invokeApplicationPublication({
    configurationPath: "presolve.json",
    sources: ["src/App.tsx=src/App.tsx"],
    entryPath: "src/App.tsx",
    outputDirectory: "dist",
  }, () => result), result);
});

test("rejects malformed caller-owned requests", () => {
  assert.throws(
    () => createApplicationBuildInvocation({ entryPath: "", outputDirectory: "dist" }),
    /entryPath must be a non-empty string/,
  );
  assert.throws(
    () => createApplicationPublicationInvocation({ configurationPath: "presolve.json", sources: [], entryPath: "src/App.tsx", outputDirectory: "dist" }),
    /sources must be a non-empty array/,
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

test("projects only the existing explicit workspace and watch-once commands", () => {
  const request = {
    configurationPath: "presolve.json",
    sources: ["app.tsx=src/App.tsx", "theme.ts=src/theme.ts"],
    format: "json",
  };
  assert.deepEqual(createApplicationWorkspaceInvocation({ ...request, verifyCleanEquivalence: true }), {
    executable: "presolve",
    arguments: ["workspace", "--config", "presolve.json", "--source", "app.tsx=src/App.tsx", "--source", "theme.ts=src/theme.ts", "--format", "json", "--verify-clean-equivalence"],
  });
  assert.deepEqual(createApplicationWatchOnceInvocation(request), {
    executable: "presolve",
    arguments: ["watch", "--once", "--config", "presolve.json", "--source", "app.tsx=src/App.tsx", "--source", "theme.ts=src/theme.ts", "--format", "json"],
  });
  const result = Object.freeze({ outcome: "unchanged" });
  assert.equal(invokeApplicationDevelopment(request, createApplicationWatchOnceInvocation, () => result), result);
});

test("rejects unsupported development handoff options", () => {
  assert.throws(
    () => createApplicationWorkspaceInvocation({ configurationPath: "presolve.json", sources: [] }),
    /sources must be a non-empty array/,
  );
  assert.throws(
    () => createApplicationWatchOnceInvocation({ configurationPath: "presolve.json", sources: ["app.tsx=src/App.tsx"], verifyCleanEquivalence: true }),
    /only supported for workspace/,
  );
});

test("selects a versioned command envelope without decoding executor output", () => {
  const request = {
    schemaVersion: 1,
    command: "build",
    input: { entryPath: "src/App.tsx", outputDirectory: "dist" },
  };
  assert.deepEqual(createApplicationCommandInvocation(request), {
    executable: "presolve",
    arguments: ["build", "src/App.tsx", "--out", "dist"],
  });
  const result = Object.freeze({ stdout: new Uint8Array([1]) });
  assert.equal(invokeApplicationCommand(request, () => result), result);
  assert.throws(
    () => createApplicationCommandInvocation({ ...request, schemaVersion: 2 }),
    /schemaVersion must be 1/,
  );
  assert.throws(
    () => createApplicationCommandInvocation({ ...request, command: "dev" }),
    /command must be build, application-build, workspace, or watch-once/,
  );
  assert.deepEqual(createApplicationCommandInvocation({
    schemaVersion: 1,
    command: "application-build",
    input: {
      configurationPath: "presolve.json",
      sources: ["src/App.tsx=src/App.tsx"],
      entryPath: "src/App.tsx",
      outputDirectory: "dist",
    },
  }), {
    executable: "presolve",
    arguments: ["application", "build", "--config", "presolve.json", "--source", "src/App.tsx=src/App.tsx", "--entry", "src/App.tsx", "--out", "dist"],
  });
});
