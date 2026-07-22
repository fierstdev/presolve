const projectCommands = new Set(["build", "check"]);

function nonEmptyString(value, field) {
  if (typeof value !== "string" || value.length === 0) {
    throw new TypeError(`${field} must be a non-empty string`);
  }
  return value;
}

function sourceArgument(source, index) {
  if (source === null || typeof source !== "object" || Array.isArray(source)) {
    throw new TypeError(`sources[${index}] must be an object`);
  }
  const logicalPath = nonEmptyString(source.logicalPath, `sources[${index}].logicalPath`);
  const relativePath = nonEmptyString(source.relativePath, `sources[${index}].relativePath`);
  return `${logicalPath}=${relativePath}`;
}

/**
 * Creates the exact caller-owned L9 build or check invocation without reading
 * or retaining configuration and source paths.
 */
export function createExplicitProjectInvocation(request) {
  if (request === null || typeof request !== "object" || Array.isArray(request)) {
    throw new TypeError("request must be an object");
  }
  const command = nonEmptyString(request.command, "command");
  if (!projectCommands.has(command)) {
    throw new RangeError("command must be build or check");
  }
  const configurationPath = nonEmptyString(request.configurationPath, "configurationPath");
  if (!Array.isArray(request.sources) || request.sources.length === 0) {
    throw new TypeError("sources must be a non-empty array");
  }

  const argumentsList = [command, "--config", configurationPath];
  request.sources.forEach((source, index) => {
    argumentsList.push("--source", sourceArgument(source, index));
  });
  argumentsList.push("--format", "json");

  return Object.freeze({
    executable: "presolve",
    arguments: Object.freeze(argumentsList),
  });
}

/**
 * Passes one immutable canonical invocation to a caller-owned executor and
 * preserves its opaque result without decoding or status translation.
 */
export function invokeExplicitProject(request, execute) {
  if (typeof execute !== "function") {
    throw new TypeError("execute must be a function");
  }
  return execute(createExplicitProjectInvocation(request));
}
