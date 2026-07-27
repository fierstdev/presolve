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
