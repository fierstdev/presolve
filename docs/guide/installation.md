# Install Presolve

Use a current pnpm installation to create a new application:

```sh
pnpm create presolve@0.2.0-beta.27 my-app
cd my-app
pnpm install
pnpm dev
```

The explicit beta version avoids pnpm 11's one-day safety delay for newly
published unversioned packages. It guarantees that a new project uses the
release documented here even during the first day of publication.

The generated project already includes the public `presolve` authoring package,
the `@presolve/cli` command, TypeScript 7, project-local Vite, canonical
document/shell/style files, a mobile-first interactive starter route, a public
favicon, and a VS Code extension recommendation. It does not require a route
registry, a component name registry, a Vite configuration file, or a
`presolve.json` file for ordinary applications.

Open the project directory—not an individual source file—in VS Code. Install
the **Presolve** extension when prompted. The project-local TypeScript version
and generated `tsconfig.json` own standard TypeScript and TSX diagnostics.

## First checks

```sh
pnpm check
pnpm build
```

`check` validates application semantics without publishing a production build.
`build` writes the compiler-issued output to `dist/`. It publishes
`app/app.css` at both the `/app.css` compatibility path and an immutable
`/app.<sha256>.css` path linked from the document head. Route documents execute
their matching `runtime.<sha256>.js` while retaining `runtime.js` for
compatibility. It also copies `public/` to the root of `dist/`; every emitted
file is included in the deployment inventory. The former `styles/` directory
remains supported for beta compatibility and is copied to `dist/styles/`. Do
not edit generated output: run the compiler again after changing source.

Read [Project structure and ownership](project-structure.md) before moving
shared UI or document metadata. Read
[Styling, assets, Tailwind, and Vite](styling-and-vite.md) before adding a CSS
transform or custom Vite entry; the canonical `app/app.css` path and the
adapter-owned Vite path have deliberately different responsibilities.

For the first static deployment target, continue with
[Cloudflare deployment](../reference/cloudflare.md).
