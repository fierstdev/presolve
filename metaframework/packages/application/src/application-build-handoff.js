function nonEmptyString(value, field) {
  if (typeof value !== "string" || value.length === 0) {
    throw new TypeError(`${field} must be a non-empty string`);
  }
  return value;
}

function mappingEntries(value, field) {
  if (value === undefined) {
    return [];
  }
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${field} must be an object when provided`);
  }
  return Object.entries(value)
    .map(([specifier, location]) => [
      nonEmptyString(specifier, `${field} specifier`),
      nonEmptyString(location, `${field}[${specifier}]`),
    ])
    .sort(([left], [right]) => left.localeCompare(right));
}

/**
 * Projects caller-owned application build inputs to the exact current Presolve
 * single-entry CLI command. It deliberately does not read files, resolve npm
 * packages, or interpret compiler outputs.
 */
export function createApplicationBuildInvocation(request) {
  if (request === null || typeof request !== "object" || Array.isArray(request)) {
    throw new TypeError("request must be an object");
  }
  const entryPath = nonEmptyString(request.entryPath, "entryPath");
  const outputDirectory = nonEmptyString(request.outputDirectory, "outputDirectory");
  const contracts = mappingEntries(request.packageContracts, "packageContracts");
  const runtimeModules = mappingEntries(request.packageRuntimeModules, "packageRuntimeModules");
  if (request.production !== undefined && request.production !== true && request.production !== false) {
    throw new TypeError("production must be a boolean when provided");
  }

  const argumentsList = ["build", entryPath, "--out", outputDirectory];
  for (const [specifier, contractPath] of contracts) {
    argumentsList.push("--package-contract", `${specifier}=${contractPath}`);
  }
  for (const [specifier, runtimeLocation] of runtimeModules) {
    argumentsList.push("--package-runtime", `${specifier}=${runtimeLocation}`);
  }
  if (request.production === true) {
    argumentsList.push("--production");
  }
  return Object.freeze({
    executable: "presolve",
    arguments: Object.freeze(argumentsList),
  });
}

/** Preserves the caller executor and its result as the application boundary. */
export function invokeApplicationBuild(request, execute) {
  if (typeof execute !== "function") {
    throw new TypeError("execute must be a function");
  }
  return execute(createApplicationBuildInvocation(request));
}
