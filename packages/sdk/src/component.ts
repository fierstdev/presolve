import {
  CapabilityConfigurationError,
  CapabilityInvocationError,
  CapabilityProtocolError,
} from "./errors.js";
import { installCapabilityTransport } from "./runtime.js";
import {
  KEY_VALUE_INTERFACE,
} from "./kv.js";
import {
  OBJECT_STORE_INTERFACE,
} from "./objects.js";
import type { CapabilityCall, CapabilityTransport } from "./transport.js";

export type ComponentCapabilityErrorCode =
  | "unavailable"
  | "invalid-key"
  | "invalid-body"
  | "invalid-handle"
  | "invalid-request"
  | "internal";

export interface ComponentCapabilityError {
  readonly code: ComponentCapabilityErrorCode;
  readonly message: string;
}

export type ComponentResult<T> =
  | {
      readonly tag: "ok";
      readonly value: T;
    }
  | {
      readonly tag: "error";
      readonly error: ComponentCapabilityError;
    };

export interface ComponentObjectInfo {
  readonly key: string;
  readonly size: bigint;
}

export interface ComponentReadSession {
  readonly handle: bigint;
  readonly info: ComponentObjectInfo;
}

/**
 * Normalized guest-side projection of `presolve:kv/store@0.1.0`.
 *
 * Build tooling may adapt generated Component bindings into this facade. The
 * application-facing SDK remains independent of any particular WIT generator.
 */
export interface ComponentKeyValueImports {
  get(key: string): ComponentResult<string | null>;
  set(key: string, value: string): ComponentResult<void>;
  delete(key: string): ComponentResult<boolean>;
}

/**
 * Normalized guest-side projection of `presolve:objects/store@0.1.0`.
 *
 * Object bodies follow the existing WIT handle/read/write protocol rather than
 * the container bridge's JSON byte-array representation.
 */
export interface ComponentObjectStoreImports {
  get(key: string): ComponentResult<ComponentReadSession | null>;
  read(handle: bigint, maxBytes: bigint): ComponentResult<Uint8Array>;
  closeRead(handle: bigint): ComponentResult<void>;

  put(key: string, size: bigint): ComponentResult<bigint>;
  write(handle: bigint, chunk: Uint8Array): ComponentResult<void>;
  finishPut(handle: bigint): ComponentResult<ComponentObjectInfo>;
  abortPut(handle: bigint): ComponentResult<void>;

  head(key: string): ComponentResult<ComponentObjectInfo | null>;
  delete(key: string): ComponentResult<boolean>;
  listObjects(prefix: string): ComponentResult<readonly ComponentObjectInfo[]>;
}

export interface ComponentCapabilityImports {
  readonly kv?: ComponentKeyValueImports;
  readonly objects?: ComponentObjectStoreImports;
}

export interface ComponentTransportOptions {
  /**
   * Maximum object body chunk passed to one WIT `read`/`write`.
   *
   * The default is 64 KiB.
   */
  readonly objectChunkSize?: number;
}

const DEFAULT_OBJECT_CHUNK_SIZE = 64 * 1024;

interface KvKeyInput {
  readonly key: string;
}

interface KvSetInput extends KvKeyInput {
  readonly value: string;
}

interface ObjectPutInput {
  readonly key: string;
  readonly body: readonly number[];
}


function unwrap<T>(
  interfaceName: string,
  operation: string,
  result: ComponentResult<T>,
): T {
  if (result.tag === "ok") {
    return result.value;
  }

  throw new CapabilityInvocationError(
    interfaceName,
    operation,
    result.error.code,
    result.error.message,
  );
}

function asRecord(
  interfaceName: string,
  operation: string,
  value: unknown,
): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new CapabilityProtocolError(
      interfaceName,
      operation,
      "expected operation input object",
    );
  }

  return value as Record<string, unknown>;
}

function stringField(
  interfaceName: string,
  operation: string,
  input: Record<string, unknown>,
  field: string,
): string {
  const value = input[field];

  if (typeof value !== "string") {
    throw new CapabilityProtocolError(
      interfaceName,
      operation,
      `expected input \`${field}\` to be string`,
    );
  }

  return value;
}

function byteArrayField(
  interfaceName: string,
  operation: string,
  input: Record<string, unknown>,
  field: string,
): Uint8Array {
  const value = input[field];

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
      interfaceName,
      operation,
      `expected input \`${field}\` to be an array of bytes`,
    );
  }

  return Uint8Array.from(value);
}

function safeSize(
  operation: string,
  size: bigint,
): number {
  if (size < 0n || size > BigInt(Number.MAX_SAFE_INTEGER)) {
    throw new CapabilityProtocolError(
      OBJECT_STORE_INTERFACE,
      operation,
      `object size ${size} cannot be represented safely in TypeScript`,
    );
  }

  return Number(size);
}

function objectInfo(
  operation: string,
  info: ComponentObjectInfo,
): { readonly key: string; readonly size: number } {
  return {
    key: info.key,
    size: safeSize(operation, info.size),
  };
}

function validateChunkSize(value: number | undefined): number {
  const chunkSize = value ?? DEFAULT_OBJECT_CHUNK_SIZE;

  if (!Number.isSafeInteger(chunkSize) || chunkSize <= 0) {
    throw new CapabilityConfigurationError(
      "Component object chunk size must be a positive safe integer",
    );
  }

  return chunkSize;
}

export class ComponentTransport implements CapabilityTransport {
  readonly #imports: ComponentCapabilityImports;
  readonly #objectChunkSize: number;

  constructor(
    imports: ComponentCapabilityImports,
    options: ComponentTransportOptions = {},
  ) {
    this.#imports = imports;
    this.#objectChunkSize = validateChunkSize(options.objectChunkSize);
  }

  async call<TOutput = unknown, TInput = unknown>(
    request: CapabilityCall<TInput>,
  ): Promise<TOutput> {
    let output: unknown;

    switch (request.interface) {
      case KEY_VALUE_INTERFACE:
        output = this.#callKv(request.operation, request.input);
        break;

      case OBJECT_STORE_INTERFACE:
        output = this.#callObjects(request.operation, request.input);
        break;

      default:
        throw new CapabilityConfigurationError(
          `no Component projection is installed for capability \`${request.interface}\``,
        );
    }

    return output as TOutput;
  }

  #kv(): ComponentKeyValueImports {
    const kv = this.#imports.kv;

    if (kv === undefined) {
      throw new CapabilityConfigurationError(
        `Component imports do not provide \`${KEY_VALUE_INTERFACE}\``,
      );
    }

    return kv;
  }

  #objects(): ComponentObjectStoreImports {
    const objects = this.#imports.objects;

    if (objects === undefined) {
      throw new CapabilityConfigurationError(
        `Component imports do not provide \`${OBJECT_STORE_INTERFACE}\``,
      );
    }

    return objects;
  }

  #callKv(operation: string, rawInput: unknown): unknown {
    const input = asRecord(KEY_VALUE_INTERFACE, operation, rawInput);
    const kv = this.#kv();

    switch (operation) {
      case "get": {
        const key = stringField(KEY_VALUE_INTERFACE, operation, input, "key");
        const value = unwrap(KEY_VALUE_INTERFACE, operation, kv.get(key));
        return { value };
      }

      case "set": {
        const key = stringField(KEY_VALUE_INTERFACE, operation, input, "key");
        const value = stringField(
          KEY_VALUE_INTERFACE,
          operation,
          input,
          "value",
        );
        unwrap(KEY_VALUE_INTERFACE, operation, kv.set(key, value));
        return {};
      }

      case "delete": {
        const key = stringField(KEY_VALUE_INTERFACE, operation, input, "key");
        const deleted = unwrap(
          KEY_VALUE_INTERFACE,
          operation,
          kv.delete(key),
        );
        return { deleted };
      }

      default:
        throw new CapabilityProtocolError(
          KEY_VALUE_INTERFACE,
          operation,
          "unsupported KV operation",
        );
    }
  }

  #callObjects(operation: string, rawInput: unknown): unknown {
    const input = asRecord(OBJECT_STORE_INTERFACE, operation, rawInput);
    const objects = this.#objects();

    switch (operation) {
      case "get":
        return this.#getObject(
          objects,
          stringField(OBJECT_STORE_INTERFACE, operation, input, "key"),
        );

      case "put":
        return this.#putObject(
          objects,
          stringField(OBJECT_STORE_INTERFACE, operation, input, "key"),
          byteArrayField(
            OBJECT_STORE_INTERFACE,
            operation,
            input,
            "body",
          ),
        );

      case "head": {
        const key = stringField(
          OBJECT_STORE_INTERFACE,
          operation,
          input,
          "key",
        );
        const info = unwrap(
          OBJECT_STORE_INTERFACE,
          operation,
          objects.head(key),
        );

        return {
          info: info === null ? null : objectInfo(operation, info),
        };
      }

      case "delete": {
        const key = stringField(
          OBJECT_STORE_INTERFACE,
          operation,
          input,
          "key",
        );
        const deleted = unwrap(
          OBJECT_STORE_INTERFACE,
          operation,
          objects.delete(key),
        );

        return { deleted };
      }

      case "list": {
        const prefix = stringField(
          OBJECT_STORE_INTERFACE,
          operation,
          input,
          "prefix",
        );
        const listed = unwrap(
          OBJECT_STORE_INTERFACE,
          operation,
          objects.listObjects(prefix),
        );

        return {
          objects: listed.map((info) => objectInfo(operation, info)),
        };
      }

      default:
        throw new CapabilityProtocolError(
          OBJECT_STORE_INTERFACE,
          operation,
          "unsupported object-storage operation",
        );
    }
  }

  #getObject(
    objects: ComponentObjectStoreImports,
    key: string,
  ): unknown {
    const session = unwrap(
      OBJECT_STORE_INTERFACE,
      "get",
      objects.get(key),
    );

    if (session === null) {
      return { object: null };
    }

    const expectedSize = safeSize("get", session.info.size);
    const chunks: Uint8Array[] = [];
    let received = 0;
    let primaryError: unknown;

    try {
      while (received < expectedSize) {
        const remaining = expectedSize - received;
        const maxBytes = Math.min(this.#objectChunkSize, remaining);
        const chunk = unwrap(
          OBJECT_STORE_INTERFACE,
          "get",
          objects.read(session.handle, BigInt(maxBytes)),
        );

        if (chunk.byteLength === 0) {
          throw new CapabilityProtocolError(
            OBJECT_STORE_INTERFACE,
            "get",
            "Component object stream ended before the reported object size",
          );
        }

        if (chunk.byteLength > remaining) {
          throw new CapabilityProtocolError(
            OBJECT_STORE_INTERFACE,
            "get",
            "Component object stream exceeded the reported object size",
          );
        }

        chunks.push(chunk);
        received += chunk.byteLength;
      }
    } catch (error) {
      primaryError = error;
    }

    try {
      unwrap(
        OBJECT_STORE_INTERFACE,
        "get",
        objects.closeRead(session.handle),
      );
    } catch (closeError) {
      if (primaryError === undefined) {
        primaryError = closeError;
      }
    }

    if (primaryError !== undefined) {
      throw primaryError;
    }

    const body = new Uint8Array(expectedSize);
    let offset = 0;

    for (const chunk of chunks) {
      body.set(chunk, offset);
      offset += chunk.byteLength;
    }

    return {
      object: {
        ...objectInfo("get", session.info),
        body: Array.from(body),
      },
    };
  }

  #putObject(
    objects: ComponentObjectStoreImports,
    key: string,
    body: Uint8Array,
  ): unknown {
    const handle = unwrap(
      OBJECT_STORE_INTERFACE,
      "put",
      objects.put(key, BigInt(body.byteLength)),
    );

    let finished = false;

    try {
      for (let offset = 0; offset < body.byteLength; offset += this.#objectChunkSize) {
        const chunk = body.slice(
          offset,
          Math.min(offset + this.#objectChunkSize, body.byteLength),
        );

        unwrap(
          OBJECT_STORE_INTERFACE,
          "put",
          objects.write(handle, chunk),
        );
      }

      const info = unwrap(
        OBJECT_STORE_INTERFACE,
        "put",
        objects.finishPut(handle),
      );
      finished = true;

      return {
        info: objectInfo("put", info),
      };
    } catch (error) {
      if (!finished) {
        try {
          unwrap(
            OBJECT_STORE_INTERFACE,
            "put",
            objects.abortPut(handle),
          );
        } catch {
          // Preserve the original put/write/finish failure.
        }
      }

      throw error;
    }
  }
}

/**
 * Installs a direct Component/WIT transport over normalized guest imports.
 *
 * A TypeScript→Component build tool only needs to adapt its generated bindings
 * to `ComponentCapabilityImports`; application-facing `kv`/`objects` code does
 * not change.
 */
export function installComponentBindings(
  imports: ComponentCapabilityImports,
  options: ComponentTransportOptions = {},
): () => void {
  return installCapabilityTransport(
    new ComponentTransport(imports, options),
  );
}
