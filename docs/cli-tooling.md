# CLI tooling product views

L11-C activates only immutable L3 product projections:

```text
presolve inspect workspace-snapshot --schema presolve.workspace-snapshot --product <file> [--format human|json]
presolve inspect workspace-graph --schema presolve.workspace-graph --product <file> [--format human|json]
presolve graph workspace --schema presolve.workspace-graph --product <file> [--format human|json|dot]
presolve trace --schema presolve.build-trace --product <file> [--format human|json]
```

The product file is explicitly named and read once. It is negotiated through
L10 then strictly decoded by L11-B; it is never treated as source,
configuration, a project root, or a discovery target. JSON is the validated
canonical L3 document. Human and DOT output are deterministic projections.

`trace` reads one explicit build-trace product, strictly validates it through
L11-F, then renders canonical JSON or deterministic human text. It does not
compile, discover a project, inspect a build directory, or persist a trace.

All other inspect/graph views remain unsupported tooling errors (exit code 6).
`profile`, `benchmark`, `doctor`, semantic graph projection, and artifact graph
projection have no activated command adapter.
