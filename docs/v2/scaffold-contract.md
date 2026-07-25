# V2 scaffold contract

`create-presolve` establishes the beta conventional layout: `app/routes`,
`app/components`, `server`, `styles`, `assets`, `public`, and `tests`. Only
`app/routes` is default route input; the other directories declare ownership
and Vite-facing inputs without creating parallel compiler discovery.

The scaffold includes `.env.example` with a `PRESOLVE_PUBLIC_*` value and
documents that unprefixed environment values are server-owned. It intentionally
retains the current compiler-owned `presolve dev`, build, and deployment
commands until the Vite development-command adapter is a complete product.
The scaffold must remain installable and pass the existing no-configuration
ergonomic check, build, and deployment preparation fixtures.
