import { CapabilityProtocolError } from "./errors.js";
import { callCapability } from "./runtime.js";

export const KEY_VALUE_INTERFACE = "presolve:kv/store";

interface KeyInput {
  readonly key: string;
}

interface SetInput extends KeyInput {
  readonly value: string;
}

interface GetOutput {
  readonly value: unknown;
}

interface DeleteOutput {
  readonly deleted: unknown;
}

function decodeGet(output: GetOutput): string | null {
  if (output.value === null) {
    return null;
  }

  if (typeof output.value !== "string") {
    throw new CapabilityProtocolError(
      KEY_VALUE_INTERFACE,
      "get",
      "expected `value` to be string or null",
    );
  }

  return output.value;
}

function decodeDeleted(output: DeleteOutput): boolean {
  if (typeof output.deleted !== "boolean") {
    throw new CapabilityProtocolError(
      KEY_VALUE_INTERFACE,
      "delete",
      "expected `deleted` to be boolean",
    );
  }

  return output.deleted;
}

/** Reads one value from the environment-selected Presolve KV provider. */
export async function get(key: string): Promise<string | null> {
  const output = await callCapability<GetOutput, KeyInput>({
    interface: KEY_VALUE_INTERFACE,
    operation: "get",
    input: { key },
  });

  return decodeGet(output);
}

/** Stores one value in the environment-selected Presolve KV provider. */
export async function set(key: string, value: string): Promise<void> {
  await callCapability<Record<string, never>, SetInput>({
    interface: KEY_VALUE_INTERFACE,
    operation: "set",
    input: { key, value },
  });
}

/** Deletes one key and reports whether it previously existed. */
export async function deleteKey(key: string): Promise<boolean> {
  const output = await callCapability<DeleteOutput, KeyInput>({
    interface: KEY_VALUE_INTERFACE,
    operation: "delete",
    input: { key },
  });

  return decodeDeleted(output);
}

export const kv = Object.freeze({
  get,
  set,
  delete: deleteKey,
});
