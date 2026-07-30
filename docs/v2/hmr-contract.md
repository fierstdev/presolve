# Presolve-aware HMR contract

`@presolve/vite` transports a compiler-authored `presolve.hmr-update` schema
v1 product. It does not inspect source text, TypeScript facts, compiler
artifacts, captures, state layout, or resume schemas to classify a change.

The compiler is the sole producer of an update product. Each product carries a
stable `updateId`, the publication `workspaceSnapshotId`, one of the eight V2
message classes, an ordered set of affected compiler module IDs, and explicit
state-compatibility evidence. The message classes are `template-update`,
`action-update`, `computed-update`, `style-update`, `server-only-update`,
`component-instance-reload`, `route-reload`, and `full-reload`.

`stateCompatibility` is either `proven-compatible` or `reload-required`.
The adapter rejects products whose `preserveState` value does not agree with
that evidence; it never promotes an update to state-preserving itself.

For `style-update`, the adapter returns Vite's original module set so native
CSS HMR remains in control. For `full-reload`, it sends Vite's native
`full-reload` transport message. Every other class is sent as a versioned
`presolve:hmr` custom event and suppresses Vite's module-level replacement.
The Presolve runtime consumes that event and applies the compiler-selected
reload scope.

The Vite `handleHotUpdate` hook only forwards Vite observation facts to a
required compiler callback and transports the callback's product. Returning no
product is an error, rather than permission for the adapter to infer a safe
fallback. This leaves CSS and other ordinary non-semantic asset behavior under
Vite while keeping semantic state preservation fail-closed.

## Canonical `presolve dev` behavior

The canonical CLI keeps one compiler-owned development publication live. It
watches authored project inputs while excluding `dist/`, `.presolve/`,
`node_modules/`, `target/`, Git data, and Presolve's atomic publication stages.
Every admitted edit runs the same development-profile compiler publication as
`presolve dev --once`; the server never patches generated HTML or reconstructs
route identity from filenames.

An edit whose complete changed set is CSS is a `style-update`. The browser
loads the rebuilt canonical stylesheet through
`/app.css?presolve-dev=<revision>`, waits for it to load, and then removes the
old link. The document, runtime, component state, focus, and scroll position
remain in place.

Every other successful edit uses the fail-closed `full-reload` boundary until
a narrower compiler HMR product proves state compatibility. The rebuilt
file-route manifest replaces the server's previous manifest before the browser
reloads, so adding, removing, or changing a route never leaves the request host
with a stale route graph.

If compilation fails, the last successful publication remains available. The
browser presents the compiler diagnostic in an accessible alert and reloads
only after the project compiles again. Development responses are always
`Cache-Control: no-store`; production builds retain their immutable,
content-addressed cache coordinates.
