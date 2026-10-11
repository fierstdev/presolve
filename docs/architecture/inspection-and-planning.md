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

`presolve inspect` is application-only.

It discovers and validates the nearest:

```text
presolve.toml
```

It does not discover or require:

```text
presolve.env.toml
```

Inspection renders the semantic Application Contract rather than merely echoing source TOML. The current human-readable view includes:

- application name, version, description, and contract version
- explicit workloads or the legacy implicit `main` Component workload
- capability requirements, optionality, and semantic feature requirements/preferences
- application-internal interfaces and relationships
- resource requirements
- outbound network policy

Environment supply, provider selection, and deployment bindings are intentionally absent. Those belong to `check` and `plan`.

## Chunk 20 direction

Chunk 20 incrementally exposes existing Presolve state rather than creating a second operator model:

```text
20.1  application inspection
20.2  DeploymentPlan inspection
20.3  structured JSON output
20.4  immutable `.presolved` release inspection
20.5  UX checkpoint + docs
```

Structured output will be added after the human-facing semantic boundaries are settled so machine-readable representations reflect the same model rather than driving it prematurely.
