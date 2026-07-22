# Phase N N1-A semantic package contract

## Authority and boundary

N1-A admits **semantic package bindings**, not arbitrary npm execution. A
caller supplies a versioned resolution table that maps the exact external
specifier in source to one contract. The compiler validates the contract and
binds only its declared named or default exports. It does not inspect package
source, read `node_modules`, resolve lockfiles, install dependencies, execute
package code, or infer behavior from TypeScript declarations.

This is an intentional compiler input, rather than a framework adapter or a
fallback runtime. A bare import without a matching contract fails with
`PSBIND1009`; a requested export absent from that contract fails with
`PSBIND1010`. Namespace package imports and package re-exports are not admitted
in N1-A.

## Contract schema

`SemanticPackageContract` schema version `1` contains:

* non-empty resolved `package` and `version`;
* `integrity` as an exact `sha256:` digest of 64 hexadecimal characters; and
* one or more non-empty export names, each with a semantic `kind`, non-empty
  `type_signature`, `runtime_module`, and `resume_policy`.

The supported declared kinds are `pure`, `capability`, `resource`, `codec`, and
`component`. They describe the public export only; the declaration never grants
hidden State mutation, dynamic dependency discovery, DOM ownership, hydration,
or resumability.

The table rejects duplicate specifiers without replacing the original
resolution. Unknown schema fields, unsupported schema versions, incomplete
export records, invalid integrity, or duplicate entries fail closed before
binding.

## Admitted source form

```ts
import { format } from "date-kit"
```

The caller must insert a validated `date-kit` contract declaring `format` into
the explicit `SemanticPackageResolutionTable`, then invoke
`build_binding_table_with_packages`. The resulting `BindingTable` records the
package coordinate, version, integrity, selected export, and declared semantic
kind. The default `build_binding_table` remains fail-closed and therefore
reports `PSBIND1009` for every external import.

## Deliberate next boundary

N1-A does **not** make calling an imported package export compiler-native.
Binding a `pure`, `capability`, `resource`, `codec`, or `component` symbol is
not proof of its expression, lifecycle, serialization, runtime, or resume
semantics. N1-A2 must admit each usable semantic kind through the full compiler
path: source use, type/boundary validation, dependency/lifecycle analysis, IR,
artifact/runtime/resume representation, canonical diagnostics, and browser
evidence. Until then, executable package use remains rejected rather than
silently becoming generic JavaScript.
