import assert from "node:assert/strict";
import test from "node:test";
import { createExplicitProjectInvocation, invokeExplicitProject } from "../src/index.js";

const sources = [
  { logicalPath: "z.tsx", relativePath: "src/Z.tsx" },
  { logicalPath: "a.tsx", relativePath: "src/A.tsx" },
];

test("build handoff preserves each caller-supplied argument and source order", () => {
  const invocation = createExplicitProjectInvocation({
    command: "build",
    configurationPath: "presolve.json",
    sources,
  });

  assert.deepEqual(invocation, {
    executable: "presolve",
    arguments: [
      "build",
      "--config",
      "presolve.json",
      "--source",
      "z.tsx=src/Z.tsx",
      "--source",
      "a.tsx=src/A.tsx",
      "--format",
      "json",
    ],
  });
  assert.ok(Object.isFrozen(invocation));
  assert.ok(Object.isFrozen(invocation.arguments));
});

test("check handoff passes its opaque executor result through unchanged", () => {
  const opaqueResult = Object.freeze({
    exitCode: 0,
    stdout: new Uint8Array([1, 2]),
    stderr: new Uint8Array(),
  });
  let received;
  const result = invokeExplicitProject(
    {
      command: "check",
      configurationPath: "presolve.json",
      sources: [{ logicalPath: "Counter.tsx", relativePath: "src/Counter.tsx" }],
    },
    (invocation) => {
      received = invocation;
      return opaqueResult;
    }
  );

  assert.equal(result, opaqueResult);
  assert.deepEqual(received.arguments, [
    "check",
    "--config",
    "presolve.json",
    "--source",
    "Counter.tsx=src/Counter.tsx",
    "--format",
    "json",
  ]);
});

test("handoff rejects malformed request shapes without source access", () => {
  assert.throws(
    () => createExplicitProjectInvocation({ command: "build", configurationPath: "presolve.json", sources: [] }),
    /sources must be a non-empty array/
  );
  assert.throws(
    () => createExplicitProjectInvocation({ command: "watch", configurationPath: "presolve.json", sources }),
    /command must be build or check/
  );
  assert.throws(
    () => invokeExplicitProject({ command: "check", configurationPath: "presolve.json", sources }, null),
    /execute must be a function/
  );
});
