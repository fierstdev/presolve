# Presolve 0.1 alpha

`0.1.0-alpha.1` is Presolve's first public release train. The compiler,
framework, metaframework, CLI, language tooling, and VS Code extension move in
lockstep while their public compatibility contract is proven by real projects.

## Supported

- TypeScript 7.0-native decorators and TSX through the public `presolve` package.
- File routes in `app/routes` and a root layout in `app/layout.tsx`.
- Components, state/actions, computed values/effects, component composition,
  slots, Context, forms, resource declarations, and opaque package boundaries
  where the compiler admits the form.
- Compiler-published static HTML/runtime/resume artifacts and Cloudflare Workers
  Static Assets deployment preparation.
- macOS Apple Silicon and Intel, Linux x64, and Windows x64 CLI packages when
  published for the release tag.

## Explicitly not supported

- SSR, streaming, a generic JavaScript server runtime, or executable server
  loader/action handoffs.
- Automatic database, authentication, session, environment, or deployment
  resource provisioning.
- A guarantee that arbitrary TypeScript or arbitrary third-party packages are
  compiler-admitted. Use compiler-declared package contracts or `@opaque` for a
  deliberate non-semantic boundary.
- TypeScript 7.1 as a supported baseline. It is a compatibility target pending
  the public declaration and starter-project matrix.

The compiler fails closed for unsupported semantics. The framework does not
silently introduce a fallback reactive runtime.
