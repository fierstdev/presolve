# Chunk 20 checkpoint: inspection and planning UX

Chunk 20 turns Presolve's application, environment, resolver, and release models into explicit developer/operator inspection surfaces.

The command boundary is:

```text
presolve inspect
    What does this source application mean?

presolve inspect <release>.presolved
    What immutable application release is this?

presolve check
    Can this application run in this environment?

presolve plan
    What exact symbolic selections did the resolver make?

presolve build
    Construct the immutable application release.
```

These commands intentionally answer different questions rather than exposing one undifferentiated internal-state dump.

## Source inspection

`presolve inspect` discovers only the nearest `presolve.toml`.

It does not require an Environment Specification.

The view exposes normalized application semantics:

- application identity and description
- explicit or implicit workloads
- capability requirements
- required and preferred semantic features
- resource requirements
- network policy
- application-internal interfaces and relationships

The implicit legacy `main` Component workload is surfaced explicitly so inspection represents Presolve's semantic model rather than merely echoing source TOML.

## Compatibility checking

`presolve check` resolves the nearest Application Contract against the nearest Environment Specification.

Its concern is compatibility:

```text
compatible
or
incompatible + stable resolver diagnostics
```

It does not materialize providers or execute workloads.

## Deployment planning

`presolve plan` exposes the exact successful symbolic `DeploymentPlan`.

The current view includes:

- resolved environment identity
- resolved workloads
- selected provider name and stable `ProviderId`
- exact provider capability version
- negotiated semantic features
- attached canonical semantic capability contract metadata
- unbound optional capabilities

The command does not infer placement, endpoints, credentials, runtime instance IDs, provider adapter configuration, or infrastructure actions that do not exist in the symbolic plan.

When resolution fails, there is no partial plan.

## Structured output

`inspect`, `check`, and `plan` support versioned JSON projections under:

```json
{
  "schema": "presolve.cli",
  "schema_version": "0.1"
}
```

The JSON schema is an explicit CLI projection rather than direct serialization of internal Rust structs.

This distinction matters because it permits internal model refactoring without silently turning Rust implementation details into the automation contract.

Rejected:

```bash
presolve check --json
presolve plan --json
```

still return non-zero, but first emit structured resolver problems to stdout.

A rejected plan contains:

```json
{
  "status": "rejected",
  "plan": null,
  "problems": [
    {
      "type": "missing_capability",
      "code": "PS2001"
    }
  ]
}
```

Automation can therefore branch on stable problem fields rather than scrape human stderr.

## Immutable release inspection

Passing a `.presolved` path switches `inspect` from source semantics to immutable release semantics:

```bash
presolve inspect dist/application.presolved
presolve inspect dist/application.presolved --json
```

The release is decoded and verified before rendering.

Release inspection includes:

- bundle format version
- canonical application release digest
- bundle byte size
- application metadata
- originating contract version
- canonical requirements
- workload kinds
- embedded Component artifact identity
- immutable OCI descriptor identity
- application topology

It does not require the original source tree or an environment.

This preserves the application/release/environment split:

```text
source Application Contract
        ↓ build
immutable Application Release
        +
Environment Specification
        ↓ resolver
DeploymentPlan
```

## Checkpoint

Run:

```bash
./scripts/check-inspection-planning.sh
```

The checkpoint creates isolated temporary applications and proves:

- source inspection succeeds before any environment exists
- human and JSON source inspection agree on normalized semantics
- a compatible local environment produces successful `check`
- `plan` exposes the exact KV provider selected by the resolver
- human and JSON plan views agree on provider identity/version
- a built `.presolved` release can be inspected from a detached directory with no source/environment files
- release inspection exposes canonical release/artifact identity
- incompatible `check --json` returns non-zero but leaves valid structured PS2001 output
- rejected `plan --json` returns non-zero, contains `plan: null`, and leaves valid structured PS2001 output
- documentation invariants remain valid

The checkpoint does not require cloud credentials, Docker, a network connection, or persistent test configuration.

## Configuration boundary

Chunk 20 introduces no new Presolve project configuration file.

The durable root-level Presolve configuration surface remains:

```text
presolve.toml
presolve.env.toml
```

JSON output is a CLI representation. Release inspection consumes `.presolved` artifacts. Neither creates a second configuration authority.

## Result

Chunk 20 establishes a coherent inspect/check/plan workflow around the existing architecture:

> An application declares requirements. An environment declares supply. Presolve can show each side, deterministically resolve the two, explain failure, and expose the resulting symbolic plan in both human and machine-readable forms.

The next phase should prioritize external product proof rather than broadening the architecture.
