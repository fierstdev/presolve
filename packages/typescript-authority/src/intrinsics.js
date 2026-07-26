export const CANONICAL_INTRINSIC_KINDS = Object.freeze([
  "component", "state", "action", "computed", "effect", "slot", "context",
  "provide", "consume", "form", "serialize", "field", "validate", "submit",
  "resource", "loader", "server_action", "opaque",
]);

/** Builds a registry exclusively from TypeScript-resolved canonical targets. */
export function createCanonicalIntrinsicRegistry(entries) {
  const registry = new Map();
  for (const entry of entries) {
    if (!CANONICAL_INTRINSIC_KINDS.includes(entry.kind)) {
      throw new TypeError(`unknown Presolve intrinsic kind ${entry.kind}`);
    }
    const identity = targetIdentity(entry.symbol);
    if (!identity) throw new TypeError(`intrinsic ${entry.kind} has no resolved target identity`);
    const key = identityKey(identity);
    if (registry.has(key)) throw new TypeError(`duplicate canonical intrinsic identity for ${entry.kind}`);
    registry.set(key, { kind: entry.kind, identity });
  }
  return registry;
}

/** Classifies a use-site result from `analyzeTypeScriptProject`, never its spelling. */
export function classifyResolvedIntrinsic(registry, symbol) {
  const identity = targetIdentity(symbol);
  return identity ? registry.get(identityKey(identity)) : undefined;
}

/**
 * Classifies a resolved base-class chain without relying on the local spelling
 * of a component base. The TypeScript adapter supplies the chain, including
 * aliases and indirect bases; the registry supplies framework meaning.
 */
export function classifyResolvedComponentHeritage(registry, bases) {
  for (const base of bases) {
    const intrinsic = classifyResolvedIntrinsic(registry, base);
    if (intrinsic?.kind === "component") return intrinsic;
  }
  return undefined;
}

function targetIdentity(symbol) {
  return symbol?.aliasTarget?.identity ?? symbol?.identity;
}

function identityKey(identity) {
  if (!identity || !Array.isArray(identity.declarationModules)) return "";
  return JSON.stringify([identity.name, identity.flags, [...identity.declarationModules].sort()]);
}
