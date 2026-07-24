# Getting started

Create a new project with pnpm:

```sh
pnpm create presolve my-app
cd my-app
pnpm install
pnpm dev
```

The starter contains no route registry, source-file list, component name
strings, or build artifact configuration. `app/routes/index.tsx` is the home
route and `app/layout.tsx`, if present, is its inferred layout.

Open the project in VS Code and install the **Presolve** extension when
prompted. The generated `tsconfig.json` uses the project-local TypeScript 7
toolchain and `jsx: preserve`; normal TypeScript diagnostics stay visible.

Before publishing a static site:

```sh
pnpm check
pnpm build
pnpm deploy:prepare
```

`deploy:prepare` validates the compiler artifact digest inventory locally. Run
`pnpm deploy` only after signing in to Cloudflare with Wrangler.
