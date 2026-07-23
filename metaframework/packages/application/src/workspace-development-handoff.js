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

function format(value) {
  if (value === undefined) return undefined;
  if (value !== "human" && value !== "json") {
    throw new TypeError("format must be human or json when provided");
  }
  return value;
}

function sharedArguments(command, request) {
  if (request === null || typeof request !== "object" || Array.isArray(request)) {
    throw new TypeError("request must be an object");
  }
  const argumentsList = [command, "--config", nonEmptyString(request.configurationPath, "configurationPath")];
  for (const source of sources(request.sources)) argumentsList.push("--source", source);
  const selectedFormat = format(request.format);
  if (selectedFormat !== undefined) argumentsList.push("--format", selectedFormat);
  return argumentsList;
}

export function createApplicationWorkspaceInvocation(request) {
  const argumentsList = sharedArguments("workspace", request);
  if (request.verifyCleanEquivalence === true) {
    argumentsList.push("--verify-clean-equivalence");
  } else if (request.verifyCleanEquivalence !== undefined && request.verifyCleanEquivalence !== false) {
    throw new TypeError("verifyCleanEquivalence must be a boolean when provided");
  }
  return Object.freeze({ executable: "presolve", arguments: Object.freeze(argumentsList) });
}

export function createApplicationWatchOnceInvocation(request) {
  const argumentsList = ["watch", "--once", ...sharedArguments("workspace", request).slice(1)];
  if (request.verifyCleanEquivalence !== undefined) {
    throw new TypeError("verifyCleanEquivalence is only supported for workspace");
  }
  return Object.freeze({ executable: "presolve", arguments: Object.freeze(argumentsList) });
}

export function invokeApplicationDevelopment(request, createInvocation, execute) {
  if (typeof createInvocation !== "function" || typeof execute !== "function") {
    throw new TypeError("createInvocation and execute must be functions");
  }
  return execute(createInvocation(request));
}
