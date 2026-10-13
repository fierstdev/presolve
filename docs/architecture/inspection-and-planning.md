# Inspection and planning UX

Presolve separates four developer questions:

```text
presolve inspect
    What is this application?

presolve check
    Can this application run in this environment?

presolve plan
    What exact workload/provider selections would Presolve make here?

presolve build
    What immutable application release is this?
```

These operations share underlying models but have different dependency boundaries.

## Inspection boundary

With no positional argument, `presolve inspect` discovers and validates the nearest:

```text
presolve.toml
```

This source-inspection mode does not discover or require:

```text
presolve.env.toml
```

It renders the semantic Application Contract rather than merely echoing source TOML. The current human-readable view includes:

- application name, version, description, and contract version
- explicit workloads or the legacy implicit `main` Component workload
- capability requirements, optionality, and semantic feature requirements/preferences
- application-internal interfaces and relationships
- resource requirements
- outbound network policy

Environment supply, provider selection, and deployment bindings are intentionally absent. Those belong to `check` and `plan`.

## DeploymentPlan boundary

`presolve plan` is environment-dependent.

It performs the same Application Contract + Environment Specification resolution used by `presolve check`, but its successful output is centered on the exact `DeploymentPlan` rather than compatibility messaging.

The human-readable plan view includes:

- application and environment context
- the exact resolved `EnvironmentId`
- resolved workload names and execution kinds
- capability interface and exact selected provider version
- selected provider name and stable `ProviderId`
- negotiated semantic features
- the canonical semantic capability contract attached to the binding, when available
- unbound optional capabilities

For known semantic contracts, the plan view also exposes the contract's canonical operations, guarantees, and optional feature vocabulary. This metadata comes directly from the resolver's `CapabilityBinding`; the CLI does not reconstruct it.

A plan does **not** currently contain:

- workload placement
- service endpoints
- provider credentials
- provider realization adapter identity
- registry locations
- runtime process/container IDs
- infrastructure provisioning actions

`presolve plan` therefore does not display or infer those concepts.

When resolution fails, no `DeploymentPlan` exists. The command surfaces the same stable PS2xxx resolver diagnostics used by `presolve check` and exits unsuccessfully.

This preserves the distinction:

```text
check
    compatibility-oriented explanation

plan
    exact successful resolver output
```

## Structured output boundary

`inspect`, `check`, and `plan` support `--json`.

Machine output is an explicit CLI projection rather than direct serialization of internal Rust types. Every payload begins with:

```json
{
  "schema": "presolve.cli",
  "schema_version": "0.1",
  "kind": "..."
}
```

This keeps internal refactors from silently redefining the automation surface.

The three structured commands retain their semantic boundaries:

```text
inspect --json
    normalized application semantics
    no environment required

check --json
    compatibility status + structured resolver problems
    no full DeploymentPlan payload

plan --json
    exact symbolic DeploymentPlan when resolved
    structured resolver problems when rejected
```

Rejected `check` and `plan` operations emit their structured payload to stdout and then exit unsuccessfully. This allows CI and other automation to inspect stable PS2xxx problem fields without scraping human diagnostics.

`plan --json` does not fabricate a partial plan: rejected resolution uses `"plan": null`.

Human-readable output remains the default and is unchanged by this chunk.

See [CLI JSON output](../reference/cli-json.md) for the current schema.

## Immutable release inspection boundary

`presolve inspect` now has two environment-independent forms:

```text
presolve inspect
    inspect the nearest source Application Contract

presolve inspect path/to/application.presolved
    inspect one immutable application release
```

The release form does not require the original source tree. It reads the requested `.presolved` file, decodes it through `PresolveBundle::decode`, and therefore verifies manifest/artifact integrity before displaying state.

Release inspection exposes:

- bundle format version
- canonical application release digest
- encoded bundle size
- application metadata
- originating Application Contract version
- canonical capability/resource/network requirements
- workload execution kinds
- embedded Component artifact identity
- immutable OCI descriptor identity
- application-internal topology

The release digest is derived from the canonical bundle manifest. Component bytes participate transitively through their content digests; OCI workloads participate through immutable OCI descriptor identity.

Release inspection remains distinct from deployment planning:

```text
release inspection
    what immutable application release is this?

plan
    how does this environment satisfy the application?
```

Provider selections, credentials, realization adapters, endpoints, placement, and environment identity are therefore absent from release inspection.

## Chunk 20 result

Chunk 20 exposes existing Presolve state without creating a second operator model:

```text
20.1  application inspection
20.2  DeploymentPlan inspection
20.3  structured JSON output
20.4  immutable `.presolved` release inspection
20.5  UX checkpoint + docs
```

The final surface keeps human and machine representations aligned around the same semantic boundaries. Structured output is an explicit CLI projection, release inspection is environment-independent, and planning never fabricates state that the resolver did not produce.

The checkpoint is documented in [Chunk 20 checkpoint](../development/chunk-20-checkpoint.md).
