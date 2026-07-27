# Production audit contract

`presolve_compiler::production_audit` publishes the schema v1
`production-audit.json` artifact. It is the compiler-owned audit of the
already-frozen optimization and runtime-cost reports; Vite and release tooling
may verify and display it, but may not recreate its policy.

The audit accepts only matching schema-v1 reports for the same resume build,
with compiler validation recorded as `valid`, equal runtime-table counts, and
production-artifact bytes no greater than the complete production-byte total.
It records the authority and invariant counts, deterministic check names, and
the sole passing status `passed`. Failures use `PSV2A001` through `PSV2A005`.

Application publication and the conventional CLI build both emit this artifact
beside `production.runtime.json`, `optimization-report.json`, and
`runtime-cost-report.json`. `@presolve/vite` verifies the publication-manifest
digest, UTF-8 JSON shape, and passing schema before exposing the audit; it does
not evaluate budgets, infer validation, or modify the report.
