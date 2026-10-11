import { CapabilityProtocolError } from "./errors.js";
import { callCapability } from "./runtime.js";

export const OBJECT_STORE_INTERFACE = "presolve:objects/store";

export interface ObjectInfo {
  readonly key: string;
  readonly size: number;
}

export interface StoredObject extends ObjectInfo {
  readonly body: Uint8Array;
}

interface KeyInput {
  readonly key: string;
}

interface PutInput extends KeyInput {
  readonly body: readonly number[];
}

interface ListInput {
  readonly prefix: string;
}

interface ObjectValue {
  readonly key?: unknown;
  readonly size?: unknown;
  readonly body?: unknown;
}

interface GetOutput {
  readonly object?: unknown;
}

interface InfoOutput {
  readonly info?: unknown;
}

interface DeleteOutput {
  readonly deleted?: unknown;
}

interface ListOutput {
  readonly objects?: unknown;
}

function decodeObjectInfo(value: unknown, operation: string): ObjectInfo {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new CapabilityProtocolError(
      OBJECT_STORE_INTERFACE,
      operation,
      "expected object metadata",
    );
  }

  const candidate = value as ObjectValue;

  if (typeof candidate.key !== "string") {
    throw new CapabilityProtocolError(
      OBJECT_STORE_INTERFACE,
      operation,
      "expected object `key` to be string",
    );
  }

  if (
    typeof candidate.size !== "number" ||
    !Number.isSafeInteger(candidate.size) ||
    candidate.size < 0
  ) {
    throw new CapabilityProtocolError(
      OBJECT_STORE_INTERFACE,
      operation,
      "expected object `size` to be a non-negative safe integer",
    );
  }

  return {
    key: candidate.key,
    size: candidate.size,
  };
}

function decodeBody(value: unknown, operation: string): Uint8Array {
  if (
    !Array.isArray(value) ||
    value.some(
      (byte) =>
        typeof byte !== "number" ||
        !Number.isInteger(byte) ||
        byte < 0 ||
        byte > 255,
    )
  ) {
    throw new CapabilityProtocolError(
      OBJECT_STORE_INTERFACE,
      operation,
      "expected object `body` to be an array of bytes",
    );
  }

  return Uint8Array.from(value);
}

/** Reads one object and its body. */
export async function get(key: string): Promise<StoredObject | null> {
  const output = await callCapability<GetOutput, KeyInput>({
    interface: OBJECT_STORE_INTERFACE,
    operation: "get",
    input: { key },
  });

  if (output.object === null) {
    return null;
  }

  const info = decodeObjectInfo(output.object, "get");
  const body = decodeBody((output.object as ObjectValue).body, "get");

  if (body.byteLength !== info.size) {
    throw new CapabilityProtocolError(
      OBJECT_STORE_INTERFACE,
      "get",
      `object body length ${body.byteLength} does not match reported size ${info.size}`,
    );
  }

  return {
    ...info,
    body,
  };
}

/**
 * Stores one object body.
 *
 * Bridge v0.1 transports object bodies as byte arrays. The public SDK uses
 * Uint8Array so application code is not coupled to that transport encoding.
 */
export async function put(key: string, body: Uint8Array): Promise<ObjectInfo> {
  const output = await callCapability<InfoOutput, PutInput>({
    interface: OBJECT_STORE_INTERFACE,
    operation: "put",
    input: {
      key,
      body: Array.from(body),
    },
  });

  return decodeObjectInfo(output.info, "put");
}

/** Reads object metadata without fetching the body. */
export async function head(key: string): Promise<ObjectInfo | null> {
  const output = await callCapability<InfoOutput, KeyInput>({
    interface: OBJECT_STORE_INTERFACE,
    operation: "head",
    input: { key },
  });

  if (output.info === null) {
    return null;
  }

  return decodeObjectInfo(output.info, "head");
}

/** Deletes one object and reports whether it previously existed. */
export async function deleteObject(key: string): Promise<boolean> {
  const output = await callCapability<DeleteOutput, KeyInput>({
    interface: OBJECT_STORE_INTERFACE,
    operation: "delete",
    input: { key },
  });

  if (typeof output.deleted !== "boolean") {
    throw new CapabilityProtocolError(
      OBJECT_STORE_INTERFACE,
      "delete",
      "expected `deleted` to be boolean",
    );
  }

  return output.deleted;
}

/** Lists object metadata under `prefix`. */
export async function list(prefix = ""): Promise<readonly ObjectInfo[]> {
  const output = await callCapability<ListOutput, ListInput>({
    interface: OBJECT_STORE_INTERFACE,
    operation: "list",
    input: { prefix },
  });

  if (!Array.isArray(output.objects)) {
    throw new CapabilityProtocolError(
      OBJECT_STORE_INTERFACE,
      "list",
      "expected `objects` to be an array",
    );
  }

  return output.objects.map((value) => decodeObjectInfo(value, "list"));
}

export const objects = Object.freeze({
  get,
  put,
  head,
  delete: deleteObject,
  list,
});
