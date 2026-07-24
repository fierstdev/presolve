# Presolve

Presolve is a compiler-founded framework for TypeScript web applications. You
author components, routes, and explicit capability boundaries; the compiler
publishes the HTML, browser runtime, route table, and deployment inventory.
There is no application renderer, dependency tracker, or router hidden beside
the compiler.

**Presolve 0.1.0-alpha.1** is a public technical preview. It is suitable for
evaluation and static documentation/marketing sites. It is not yet a general
purpose replacement for every React/Next application.

## Start a project

```sh
pnpm create presolve my-app
cd my-app
pnpm install
pnpm dev
```

The starter includes a TypeScript 7 project configuration, file routes, the
public `presolve` authoring package, a local `presolve` CLI, and a VS Code
extension recommendation. Open the directory in VS Code after installing the
**Presolve** extension; the workspace TypeScript project owns normal TypeScript
and TSX syntax diagnostics.

## What is in this repository

| Surface | Package / location | Purpose |
| --- | --- | --- |
| Compiler | `presolve-compiler`, `presolve-cli` | Canonical semantic analysis and artifact publication. |
| Framework | `presolve` | TypeScript authoring vocabulary; its decorators have no runtime authority. |
| Metaframework | `@presolve/application` | Compiler-owned project discovery, file routes, build, and deployment handoffs. |
| Tooling | `@presolve/compiler-wasm`, `@presolve/language-service`, `@presolve/lsp` | Compiler-product queries for editor integrations. |
| VS Code | `presolve-vscode` | Workspace integration over the public TypeScript project and compiler tooling. |
| Scaffold | `create-presolve` | The `pnpm create presolve` application starter. |
| Site example | `examples/presolve-site` | A reference documentation-site application for local compiler and deployment verification. |

The package train is intentionally lockstep during the alpha: compatible
compiler, framework, tooling, and CLI artifacts carry the same prerelease
version.

## Supported alpha workflow

```sh
pnpm check
pnpm build
pnpm deploy:prepare
pnpm deploy
```

Routes are inferred from `app/routes`; layouts are inferred from `app/layout`.
The current Cloudflare adapter deploys compiler-published static assets through
Workers Static Assets and validates the immutable artifact inventory before it
invokes Wrangler. It intentionally rejects server loader/action handoffs: a
server-capability executor is not bundled into the static adapter.

## Alpha scope

Presolve 0.1 supports compiler-admitted TypeScript/TSX components, state and
actions, computed values and effects, components and slots, Context, forms,
file routes, static production artifacts, resumability products, and Cloudflare
static deployment preparation.

It does **not** yet provide SSR, streaming, a generic server runtime, executable
server actions/loaders, database/auth/session abstractions, or automatic
deployment provisioning. Unsupported source is rejected with compiler
diagnostics rather than silently delegated to a second runtime.

Read the [alpha guide](docs/alpha.md), [framework guide](docs/framework.md),
[metaframework guide](docs/metaframework.md), [tooling guide](docs/tooling.md),
and [Cloudflare deployment guide](docs/deploy-cloudflare.md).

## Develop this repository

```sh
pnpm install
cargo test --workspace
pnpm test
pnpm release:check
```

The reference site example is a local dogfood application:

```sh
cd examples/presolve-site
pnpm check
pnpm build
pnpm deploy:prepare
```

See [contributing](CONTRIBUTING.md), [security](SECURITY.md), [support](SUPPORT.md),
and the [release guide](docs/releasing.md). Historical implementation records
remain under `docs/archive/` and `docs/specifications/`; they are not the 0.1
public product documentation.
