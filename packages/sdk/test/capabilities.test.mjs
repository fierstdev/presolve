import assert from "node:assert/strict";
import test from "node:test";

import {
  CapabilityProtocolError,
  kv,
  objects,
} from "../dist/index.js";
import { installCapabilityTransport } from "../dist/runtime.js";

test("typed KV API projects semantic operations", async () => {
  const calls = [];
  const restore = installCapabilityTransport({
    async call(request) {
      calls.push(request);

      if (request.operation === "get") return { value: "stored" };
      if (request.operation === "delete") return { deleted: true };
      return {};
    },
  });

  try {
    await kv.set("session", "abc");
    assert.equal(await kv.get("session"), "stored");
    assert.equal(await kv.delete("session"), true);
  } finally {
    restore();
  }

  assert.deepEqual(calls, [
    {
      interface: "presolve:kv/store",
      operation: "set",
      input: { key: "session", value: "abc" },
    },
    {
      interface: "presolve:kv/store",
      operation: "get",
      input: { key: "session" },
    },
    {
      interface: "presolve:kv/store",
      operation: "delete",
      input: { key: "session" },
    },
  ]);
});

test("typed object API hides bridge byte-array encoding", async () => {
  const restore = installCapabilityTransport({
    async call(request) {
      switch (request.operation) {
        case "put":
          assert.deepEqual(request.input.body, [1, 2, 3]);
          return { info: { key: request.input.key, size: 3 } };
        case "get":
          return {
            object: { key: request.input.key, size: 3, body: [1, 2, 3] },
          };
        case "head":
          return { info: { key: request.input.key, size: 3 } };
        case "list":
          return { objects: [{ key: "images/logo.bin", size: 3 }] };
        case "delete":
          return { deleted: true };
        default:
          throw new Error(`unexpected operation ${request.operation}`);
      }
    },
  });

  try {
    assert.deepEqual(
      await objects.put("images/logo.bin", Uint8Array.from([1, 2, 3])),
      { key: "images/logo.bin", size: 3 },
    );

    const object = await objects.get("images/logo.bin");
    assert.equal(object?.key, "images/logo.bin");
    assert.equal(object?.size, 3);
    assert.deepEqual([...object.body], [1, 2, 3]);

    assert.deepEqual(await objects.head("images/logo.bin"), {
      key: "images/logo.bin",
      size: 3,
    });
    assert.deepEqual(await objects.list("images/"), [
      { key: "images/logo.bin", size: 3 },
    ]);
    assert.equal(await objects.delete("images/logo.bin"), true);
  } finally {
    restore();
  }
});

test("typed clients reject malformed transport output", async () => {
  const restore = installCapabilityTransport({
    async call() {
      return { value: 42 };
    },
  });

  try {
    await assert.rejects(() => kv.get("bad"), CapabilityProtocolError);
  } finally {
    restore();
  }
});
