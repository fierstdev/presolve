function nonEmptyString(value, field) {
  if (typeof value !== "string" || value.length === 0) {
    throw new TypeError(`${field} must be a non-empty string`);
  }
  return value;
}

function sources(value) {
  if (!Array.isArray(value) || value.length === 0) {
    throw new TypeError("sources must be a non-empty array");
  }
  return value.map((source, index) => nonEmptyString(source, `sources[${index}]`));
}

function mappingEntries(value, field) {
  if (value === undefined) return [];
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
 * Projects one caller-owned complete application request to the canonical
 * multi-source compiler publication command. This module never reads sources,
 * derives an entry, parses manifests, or merges compiler artifacts.
 */
export function createApplicationPublicationInvocation(request) {
  if (request === null || typeof request !== "object" || Array.isArray(request)) {
    throw new TypeError("request must be an object");
  }
  const configurationPath = nonEmptyString(request.configurationPath, "configurationPath");
  const entryPath = nonEmptyString(request.entryPath, "entryPath");
  const outputDirectory = nonEmptyString(request.outputDirectory, "outputDirectory");
  const contracts = mappingEntries(request.packageContracts, "packageContracts");
  const runtimeModules = mappingEntries(request.packageRuntimeModules, "packageRuntimeModules");
  if (request.production !== undefined && request.production !== true && request.production !== false) {
    throw new TypeError("production must be a boolean when provided");
  }

  const argumentsList = ["application", "build", "--config", configurationPath];
  for (const source of sources(request.sources)) argumentsList.push("--source", source);
  argumentsList.push("--entry", entryPath, "--out", outputDirectory);
  for (const [specifier, contractPath] of contracts) {
    argumentsList.push("--package-contract", `${specifier}=${contractPath}`);
  }
  for (const [specifier, runtimeLocation] of runtimeModules) {
    argumentsList.push("--package-runtime", `${specifier}=${runtimeLocation}`);
  }
  if (request.production === true) argumentsList.push("--production");
  return Object.freeze({ executable: "presolve", arguments: Object.freeze(argumentsList) });
}

/** Preserves the caller executor and its result as the application boundary. */
export function invokeApplicationPublication(request, execute) {
  if (typeof execute !== "function") throw new TypeError("execute must be a function");
  return execute(createApplicationPublicationInvocation(request));
}
