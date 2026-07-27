# Vitest and Playwright integration contract

`@presolve/testing` provides `createPresolveVitestConfig` and
`createPresolvePlaywrightProject`. Both consume `@presolve/vite`'s existing
compiler-product plugin, making that plugin the sole publication-manifest and
artifact-digest authority. The testing package does not parse application
source, recreate diagnostics, or compare generated artifacts with a parallel
decoder.

The Vitest helper returns an immutable Vite plugin list, compiler snapshot
identity, and declared route fixtures. The Playwright helper returns the same
compiler-bound Vite metadata plus a validated caller-owned HTTP(S) origin.
It normalizes only the origin; route fixtures remain explicit application test
inputs. These records are integration configuration, not a test runner or
server implementation.

Applications still start development or preview servers through the existing
compiler/Vite command boundary. Vitest and Playwright run those caller-owned
tests, while compiler diagnostics and published artifact products remain the
authoritative evidence for semantic and resumability behavior.
