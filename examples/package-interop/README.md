# Package interoperability

This application proves Presolve's decorator-free terminal package invocation
boundary. `app/routes/index.tsx` imports a normal installed package. TypeScript
proves the exact named export, the compiler records the Action use site, Vite
bundles the browser implementation, and the Presolve runtime invokes it once
per admitted Action event.

Run `pnpm install`, `pnpm check`, and `pnpm build` from the repository root,
then `pnpm --dir examples/package-interop dev`.
