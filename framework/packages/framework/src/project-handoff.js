function nonEmptyString(value, field) {
  if (typeof value !== "string" || value.length === 0) {
    throw new TypeError(`${field} must be a non-empty string`);
  }
  return value;
}

/**
 * Creates the current compiler's exact single-source artifact-publication
 * invocation without reading or retaining either path.
 */
export function createArtifactBuildInvocation(request) {
  if (request === null || typeof request !== "object" || Array.isArray(request)) {
    throw new TypeError("request must be an object");
  }
  const sourcePath = nonEmptyString(request.sourcePath, "sourcePath");
  const outputDirectory = nonEmptyString(request.outputDirectory, "outputDirectory");
  const argumentsList = ["build", sourcePath, "--out", outputDirectory];
  if (request.production === true) {
    argumentsList.push("--production");
  } else if (request.production !== undefined && request.production !== false) {
    throw new TypeError("production must be a boolean when provided");
  }

  return Object.freeze({
    executable: "presolve",
    arguments: Object.freeze(argumentsList),
  });
}

/**
 * Passes one immutable canonical invocation to a caller-owned executor and
 * preserves its opaque result without decoding or status translation.
 */
export function invokeArtifactBuild(request, execute) {
  if (typeof execute !== "function") {
    throw new TypeError("execute must be a function");
  }
  return execute(createArtifactBuildInvocation(request));
}
