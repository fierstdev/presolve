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
