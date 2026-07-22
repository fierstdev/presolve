# Phase N N0 semantic capability registry contract

**Status:** N0 implementation authority.

## Product

The compiler publishes one versioned semantic capability registry. It reports
the compiler's current admission decision; it does not parse application source,
change lowering, or create a runtime feature.

```sh
presolve explain --capabilities --format json
```

The product has schema version `1`. Its ordered records include an ID, class,
admission status, source form, semantic owner, type/dependency/resume rules,
artifact impact, proof fixture, and rejection reason for every deferred family.

## Admission contract

A capability can move from `deferred` to `admitted` only when its implementation
has every product relevant to that capability: source normalization, semantic
identity, type/boundary rules, dependency and lifecycle analysis, IR,
artifact/runtime/resume policy, canonical diagnostics, and the required
fixtures. A non-executable binding may end at the canonical binding-table
product; an executable capability must satisfy the full lowering path. N0
records current compiler-native and bounded families without changing any of
those existing products.

`semantic_package_exports`, `module_types`, `resources`, and `opaque_typescript`
are deliberately deferred records. N1-A admits only an integrity-checked
third-party **binding**; invoking an imported package export remains unsupported
until N1-A2 supplies its explicit executable semantic contract.

## Boundaries

The registry is an inspection product, not a source-level escape hatch or a
framework feature flag. Its records do not permit arbitrary TypeScript, package
installation, package source inspection, runtime reflection, or fallback
execution. Existing ASM, artifact, runtime, resume, and framework schema
versions remain unchanged in N0.

## Evidence

The focused N0 verifier checks the contract, registry ordering, admitted and
deferred records, deterministic JSON, and the canonical CLI projection. It does
not run unrelated compiler suites.
