import assert from "node:assert/strict";
import test from "node:test";

import {
  CapabilityInvocationError,
  kv,
  objects,
} from "../dist/index.js";
import {
  ComponentTransport,
  installComponentBindings,
} from "../dist/component.js";
import { installCapabilityTransport } from "../dist/runtime.js";

const ok = (value) => ({ tag: "ok", value });
const failure = (code, message) => ({
  tag: "error",
  error: { code, message },
});

test("Component projection maps typed KV calls to direct imports", async () => {
  const calls = [];
  const restore = installComponentBindings({
    kv: {
      get(key) {
        calls.push(["get", key]);
        return ok("stored");
      },
      set(key, value) {
        calls.push(["set", key, value]);
        return ok(undefined);
      },
      delete(key) {
        calls.push(["delete", key]);
        return ok(true);
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

test("Component object get consumes WIT read sessions and closes them", async () => {
  const calls = [];
  const chunks = [
    Uint8Array.from([1, 2]),
    Uint8Array.from([3, 4]),
  ];

  const restore = installComponentBindings(
    {
      objects: {
        get(key) {
          calls.push(["get", key]);
          return ok({
            handle: 7n,
            info: { key, size: 4n },
          });
        },
        read(handle, maxBytes) {
          calls.push(["read", handle, maxBytes]);
          return ok(chunks.shift() ?? new Uint8Array());
        },
        closeRead(handle) {
          calls.push(["closeRead", handle]);
          return ok(undefined);
        },
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
    const object = await objects.get("images/logo.bin");

    assert.equal(object?.key, "images/logo.bin");
    assert.equal(object?.size, 4);
    assert.deepEqual([...object.body], [1, 2, 3, 4]);
  } finally {
    restore();
  }

  assert.deepEqual(calls, [
    ["get", "images/logo.bin"],
    ["read", 7n, 2n],
    ["read", 7n, 2n],
    ["closeRead", 7n],
  ]);
});

test("Component object put streams chunks and finishes the handle", async () => {
  const calls = [];

  const restore = installComponentBindings(
    {
      objects: {
        get() {
          throw new Error("not used");
        },
        read() {
          throw new Error("not used");
        },
        closeRead() {
          throw new Error("not used");
        },
        put(key, size) {
          calls.push(["put", key, size]);
          return ok(9n);
        },
        write(handle, chunk) {
          calls.push(["write", handle, [...chunk]]);
          return ok(undefined);
        },
        finishPut(handle) {
          calls.push(["finishPut", handle]);
          return ok({ key: "asset.bin", size: 5n });
        },
        abortPut(handle) {
          calls.push(["abortPut", handle]);
          return ok(undefined);
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
    assert.deepEqual(
      await objects.put("asset.bin", Uint8Array.from([1, 2, 3, 4, 5])),
      { key: "asset.bin", size: 5 },
    );
  } finally {
    restore();
  }

  assert.deepEqual(calls, [
    ["put", "asset.bin", 5n],
    ["write", 9n, [1, 2]],
    ["write", 9n, [3, 4]],
    ["write", 9n, [5]],
    ["finishPut", 9n],
  ]);
});

test("Component object put aborts after a failed write", async () => {
  const calls = [];
  const transport = new ComponentTransport(
    {
      objects: {
        get() {
          throw new Error("not used");
        },
        read() {
          throw new Error("not used");
        },
        closeRead() {
          throw new Error("not used");
        },
        put() {
          return ok(11n);
        },
        write(handle) {
          calls.push(["write", handle]);
          return failure("invalid-body", "rejected");
        },
        finishPut() {
          throw new Error("must not finish");
        },
        abortPut(handle) {
          calls.push(["abortPut", handle]);
          return ok(undefined);
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
  const restore = installCapabilityTransport(transport);

  try {
    await assert.rejects(
      () => objects.put("bad.bin", Uint8Array.from([1, 2, 3])),
      (error) =>
        error instanceof CapabilityInvocationError &&
        error.code === "invalid-body",
    );
  } finally {
    restore();
  }

  assert.deepEqual(calls, [
    ["write", 11n],
    ["abortPut", 11n],
  ]);
});

test("Component result errors become typed capability invocation errors", async () => {
  const restore = installComponentBindings({
    kv: {
      get() {
        return failure("invalid-key", "empty key");
      },
      set() {
        return ok(undefined);
      },
      delete() {
        return ok(false);
      },
    },
  });

  try {
    await assert.rejects(
      () => kv.get(""),
      (error) =>
        error instanceof CapabilityInvocationError &&
        error.code === "invalid-key",
    );
  } finally {
    restore();
  }
});
