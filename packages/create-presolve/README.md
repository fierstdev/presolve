# create-presolve

Create a new Presolve application with pnpm:

```sh
pnpm create presolve@0.2.0-beta.27 my-app
cd my-app
pnpm install
pnpm dev
```

The explicit beta version is intentional: pnpm 11 protects unversioned installs
with a one-day minimum package age. After a release has matured, `pnpm create
presolve my-app` resolves the same `latest` creator.

Run the command without a directory in an interactive terminal to choose the
destination from a prompt. Use `--help` for the complete invocation and
`--version` to inspect the release train.

The generated project is a complete, accessible application rather than an
empty fixture. It includes:

- the canonical `app/index.html`, `app/app.tsx`, and `app/app.css` ownership
  model;
- a mobile-first global design baseline and interactive route;
- file routes, shared-component, server, asset, public, and test homes;
- a real public favicon and metadata;
- the public framework, compiler CLI, TypeScript 7, project-local Vite, and
  Cloudflare deployment tooling;
- a VS Code extension recommendation; and
- an application README explaining structure, styling, Vite, build products,
  and deployment.

The creator refuses to overwrite any existing path.
