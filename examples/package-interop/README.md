# Package interoperability

This application proves Presolve's decorator-free terminal package invocation
boundary. `app/routes/index.tsx` imports a normal installed package. TypeScript
proves the exact named exports and call signatures, the compiler records each
Action use site and primitive argument codec, Vite bundles the browser
implementation, and the Presolve runtime invokes it once per admitted Action
event.

The synchronous example forwards string, number, boolean, and null values. The
Promise example proves runtime-owned `AbortSignal` injection,
replace-previous-per-component-instance cancellation, successful completion,
failure evidence, pagehide cleanup, and resume without replay.

Run `pnpm install`, `pnpm check`, and `pnpm build` from the repository root,
then `pnpm --dir examples/package-interop dev`.
