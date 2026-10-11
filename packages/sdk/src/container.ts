import {
  CapabilityConfigurationError,
  CapabilityInvocationError,
  CapabilityProtocolError,
} from "./errors.js";
import { installCapabilityTransport } from "./runtime.js";
import type { CapabilityCall, CapabilityTransport } from "./transport.js";

export const CAPABILITY_BRIDGE_PROTOCOL = "presolve-http-json/0.1";
export const CAPABILITY_ENDPOINT_ENV = "PRESOLVE_CAPABILITY_ENDPOINT";
export const CAPABILITY_TOKEN_ENV = "PRESOLVE_CAPABILITY_TOKEN";
export const CAPABILITY_PROTOCOL_ENV = "PRESOLVE_CAPABILITY_PROTOCOL";

export interface ContainerBridgeConfig {
  readonly endpoint: string;
  readonly token: string;
  readonly protocol: string;
}

interface BridgeSuccess {
  readonly status: "ok";
  readonly output: unknown;
}

interface BridgeFailure {
  readonly status: "error";
  readonly code: string;
  readonly message: string;
}

type BridgeResponse = BridgeSuccess | BridgeFailure;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function decodeBridgeResponse(value: unknown): BridgeResponse {
  if (!isRecord(value)) {
    throw new CapabilityProtocolError(
      "presolve:transport/http-json",
      "response",
      "expected JSON object response",
    );
  }

  if (value.status === "ok") {
    return {
      status: "ok",
      output: value.output,
    };
  }

  if (
    value.status === "error" &&
    typeof value.code === "string" &&
    typeof value.message === "string"
  ) {
    return {
      status: "error",
      code: value.code,
      message: value.message,
    };
  }

  throw new CapabilityProtocolError(
    "presolve:transport/http-json",
    "response",
    "malformed capability bridge response envelope",
  );
}

export class ContainerBridgeTransport implements CapabilityTransport {
  readonly #config: ContainerBridgeConfig;

  constructor(config: ContainerBridgeConfig) {
    if (config.protocol !== CAPABILITY_BRIDGE_PROTOCOL) {
      throw new CapabilityConfigurationError(
        `unsupported container capability protocol \`${config.protocol}\`; expected \`${CAPABILITY_BRIDGE_PROTOCOL}\``,
      );
    }

    this.#config = config;
  }

  async call<TOutput = unknown, TInput = unknown>(
    request: CapabilityCall<TInput>,
  ): Promise<TOutput> {
    const response = await fetch(this.#config.endpoint, {
      method: "POST",
      headers: {
        authorization: `Bearer ${this.#config.token}`,
        "content-type": "application/json",
      },
      body: JSON.stringify(request),
    });

    let decodedJson: unknown;

    try {
      decodedJson = await response.json();
    } catch (error) {
      throw new CapabilityProtocolError(
        request.interface,
        request.operation,
        `capability bridge returned invalid JSON: ${String(error)}`,
      );
    }

    const decoded = decodeBridgeResponse(decodedJson);

    if (decoded.status === "error") {
      throw new CapabilityInvocationError(
        request.interface,
        request.operation,
        decoded.code,
        decoded.message,
      );
    }

    if (!response.ok) {
      throw new CapabilityProtocolError(
        request.interface,
        request.operation,
        `capability bridge returned HTTP ${response.status} with an ok envelope`,
      );
    }

    return decoded.output as TOutput;
  }
}

export function containerBridgeConfigFromEnvironment(
  environment: Readonly<Record<string, string | undefined>> = processEnvironment(),
): ContainerBridgeConfig {
  const endpoint = environment[CAPABILITY_ENDPOINT_ENV];
  const token = environment[CAPABILITY_TOKEN_ENV];
  const protocol = environment[CAPABILITY_PROTOCOL_ENV];

  if (endpoint === undefined || endpoint.length === 0) {
    throw new CapabilityConfigurationError(
      `required environment variable \`${CAPABILITY_ENDPOINT_ENV}\` is not set`,
    );
  }

  if (token === undefined || token.length === 0) {
    throw new CapabilityConfigurationError(
      `required environment variable \`${CAPABILITY_TOKEN_ENV}\` is not set`,
    );
  }

  if (protocol === undefined || protocol.length === 0) {
    throw new CapabilityConfigurationError(
      `required environment variable \`${CAPABILITY_PROTOCOL_ENV}\` is not set`,
    );
  }

  return { endpoint, token, protocol };
}

/**
 * Installs the HTTP/JSON capability transport injected by a Presolve container
 * executor. Returns a restore function matching the low-level runtime API.
 */
export function installContainerBridge(
  config: ContainerBridgeConfig = containerBridgeConfigFromEnvironment(),
): () => void {
  return installCapabilityTransport(new ContainerBridgeTransport(config));
}

function processEnvironment(): Readonly<Record<string, string | undefined>> {
  const candidate = globalThis as typeof globalThis & {
    readonly process?: {
      readonly env?: Record<string, string | undefined>;
    };
  };

  return candidate.process?.env ?? {};
}
