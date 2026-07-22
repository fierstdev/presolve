import assert from "node:assert/strict";
import test from "node:test";
import { createArtifactBuildInvocation, invokeArtifactBuild } from "../src/index.js";

test("artifact build handoff preserves caller-supplied source and output paths", () => {
  const invocation = createArtifactBuildInvocation({
    sourcePath: "src/Counter.tsx",
    outputDirectory: "dist/counter",
  });

  assert.deepEqual(invocation, {
    executable: "presolve",
    arguments: ["build", "src/Counter.tsx", "--out", "dist/counter"],
  });
  assert.ok(Object.isFrozen(invocation));
  assert.ok(Object.isFrozen(invocation.arguments));
});

test("artifact build supports the compiler-owned production profile and passes opaque results unchanged", () => {
  const opaqueResult = Object.freeze({
    exitCode: 0,
    stdout: new Uint8Array([1, 2]),
    stderr: new Uint8Array(),
  });
  let received;
  const result = invokeArtifactBuild(
    {
      sourcePath: "src/Counter.tsx",
      outputDirectory: "dist/counter",
      production: true,
    },
    (invocation) => {
      received = invocation;
      return opaqueResult;
    }
  );

  assert.equal(result, opaqueResult);
  assert.deepEqual(received.arguments, ["build", "src/Counter.tsx", "--out", "dist/counter", "--production"]);
});

test("handoff rejects malformed request shapes without source access", () => {
  assert.throws(
    () => createArtifactBuildInvocation({ sourcePath: "", outputDirectory: "dist" }),
    /sourcePath must be a non-empty string/
  );
  assert.throws(
    () => createArtifactBuildInvocation({ sourcePath: "src/Counter.tsx", outputDirectory: "dist", production: "yes" }),
    /production must be a boolean/
  );
  assert.throws(
    () => invokeArtifactBuild({ sourcePath: "src/Counter.tsx", outputDirectory: "dist" }, null),
    /execute must be a function/
  );
});
