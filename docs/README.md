# Presolve documentation

This is the public documentation index. A document is either a **Reference**
(the current contract for an established surface), a **Guide** (an accepted way
to use an established surface), or an **Archive** (historical, non-normative
engineering material). A guide never creates semantics that its linked
reference does not establish.

## Current references

| Label | Subject | Authority |
| --- | --- | --- |
| Reference | accepted L9/L11 command adapters and limitations | [CLI reference](cli-reference.md) |
| Reference | help, reserved exits, L10 schemas, and package exports | [public surface matrix](public-surface-matrix.md) |
| Reference | existing frozen language, runtime, platform, and editor authorities | [frozen contract map](frozen-contract-map.md) |
| Reference | explicit CLI configuration and build/check boundary | [CLI build/check](cli-build-check.md) |
| Reference | service, incremental, cache, workspace, and watch boundaries | [platform](compiler-platform-contract.md), [service](compiler-service-contract.md), [workspace](workspace-architecture-contract.md), [watch](watch-mode-contract.md) |
| Reference | State/Actions/Computed runtime boundary | [runtime contract](runtime-contract.md) |
| Reference | Context, Components/Slots, and Forms | [Context](context-contract.md), [Components](component-contract.md), [Forms](forms-contract.md) |
| Reference | resumability and production runtime | [resumability](resumability-contract.md), [production optimization](production-optimization-contract.md) |
| Reference | testing lanes and alpha examples | [testing](testing-contract.md), [reproducibility lanes](reproducibility-lanes.md), [examples](examples-contract.md) |

## Guides

The five bounded alpha examples are the only current Guides: [Counter](../examples/counter/), [Components/Context/Slots](../examples/components-context-slots/), [Forms](../examples/forms/), [explicit workspace](../examples/explicit-workspace/), and [production/resume](../examples/production-resume/). Each guide is an explicit-input proof; it does not scaffold a project, discover sources, or authorize a reserved capability.

## Ownership and version policy

The compiler owns language semantics, canonical identities, diagnostics, and
immutable products. Public documentation may describe only behavior proved by
the linked current reference or executable fixture. A frozen contract retains
its stated schema/version meanings; an incompatible documentation claim needs
an approved contract amendment and the corresponding product verification. The
active Phase L continuation boundary is [L13--L21](specifications/phase-l/PHASE_L_L13_L21_CONTINUATION_CONTRACT.md).

## Verifiable command snippets

Every executable public command snippet uses this exact marker immediately
before a fenced `sh` block:

<!-- presolve-snippet: id=<lowercase-dash-id>; kind=command -->
```sh
presolve <accepted-command> ...
```

The snippet verifier owns the marker grammar. It executes only snippets with
`kind=command`, using the listed explicit inputs; prose and non-command blocks
are never inferred as commands.

## Archive

The [engineering archive](archive/engineering/README.md) preserves historical
Presolve-era material. It is **Archive** documentation: non-normative, not an
active guide or reference, and not a source of public command authority.
