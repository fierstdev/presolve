# Presolve

Presolve is a compiler-founded framework for TypeScript web applications. You
write components and conventional application files; the compiler publishes the
HTML, browser artifacts, route inventory, resumability records, and deployment
plan. There is no separate renderer, dependency tracker, or router deciding
application semantics beside it.

`0.1.0-alpha.5` is a public technical preview for evaluation and static sites.
It is not yet a replacement for every React or Next.js application.

## Create an application

```sh
pnpm create presolve my-app
cd my-app
pnpm install
pnpm dev
```

The generated application includes TypeScript 7, routes under `app/routes`, the
public `presolve` package, the `@presolve/cli` command, and a VS Code extension
recommendation. It needs no route registry or configuration file for the normal
workflow.

## Products

| Product | Package | What it provides |
| --- | --- | --- |
| Framework | `presolve` | Typed compiler intrinsics for components and application features. |
| Compiler and application CLI | `@presolve/cli` | Development, checks, builds, file routes, and deployment preparation. |
| Scaffold | `create-presolve` | The `pnpm create presolve` starter. |
| Editor | `presolve-vscode` | Workspace integration with the project TypeScript configuration. |
| Tooling APIs | `@presolve/compiler-wasm`, `@presolve/language-service`, `@presolve/lsp` | Compiler-product queries for editor integrations. |
| Rust crates | `presolve-parser`, `presolve-compiler`, `presolve-cli` | Embedding and toolchain integration. |

The alpha release train is lockstep: compatible published packages and crates
share the same prerelease version.

## Documentation

Start with the [introduction](docs/guide/introduction.md) and
[installation guide](docs/guide/installation.md). The complete guide and
reference cover components, state, [resumability](docs/guide/resumability.md),
routes, packages, VS Code, Cloudflare, and the maintainer
[publication runbook](docs/reference/publishing.md).

## Contributing

```sh
pnpm install
pnpm check
pnpm release:check
```

See [contributing](CONTRIBUTING.md), [security](SECURITY.md), and
[support](SUPPORT.md).
