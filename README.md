# Presolve

Presolve is a compiler-founded framework for TypeScript web applications. You
write components and conventional application files; the compiler publishes the
HTML, browser artifacts, route inventory, resumability records, and deployment
plan. There is no separate renderer, dependency tracker, or router deciding
application semantics beside it.

`0.2.0-beta.19` is the prepared beta for compiler-owned application products,
including resumability, structural components, slots, context, forms,
resources, and the documented Action surface. It is not a replacement for
every React or Next.js application: generic server execution and unadmitted
semantics remain deliberately unsupported.

## Create an application

```sh
pnpm create presolve my-app
cd my-app
pnpm install
pnpm dev
```

The generated application includes TypeScript 7, routes under `app/routes`, the
public `@presolve/framework` package installed under the canonical `presolve`
authoring alias, the `@presolve/cli` command, and a VS Code extension
recommendation. Its polished starter demonstrates the canonical document,
application shell, global CSS, public asset, route, and
interactive State/Action model. It needs no route registry, Vite configuration,
or Presolve configuration file for the normal workflow.

Layouts extend `Component` and use the slot field form:
`children: SlotContent = slot()`. During build, `app/app.css` is published as
`/app.css` and linked from the generated document head; `public/` is copied to
the root of `dist/`; both are integrity-listed for deployment.

## Products

| Product | Package | What it provides |
| --- | --- | --- |
| Framework | `@presolve/framework` (installed as `presolve`) | Typed compiler intrinsics for components and application features. |
| Compiler and application CLI | `@presolve/cli` | Development, checks, builds, file routes, and deployment preparation. |
| Scaffold | `create-presolve` | The `pnpm create presolve` starter. |
| Editor | `presolve-vscode` | Workspace integration with the project TypeScript configuration. |
| Tooling APIs | `@presolve/compiler-wasm`, `@presolve/language-service`, `@presolve/lsp` | Compiler-product queries for editor integrations. |
| Rust crates | `presolve-parser`, `presolve-compiler`, `presolve-cli` | Embedding and toolchain integration. |

The beta release train is lockstep: compatible published packages and crates
share the same prerelease version.

## Documentation

Start with the [introduction](docs/guide/introduction.md) and
[installation guide](docs/guide/installation.md). The complete guide and
reference cover [project ownership](docs/guide/project-structure.md),
[styling and Vite](docs/guide/styling-and-vite.md), components, state,
[resumability](docs/guide/resumability.md), routes, packages, VS Code,
Cloudflare, and the maintainer
[publication runbook](docs/reference/publishing.md).

## Contributing

```sh
pnpm install
pnpm check
pnpm release:check
```

See [contributing](CONTRIBUTING.md), [security](SECURITY.md), and
[support](SUPPORT.md).
