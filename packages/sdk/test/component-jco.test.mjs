import assert from "node:assert/strict";
import test from "node:test";

import {
  CapabilityInvocationError,
  kv,
  objects,
} from "../dist/index.js";
import { installJcoComponentBindings } from "../dist/component-jco.js";

test("Jco Component adapter maps direct KV imports onto the SDK", async () => {
  const calls = [];
  const restore = installJcoComponentBindings({
    kv: {
      get(key) {
        calls.push(["get", key]);
        return "stored";
      },
      set(key, value) {
        calls.push(["set", key, value]);
      },
      delete(key) {
        calls.push(["delete", key]);
        return true;
      },
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
    ["set", "session", "abc"],
    ["get", "session"],
    ["delete", "session"],
  ]);
});

test("Jco undefined options normalize to Presolve null semantics", async () => {
  const restore = installJcoComponentBindings({
    kv: {
      get() {
        return undefined;
      },
      set() {},
      delete() {
        return false;
      },
    },
  });

  try {
    assert.equal(await kv.get("missing"), null);
  } finally {
    restore();
  }
});

test("Jco ComponentError payloads preserve capability error codes", async () => {
  const restore = installJcoComponentBindings({
    kv: {
      get() {
        const error = new Error("component call failed");
        error.payload = {
          tag: "invalid-key",
          val: "empty key",
        };
        throw error;
      },
      set() {},
      delete() {
        return false;
      },
    },
  });

  try {
    await assert.rejects(
      () => kv.get(""),
      (error) =>
        error instanceof CapabilityInvocationError &&
        error.code === "invalid-key" &&
        error.message.includes("empty key"),
    );
  } finally {
    restore();
  }
});

test("Jco object imports retain the existing chunked object SDK path", async () => {
  const chunks = [Uint8Array.from([1, 2]), Uint8Array.from([3])];

  const restore = installJcoComponentBindings(
    {
      objects: {
        get(key) {
          return {
            handle: 5n,
            info: { key, size: 3n },
          };
        },
        read() {
          return chunks.shift() ?? new Uint8Array();
        },
        closeRead() {},
        put() {
          throw new Error("not used");
        },
        write() {
          throw new Error("not used");
        },
        finishPut() {
          throw new Error("not used");
        },
        abortPut() {
          throw new Error("not used");
        },
        head() {
          throw new Error("not used");
        },
        delete() {
          throw new Error("not used");
        },
        listObjects() {
          throw new Error("not used");
        },
      },
    },
    { objectChunkSize: 2 },
  );

  try {
    const object = await objects.get("asset.bin");
    assert.deepEqual([...object.body], [1, 2, 3]);
  } finally {
    restore();
  }
});
