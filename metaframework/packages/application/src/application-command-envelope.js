import { createApplicationBuildInvocation } from "./application-build-handoff.js";
import {
  createApplicationWatchOnceInvocation,
  createApplicationWorkspaceInvocation,
} from "./workspace-development-handoff.js";

export const APPLICATION_COMMAND_SCHEMA_VERSION = 1;

export function createApplicationCommandInvocation(request) {
  if (request === null || typeof request !== "object" || Array.isArray(request)) {
    throw new TypeError("request must be an object");
  }
  if (request.schemaVersion !== APPLICATION_COMMAND_SCHEMA_VERSION) {
    throw new TypeError(`schemaVersion must be ${APPLICATION_COMMAND_SCHEMA_VERSION}`);
  }
  if (request.input === null || typeof request.input !== "object" || Array.isArray(request.input)) {
    throw new TypeError("input must be an object");
  }
  switch (request.command) {
    case "build": return createApplicationBuildInvocation(request.input);
    case "workspace": return createApplicationWorkspaceInvocation(request.input);
    case "watch-once": return createApplicationWatchOnceInvocation(request.input);
    default: throw new TypeError("command must be build, workspace, or watch-once");
  }
}

export function invokeApplicationCommand(request, execute) {
  if (typeof execute !== "function") {
    throw new TypeError("execute must be a function");
  }
  return execute(createApplicationCommandInvocation(request));
}
