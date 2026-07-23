# Phase O O1 explicit application build handoff

**Status:** O1 implementation authority.

## Request

`createApplicationBuildInvocation(request)` accepts only this caller-owned
request shape:

```ts
{
  entryPath: "src/App.tsx",
  outputDirectory: "dist",
  production?: boolean,
  packageContracts?: { [specifier: string]: "contracts/pkg.json" },
  packageRuntimeModules?: { [specifier: string]: "./runtime/pkg.js" }
}
```

Every string is non-empty. Mapping keys are non-empty unique package
specifiers, values are non-empty caller-selected paths/locations, and mappings
are ordered lexically by specifier. The request is an API boundary, not a file
format: O1 does not read a project file or package contract.

## Exact projection

The request projects only to the existing single-entry artifact publisher:

```sh
presolve build <entryPath> --out <outputDirectory> \
  --package-contract <specifier>=<path> \
  --package-runtime <specifier>=<location> \
  --production
```

Package flags are emitted only when supplied. O1 does not verify that mappings
are sufficient or valid; the compiler owns that validation and its diagnostics.
`invokeApplicationBuild(request, execute)` passes the immutable invocation to a
caller-owned executor and returns its result unchanged.

## Boundaries

O1 intentionally supports the compiler's current single-entry build product
only. It does not synthesize a multi-source build, merge artifacts, discover an
entrypoint, resolve npm packages, read sources, or interpret build output.

## Evidence

`scripts/verify-o1-explicit-application-build-handoff.sh` proves immutable
argument projection, deterministic mapping order, malformed-request rejection,
opaque-result passthrough, and one actual canonical compiler build with an
explicit semantic-package Resource contract/runtime mapping.
