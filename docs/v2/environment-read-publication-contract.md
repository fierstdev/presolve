# Environment-read publication contract

This contract is the artifact boundary following
[`environment-read-lowering-contract.md`](environment-read-lowering-contract.md).
It publishes only values that a caller has already admitted through
`EnvironmentInputManifestV1` and that source lowering has already joined to an
authority-proven V2 `environment.public` call.

## Product

`environment_publication` consumes one schema-v1 `EnvironmentReadLoweringV1`.
It rejects a lowering that contains diagnostics, then emits schema-v1 JSON with
one deterministic `browserValues` map. The map contains exactly the names read
by admitted source and their literal browser values. Repeated reads of the
same name collapse to one entry.

The product does not accept a dotenv path, a process environment, a Vite
environment object, or an arbitrary name/value map. Those would bypass the
source and manifest joins that establish browser authority.

## Failure boundary

Any source-lowering diagnostic prevents publication. In particular, dynamic,
server-owned, unprefixed, undeclared, or missing-manifest reads cannot yield a
partial browser artifact. Publication is therefore all-or-nothing for the
provided lowering product.

## Integration boundary

The file-route publication inventory accepts this immutable artifact as an
optional compiler input and emits it at `environment.browser.json`. Project-
level selection of a named environment manifest remains a separate contract.
Adapters may transport the artifact but may not add names or values to it.

## Vite runtime transport

When the compiler inventory contains `environment.browser.json`, `@presolve/vite`
may expose it only as the digest-verified virtual module
`virtual:presolve/v1/environment.browser.json`. The module validates the
schema-v1 artifact and exports a frozen `browserValues` map with exactly the
published public names. It never consults dotenv files, `process.env`, or
`import.meta.env`; malformed, server-owned, or digest-mismatched values fail
before Vite receives a module source.

This transport does not authorize Vite to rewrite authored
`environment.public` calls. A compiler-owned lowering must select and import
the virtual module before those source calls can execute in a browser bundle.
