# CLI JSON output

`presolve inspect`, `presolve check`, and `presolve plan` support versioned machine-readable output:

```bash
presolve inspect --json
presolve check --json
presolve plan --json
```

JSON mode writes JSON only to standard output. Human-oriented rendering is not mixed into stdout.

The current pre-stable schema identifies itself with:

```json
{
  "schema": "presolve.cli",
  "schema_version": "0.1"
}
```

An incompatible schema change requires a schema-version change. The overall Presolve CLI remains pre-stable until explicitly documented otherwise.

## Application inspection

`presolve inspect --json` is environment-independent. It reports normalized application semantics rather than raw TOML syntax. The implicit `main` Component workload is therefore represented explicitly with `"implicit": true`.

## Compatibility check

`presolve check --json` reports:

```json
{
  "schema": "presolve.cli",
  "schema_version": "0.1",
  "kind": "compatibility_check",
  "application": { "...": "..." },
  "environment": { "...": "..." },
  "status": "compatible",
  "problems": []
}
```

On rejected resolution, `status` is `"incompatible"` and `problems` contains structured resolver diagnostics. The command still exits unsuccessfully after writing the JSON payload.

## Deployment plan

`presolve plan --json` is the machine-readable view of the current symbolic resolver output.

Successful resolution contains `"status": "resolved"` and a `plan` object with:

- `environment_id`
- resolved `workloads`
- exact capability `bindings`
- selected `provider_id`, name, and exact version
- negotiated features
- canonical semantic capability contract metadata when available
- `unbound_optional_capabilities`

If resolution fails, no partial plan is fabricated:

```json
{
  "status": "rejected",
  "plan": null,
  "problems": [
    {
      "type": "missing_capability",
      "code": "PS2001",
      "interface": "presolve:kv/store",
      "version": "^0.1"
    }
  ]
}
```

The command still exits unsuccessfully.

## Resolver problem shapes

Current problem types are:

```text
unsupported_workload
missing_capability
missing_capability_features
insufficient_memory
insufficient_cpu
```

Every problem carries its stable `PS2xxx` diagnostic `code` plus type-specific machine fields.

Automation should branch on `type` and `code`, not human-readable stderr text.

## Output guarantees and non-guarantees

JSON mode does not change discovery, validation, resolution, provider selection, or exit-status semantics.

The JSON representation is an explicit CLI projection. It is not direct serialization of Rust implementation structs.

Object key ordering and whitespace are not semantic and should not be relied upon.

No new project configuration file is introduced by structured output.
