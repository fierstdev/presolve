import assert from "node:assert/strict";
import http from "node:http";
import test from "node:test";

import { CapabilityInvocationError, kv } from "../dist/index.js";
import {
  CAPABILITY_BRIDGE_PROTOCOL,
  CAPABILITY_ENDPOINT_ENV,
  CAPABILITY_PROTOCOL_ENV,
  CAPABILITY_TOKEN_ENV,
  ContainerBridgeTransport,
  containerBridgeConfigFromEnvironment,
  installContainerBridge,
} from "../dist/container.js";
import { installCapabilityTransport } from "../dist/runtime.js";

function listen(server) {
  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      server.off("error", reject);
      resolve();
    });
  });
}

function close(server) {
  return new Promise((resolve, reject) => {
    server.close((error) => {
      if (error) reject(error);
      else resolve();
    });
  });
}

test("container bridge adapter matches Rust HTTP/JSON envelope", async () => {
  const requests = [];
  const server = http.createServer(async (request, response) => {
    const chunks = [];
    for await (const chunk of request) chunks.push(chunk);

    requests.push({
      method: request.method,
      url: request.url,
      authorization: request.headers.authorization,
      body: JSON.parse(Buffer.concat(chunks).toString("utf8")),
    });

    response.writeHead(200, { "content-type": "application/json" });
    response.end(
      JSON.stringify({
        status: "ok",
        output: { value: "from-bridge" },
      }),
    );
  });

  await listen(server);

  try {
    const address = server.address();
    assert.notEqual(address, null);
    assert.equal(typeof address, "object");

    const restore = installCapabilityTransport(
      new ContainerBridgeTransport({
        endpoint: `http://127.0.0.1:${address.port}/v1/call`,
        token: "test-token",
        protocol: CAPABILITY_BRIDGE_PROTOCOL,
      }),
    );

    try {
      assert.equal(await kv.get("example"), "from-bridge");
    } finally {
      restore();
    }
  } finally {
    await close(server);
  }

  assert.deepEqual(requests, [
    {
      method: "POST",
      url: "/v1/call",
      authorization: "Bearer test-token",
      body: {
        interface: "presolve:kv/store",
        operation: "get",
        input: { key: "example" },
      },
    },
  ]);
});

test("container bridge maps remote error envelopes", async () => {
  const server = http.createServer((_request, response) => {
    response.writeHead(200, { "content-type": "application/json" });
    response.end(
      JSON.stringify({
        status: "error",
        code: "invalid-key",
        message: "empty key",
      }),
    );
  });

  await listen(server);

  try {
    const address = server.address();
    assert.notEqual(address, null);
    assert.equal(typeof address, "object");

    const restore = installContainerBridge({
      endpoint: `http://127.0.0.1:${address.port}/v1/call`,
      token: "test-token",
      protocol: CAPABILITY_BRIDGE_PROTOCOL,
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
  } finally {
    await close(server);
  }
});

test("container bridge reads Presolve discovery variables", () => {
  assert.deepEqual(
    containerBridgeConfigFromEnvironment({
      [CAPABILITY_ENDPOINT_ENV]: "http://host.docker.internal:4000/v1/call",
      [CAPABILITY_TOKEN_ENV]: "secret",
      [CAPABILITY_PROTOCOL_ENV]: CAPABILITY_BRIDGE_PROTOCOL,
    }),
    {
      endpoint: "http://host.docker.internal:4000/v1/call",
      token: "secret",
      protocol: CAPABILITY_BRIDGE_PROTOCOL,
    },
  );
});
