# V2 scaffold contract

`create-presolve` establishes the beta conventional layout: `app/app.tsx`,
`app/app.css`, `app/index.html`, `app/routes`, `app/components`, `server`,
`assets`, `public`, and `tests`. Only `app/routes` is default route input; the
other directories declare document, composition, presentation, server, test,
or adapter ownership without creating parallel compiler discovery.

The starter demonstrates the ownership contract rather than merely creating
empty files: the application shell projects a default slot without owning a
`main` landmark, the route owns one primary `main`, the home route proves
decorator-free State and Action interactivity, the global stylesheet is
mobile-first and accessible, the document includes viewport and favicon
metadata around the required placeholders, and the favicon is a real public
asset.

The scaffold includes `.env.example` with a `PRESOLVE_PUBLIC_*` value and
negates that file from the general `.env*` ignore rule. It documents that
unprefixed environment values are server-owned. It retains the compiler-owned
`presolve dev`, build, and deployment commands. Project Vite is a bounded
physical bundler for compiler-admitted external browser entries; `assets/` and
`vite.config.ts` are not implicit semantic or publication inputs.
The scaffold must remain installable and pass the existing no-configuration
ergonomic check, build, and deployment preparation fixtures. Its decorator-free
source is accepted only through the authority bridge defined in
`authoring-build-adoption-contract.md`; the current direct build probe remains
an explicitly tracked adoption gap rather than evidence of scaffold readiness.
