import { createPresolveVitePlugin } from "@presolve/vite";

export const PRESOLVE_TEST_INTEGRATION_SCHEMA_VERSION = 1;

/** Compares caller-owned canonical byte sequences without interpretation. */
export function equalCanonicalBytes(expected, actual) {
  if (!(expected instanceof Uint8Array) || !(actual instanceof Uint8Array)) return false;
  if (expected.byteLength !== actual.byteLength) return false;
  return expected.every((value, index) => value === actual[index]);
}

/** Creates immutable metadata for an already-declared local test command. */
export function declaredTest({ name, command, lane }) {
  if (!name || !command || !lane) throw new TypeError("declared test requires name, command, and lane");
  return Object.freeze({ name, command, lane });
}

/**
 * Creates the Vitest-facing Vite configuration for an already-published
 * compiler product. The plugin remains the sole manifest validator; this
 * package neither parses source nor evaluates compiler diagnostics.
 */
export function createPresolveVitestConfig({ compilerProduct, readArtifact, fixtures = [] } = {}) {
  return createTestIntegration({
    runner: "vitest",
    compilerProduct,
    readArtifact,
    fixtures,
  });
}

/**
 * Creates Playwright project metadata for an already-published application.
 * The returned Vite plugin is for the caller's dev-server setup; Playwright
 * receives only a validated origin and compiler publication identity.
 */
export function createPresolvePlaywrightProject({
  compilerProduct,
  readArtifact,
  baseURL,
  fixtures = [],
} = {}) {
  const integration = createTestIntegration({
    runner: "playwright",
    compilerProduct,
    readArtifact,
    fixtures,
  });
  if (typeof baseURL !== "string" || !isHttpOrigin(baseURL)) {
    throw new TypeError("Presolve Playwright integration requires an absolute HTTP(S) origin baseURL");
  }
  return Object.freeze({
    ...integration,
    use: Object.freeze({ baseURL: new URL(baseURL).origin }),
  });
}

function createTestIntegration({ runner, compilerProduct, readArtifact, fixtures }) {
  if (!Array.isArray(fixtures)) {
    throw new TypeError("Presolve test integration fixtures must be an array");
  }
  const vitePlugin = createPresolveVitePlugin({ compilerProduct, readArtifact });
  return Object.freeze({
    schemaVersion: PRESOLVE_TEST_INTEGRATION_SCHEMA_VERSION,
    runner,
    compiler: vitePlugin.api,
    fixtures: Object.freeze(fixtures.map(normalizeFixture)),
    vite: Object.freeze({ plugins: Object.freeze([vitePlugin]) }),
  });
}

function normalizeFixture(fixture) {
  if (!fixture || typeof fixture !== "object" || Array.isArray(fixture)
    || typeof fixture.name !== "string" || !fixture.name
    || typeof fixture.route !== "string" || !fixture.route.startsWith("/")) {
    throw new TypeError("Presolve test fixtures require a non-empty name and absolute route");
  }
  return Object.freeze({ name: fixture.name, route: fixture.route });
}

function isHttpOrigin(value) {
  try {
    const url = new URL(value);
    return (url.protocol === "http:" || url.protocol === "https:")
      && url.pathname === "/" && !url.search && !url.hash;
  } catch {
    return false;
  }
}
