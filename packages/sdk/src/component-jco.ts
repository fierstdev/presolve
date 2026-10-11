import {
  installComponentBindings,
  type ComponentCapabilityError,
  type ComponentCapabilityErrorCode,
  type ComponentCapabilityImports,
  type ComponentKeyValueImports,
  type ComponentObjectInfo,
  type ComponentObjectStoreImports,
  type ComponentReadSession,
  type ComponentResult,
  type ComponentTransportOptions,
} from "./component.js";

export interface JcoKeyValueImports {
  get(key: string): string | null | undefined;
  set(key: string, value: string): void;
  delete(key: string): boolean;
}

export interface JcoObjectStoreImports {
  get(key: string): ComponentReadSession | null | undefined;
  read(handle: bigint, maxBytes: bigint): Uint8Array;
  closeRead(handle: bigint): void;

  put(key: string, size: bigint): bigint;
  write(handle: bigint, chunk: Uint8Array): void;
  finishPut(handle: bigint): ComponentObjectInfo;
  abortPut(handle: bigint): void;

  head(key: string): ComponentObjectInfo | null | undefined;
  delete(key: string): boolean;
  listObjects(prefix: string): readonly ComponentObjectInfo[];
}

export interface JcoComponentCapabilityImports {
  readonly kv?: JcoKeyValueImports;
  readonly objects?: JcoObjectStoreImports;
}

function normalizeErrorCode(tag: string): ComponentCapabilityErrorCode | undefined {
  const normalized = tag
    .replaceAll("_", "-")
    .replace(/[A-Z]/g, (character) => `-${character.toLowerCase()}`);

  switch (normalized) {
    case "unavailable":
    case "invalid-key":
    case "invalid-body":
    case "invalid-handle":
    case "invalid-request":
    case "internal":
      return normalized;

    default:
      return undefined;
  }
}

function errorPayload(error: unknown): unknown {
  if (typeof error === "object" && error !== null && "payload" in error) {
    return (error as { readonly payload: unknown }).payload;
  }

  return error;
}

function errorMessage(error: unknown, payload: unknown): string {
  if (typeof payload === "string") {
    return payload;
  }

  if (typeof payload === "object" && payload !== null) {
    const record = payload as Record<string, unknown>;
    const detail = record.val ?? record.value;

    if (typeof detail === "string") {
      return detail;
    }
  }

  if (error instanceof Error) {
    return error.message;
  }

  return String(error);
}

function normalizeJcoError(error: unknown): ComponentCapabilityError {
  const payload = errorPayload(error);

  if (typeof payload === "object" && payload !== null) {
    const record = payload as Record<string, unknown>;

    if (typeof record.tag === "string") {
      const code = normalizeErrorCode(record.tag);

      if (code !== undefined) {
        return {
          code,
          message: errorMessage(error, payload),
        };
      }
    }
  }

  return {
    code: "internal",
    message: errorMessage(error, payload),
  };
}

function capture<T>(operation: () => T): ComponentResult<T> {
  try {
    return {
      tag: "ok",
      value: operation(),
    };
  } catch (error) {
    return {
      tag: "error",
      error: normalizeJcoError(error),
    };
  }
}

function adaptKeyValue(imports: JcoKeyValueImports): ComponentKeyValueImports {
  return {
    get(key) {
      return capture(() => imports.get(key) ?? null);
    },
    set(key, value) {
      return capture(() => imports.set(key, value));
    },
    delete(key) {
      return capture(() => imports.delete(key));
    },
  };
}

function adaptObjectStore(
  imports: JcoObjectStoreImports,
): ComponentObjectStoreImports {
  return {
    get(key) {
      return capture(() => imports.get(key) ?? null);
    },
    read(handle, maxBytes) {
      return capture(() => imports.read(handle, maxBytes));
    },
    closeRead(handle) {
      return capture(() => imports.closeRead(handle));
    },
    put(key, size) {
      return capture(() => imports.put(key, size));
    },
    write(handle, chunk) {
      return capture(() => imports.write(handle, chunk));
    },
    finishPut(handle) {
      return capture(() => imports.finishPut(handle));
    },
    abortPut(handle) {
      return capture(() => imports.abortPut(handle));
    },
    head(key) {
      return capture(() => imports.head(key) ?? null);
    },
    delete(key) {
      return capture(() => imports.delete(key));
    },
    listObjects(prefix) {
      return capture(() => imports.listObjects(prefix));
    },
  };
}

/**
 * Installs the Presolve Component transport over Jco/ComponentizeJS guest imports.
 *
 * Jco lifts WIT `result` errors as JavaScript exceptions. This adapter converts
 * those exceptions back into the normalized Component result form used by the
 * transport-neutral Presolve SDK.
 */
export function installJcoComponentBindings(
  imports: JcoComponentCapabilityImports,
  options: ComponentTransportOptions = {},
): () => void {
  const normalized: ComponentCapabilityImports = {
    ...(imports.kv === undefined ? {} : { kv: adaptKeyValue(imports.kv) }),
    ...(imports.objects === undefined
      ? {}
      : { objects: adaptObjectStore(imports.objects) }),
  };

  return installComponentBindings(normalized, options);
}
