# CLI tooling product views

L11-C activates only immutable L3 product projections:

```text
presolve inspect workspace-snapshot --schema presolve.workspace-snapshot --product <file> [--format human|json]
presolve inspect workspace-graph --schema presolve.workspace-graph --product <file> [--format human|json]
presolve graph workspace --schema presolve.workspace-graph --product <file> [--format human|json|dot]
```

The product file is explicitly named and read once. It is negotiated through
L10 then strictly decoded by L11-B; it is never treated as source,
configuration, a project root, or a discovery target. JSON is the validated
canonical L3 document. Human and DOT output are deterministic projections.

All other inspect/graph views remain unsupported tooling errors (exit code 6).
`trace`, `profile`, `benchmark`, `doctor`, semantic graph projection, and
artifact graph projection have no L11-C product adapter.
