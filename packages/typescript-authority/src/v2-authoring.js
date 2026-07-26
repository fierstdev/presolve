import {
  analyzeTypeScriptProject,
  classifyResolvedComponentHeritage,
  classifyResolvedIntrinsic,
  createCanonicalIntrinsicRegistry,
} from "./index.js";

export const V2_AUTHORED_AUTHORITY_SCHEMA_VERSION = 2;

/**
 * Resolves explicit source positions for the implemented decorator-free V2
 * authoring forms. Syntax selection belongs to the caller; this bridge owns
 * only TypeScript resolution and canonical-registry classification.
 */
export async function analyzeV2Authoring(request) {
  validateV2AuthoringRequest(request);
  const queries = {
    symbols: [
      { id: "canonical:component", ...request.canonical.component },
      ...(request.canonical.state ? [{ id: "canonical:state", ...request.canonical.state }] : []),
      ...(request.canonical.action ? [{ id: "canonical:action", ...request.canonical.action }] : []),
      ...(request.canonical.effect ? [{ id: "canonical:effect", ...request.canonical.effect }] : []),
      ...request.states.map(site => ({ id: `state:${site.id}`, file: site.file, position: site.position })),
      ...request.actions.map(site => ({ id: `action:${site.id}`, file: site.file, position: site.position })),
      ...request.effects.map(site => ({ id: `effect:${site.id}`, file: site.file, position: site.position })),
    ],
    componentHeritage: request.components.map(site => ({
      id: `component:${site.id}`,
      file: site.file,
      position: site.position,
    })),
  };
  const authority = await analyzeTypeScriptProject({
    configFile: request.configFile,
    ...(request.cwd === undefined ? {} : { cwd: request.cwd }),
    queries,
  });
  const symbols = new Map(authority.symbols.map(entry => [entry.id, entry.symbol]));
  const registry = createCanonicalIntrinsicRegistry([
    { kind: "component", symbol: symbols.get("canonical:component") },
    ...(request.canonical.state ? [{ kind: "state", symbol: symbols.get("canonical:state") }] : []),
    ...(request.canonical.action ? [{ kind: "action", symbol: symbols.get("canonical:action") }] : []),
    ...(request.canonical.effect ? [{ kind: "effect", symbol: symbols.get("canonical:effect") }] : []),
  ]);
  return {
    schemaVersion: V2_AUTHORED_AUTHORITY_SCHEMA_VERSION,
    diagnostics: authority.diagnostics,
    components: authority.componentHeritage.flatMap(site => {
      const intrinsic = classifyResolvedComponentHeritage(registry, site.bases);
      return intrinsic ? [{ id: stripPrefix(site.id, "component:"), identity: intrinsic.identity }] : [];
    }),
    states: request.states.flatMap(site => {
      const intrinsic = classifyResolvedIntrinsic(registry, symbols.get(`state:${site.id}`));
      return intrinsic?.kind === "state" ? [{ id: site.id, identity: intrinsic.identity }] : [];
    }),
    actions: request.actions.flatMap(site => {
      const intrinsic = classifyResolvedIntrinsic(registry, symbols.get(`action:${site.id}`));
      return intrinsic?.kind === "action" ? [{ id: site.id, identity: intrinsic.identity }] : [];
    }),
    effects: request.effects.flatMap(site => {
      const intrinsic = classifyResolvedIntrinsic(registry, symbols.get(`effect:${site.id}`));
      return intrinsic?.kind === "effect" ? [{ id: site.id, identity: intrinsic.identity }] : [];
    }),
  };
}

function validateV2AuthoringRequest(request) {
  if (!request || typeof request !== "object" || typeof request.configFile !== "string") {
    throw new TypeError("V2 authoring authority requests require a configFile");
  }
  if (!request.canonical || typeof request.canonical !== "object") {
    throw new TypeError("V2 authoring authority requests require canonical framework positions");
  }
  validatePosition(request.canonical.component, "canonical component");
  if (request.schemaVersion !== V2_AUTHORED_AUTHORITY_SCHEMA_VERSION) {
    throw new TypeError(`unsupported V2 authoring authority schema version ${request.schemaVersion}`);
  }
  for (const kind of ["state", "action", "effect"]) {
    if (request.canonical[kind] !== undefined) {
      validatePosition(request.canonical[kind], `canonical ${kind}`);
    }
  }
  for (const [kind, sites] of [["component", request.components], ["state", request.states], ["action", request.actions], ["effect", request.effects]]) {
    if (!Array.isArray(sites)) throw new TypeError(`V2 authoring ${kind} sites must be an array`);
    if (kind !== "component" && sites.length > 0 && request.canonical[kind] === undefined) {
      throw new TypeError(`V2 authoring ${kind} sites require a canonical ${kind} position`);
    }
    const ids = new Set();
    for (const site of sites) {
      if (!site || typeof site.id !== "string" || !site.id || ids.has(site.id)) {
        throw new TypeError(`V2 authoring ${kind} sites require unique non-empty ids`);
      }
      ids.add(site.id);
      validatePosition(site, `${kind} site`);
    }
  }
}

function validatePosition(value, label) {
  if (!value || typeof value.file !== "string" || !Number.isInteger(value.position)) {
    throw new TypeError(`V2 authoring ${label} requires file and integer position`);
  }
}

function stripPrefix(value, prefix) {
  return value.startsWith(prefix) ? value.slice(prefix.length) : value;
}
