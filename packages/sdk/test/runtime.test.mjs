import assert from "node:assert/strict";
import test from "node:test";

import {
  callCapability,
  installCapabilityTransport,
} from "../dist/runtime.js";
import {
  CapabilityTransportUnavailableError,
} from "../dist/index.js";

test("capability calls flow through the installed transport", async () => {
  const calls = [];
  const restore = installCapabilityTransport({
    async call(request) {
      calls.push(request);
      return { value: "stored" };
    },
  });

  try {
    const output = await callCapability({
      interface: "presolve:kv/store",
      operation: "get",
      input: { key: "example" },
    });

    assert.deepEqual(output, { value: "stored" });
    assert.deepEqual(calls, [
      {
        interface: "presolve:kv/store",
        operation: "get",
        input: { key: "example" },
      },
    ]);
  } finally {
    restore();
  }
});

test("runtime scopes restore the previous transport", async () => {
  const restoreOuter = installCapabilityTransport({
    async call() {
      return "outer";
    },
  });

  try {
    const restoreInner = installCapabilityTransport({
      async call() {
        return "inner";
      },
    });

    try {
      assert.equal(
        await callCapability({
          interface: "example:test/value",
          operation: "read",
          input: null,
        }),
        "inner",
      );
    } finally {
      restoreInner();
    }

    assert.equal(
      await callCapability({
        interface: "example:test/value",
        operation: "read",
        input: null,
      }),
      "outer",
    );
  } finally {
    restoreOuter();
  }
});

test("capability use without an execution adapter fails explicitly", async () => {
  await assert.rejects(
    () =>
      callCapability({
        interface: "presolve:kv/store",
        operation: "get",
        input: { key: "missing" },
      }),
    CapabilityTransportUnavailableError,
  );
});
