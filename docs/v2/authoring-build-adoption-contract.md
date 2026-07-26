# V2 authoring build-adoption contract

The V2 authoring frontend is not complete until a freshly generated,
decorator-free application is accepted by `presolve check`, `presolve build`,
route explain, and publication. The current legacy component graph is not an
acceptable fallback for these applications.

## Authority path

1. Project discovery supplies the exact application source set and project
   `tsconfig.json`.
2. The CLI invokes the installed `@presolve/typescript-authority` bridge with
   explicit syntax-site positions selected from the parser's source-faithful
   AST.
3. The bridge returns serialized resolved symbols and component base chains.
   The canonical intrinsic registry classifies only resolved framework targets.
4. The compiler converts those exact source joins into
   `V2AuthoringResolutionsV1`, then calls `lower_v2_authoring_v1`.
5. Downstream graph, route, template, publication, and runtime products adopt
   the resulting canonical model through a versioned adapter. They must not
   infer components, State, or Actions from decorators, names, import text, or
   raw heritage spelling.

The bridge is an installed CLI dependency, not a repository-relative script,
an ambient Node process, or a Vite plugin. A missing, incompatible, malformed,
or diagnostic-bearing authority result is a clear check/build failure. The
caller must never substitute the legacy decorator graph after such a failure.

## Adoption sequence

1. Add a versioned stdin/stdout authority bridge and a compiler-owned request
   schema with source path and UTF-16 offset validation. Implemented by the
   `presolve-typescript-authority` package executable; the Rust CLI adapter
   remains the next step.
2. Add the CLI adapter and prove a generated project has the declared TypeScript
   dependency/configuration available before the compiler begins publication.
3. Adopt resolved V2 components into the file-route graph while preserving
   existing stable IDs and route source ownership.
4. Adopt canonical State and Action records into the existing runtime products,
   with separate cold and resumed evidence.
5. Retire decorator fixtures from beta evidence; retain them only as named
   alpha compatibility coverage.

## Acceptance evidence

- `create-presolve` output, after installation, passes `presolve check` and
  `presolve build` without `@` syntax;
- a component imported under an alias and an indirect subclass are published;
- a lookalike base, unresolved inheritance, TypeScript diagnostic, missing
  authority bridge, and malformed bridge response all fail closed;
- canonical State and Action fields reach their existing publication/runtime
  products with cold and resumed browser evidence; and
- no legacy decorator lowering is invoked for the decorator-free fixture.

This contract was created after a direct generated-project probe returned
`PSAPP1005_ENTRY_APPLICATION_ROOT_MISSING`; that failure remains expected until
the above adapter is implemented and tested.
