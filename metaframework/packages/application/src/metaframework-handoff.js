function nonEmpty(value, field) {
  if (typeof value !== "string" || value.length === 0) throw new TypeError(`${field} must be a non-empty string`);
  return value;
}

/** Projects explicit source authority to compiler-owned route inspection. */
export function createRouteGraphInvocation(request) {
  if (request === null || typeof request !== "object" || Array.isArray(request)) throw new TypeError("request must be an object");
  if (!Array.isArray(request.sources) || request.sources.length === 0) throw new TypeError("sources must be a non-empty array");
  const args = ["route", "graph", "--config", nonEmpty(request.configurationPath, "configurationPath")];
  for (const source of request.sources) args.push("--source", nonEmpty(source, "source"));
  return Object.freeze({ executable: "presolve", arguments: Object.freeze(args) });
}

/** Projects explicit source authority to compiler-owned static request handoff. */
export function createStaticRequestInvocation(request) {
  const graph = createRouteGraphInvocation(request);
  return Object.freeze({ executable: graph.executable, arguments: Object.freeze(["route", "request", ...graph.arguments.slice(2)]) });
}

export function invokeMetaframework(request, createInvocation, execute) {
  if (typeof createInvocation !== "function" || typeof execute !== "function") throw new TypeError("createInvocation and execute must be functions");
  return execute(createInvocation(request));
}
