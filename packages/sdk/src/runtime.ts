import { CapabilityTransportUnavailableError } from "./errors.js";
import type { CapabilityCall, CapabilityTransport } from "./transport.js";

let activeTransport: CapabilityTransport | undefined;

/**
 * Installs the capability transport for the current execution environment.
 *
 * Application code normally does not call this directly. Presolve execution
 * adapters install the appropriate transport during workload bootstrap.
 *
 * The returned function restores the previously installed transport, which is
 * useful for deterministic tests and nested runtime scopes.
 */
export function installCapabilityTransport(
  transport: CapabilityTransport,
): () => void {
  const previous = activeTransport;
  activeTransport = transport;

  return () => {
    activeTransport = previous;
  };
}

/**
 * Invokes one semantic capability operation through the active transport.
 *
 * Typed capability clients such as KV/object storage are layered on top of this
 * function. It deliberately knows nothing about HTTP, WIT, Docker, or provider
 * implementations.
 */
export async function callCapability<TOutput = unknown, TInput = unknown>(
  request: CapabilityCall<TInput>,
): Promise<TOutput> {
  const transport = activeTransport;

  if (transport === undefined) {
    throw new CapabilityTransportUnavailableError();
  }

  return transport.call<TOutput, TInput>(request);
}
