/**
 * Raised when TypeScript application code invokes a capability before an
 * execution-target adapter has installed a transport.
 */
export class CapabilityTransportUnavailableError extends Error {
  constructor() {
    super(
      "no Presolve capability transport is installed for this execution environment",
    );
    this.name = "CapabilityTransportUnavailableError";
  }
}

/** Raised when an execution-target adapter cannot be configured safely. */
export class CapabilityConfigurationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "CapabilityConfigurationError";
  }
}

/**
 * Raised when a transport or typed client receives data that violates its
 * declared protocol.
 */
export class CapabilityProtocolError extends Error {
  readonly interface: string;
  readonly operation: string;

  constructor(interfaceName: string, operation: string, message: string) {
    super(`${interfaceName}.${operation}: ${message}`);
    this.name = "CapabilityProtocolError";
    this.interface = interfaceName;
    this.operation = operation;
  }
}

/** Raised when the selected capability provider rejects an invocation. */
export class CapabilityInvocationError extends Error {
  readonly interface: string;
  readonly operation: string;
  readonly code: string;

  constructor(
    interfaceName: string,
    operation: string,
    code: string,
    message: string,
  ) {
    super(`${interfaceName}.${operation} failed with \`${code}\`: ${message}`);
    this.name = "CapabilityInvocationError";
    this.interface = interfaceName;
    this.operation = operation;
    this.code = code;
  }
}
