# Supported beta surface

Presolve 0.2 is a public beta. The compiler, framework, CLI,
metaframework, language tooling, VS Code extension, and generated artifacts
are released as a lockstep compatibility train.

## Supported

- TypeScript 7.0 projects created by `pnpm create presolve`.
- Compiler-admitted TypeScript/TSX components, state/actions, computed values,
  effects, slots, Context, forms, resources, and declared opaque package calls.
- Conventional file routes under `app/routes`, the canonical `app/app.tsx`
  application shell, `app/app.css` global stylesheet, and strict
  `app/index.html` compiler template. `app/layout.tsx` and `styles/` remain
  supported beta compatibility paths.
- Static production artifacts, resumability artifacts, and Cloudflare Workers
  Static Assets deployment preparation.
- macOS Apple Silicon and Intel, Linux x64, and Windows x64 CLI release
  packages when they are published for the selected beta release.

## Not supported

- SSR, streaming, or a generic application-JavaScript server runtime. The Node
  adapter executes only compiler-issued route-loader and Form server-action
  plans; Cloudflare Static Assets remains static-only.
- Automatic database, authentication, session, environment, or deployment
  provisioning.
- A promise that arbitrary TypeScript or arbitrary npm packages are semantically
  understood by the compiler.
- TypeScript 7.1 as the supported baseline until its release matrix is proven.

Unsupported semantic forms fail closed with compiler diagnostics. Presolve does
not switch to a general reactive runtime when a form cannot be lowered.

Read the [resumability guide](../guide/resumability.md) for the authoring,
artifact, deployment, and fallback contract.
