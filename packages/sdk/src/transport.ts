/**
 * One semantic capability invocation.
 *
 * Transport is deliberately absent from this type. The same call may later be
 * projected through Component imports, the container bridge, IPC, or another
 * compatible binding.
 */
export interface CapabilityCall<TInput = unknown> {
  readonly interface: string;
  readonly operation: string;
  readonly input: TInput;
}

/**
 * Runtime transport capable of invoking Presolve capability semantics.
 */
export interface CapabilityTransport {
  call<TOutput = unknown, TInput = unknown>(
    request: CapabilityCall<TInput>,
  ): Promise<TOutput>;
}
