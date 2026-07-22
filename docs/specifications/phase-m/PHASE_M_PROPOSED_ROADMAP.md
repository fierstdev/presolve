# Phase M roadmap: Presolve Framework Foundation

**Status:** M0/M1 owner-accepted; M2 and later remain sequential implementation
authority only when their preceding slice is complete and committed.

## Product boundary

Phase M builds a private Presolve Framework around the compiler exactly as it
is frozen. It is a TypeScript authoring and conformance product, not a new
language or UI runtime. Its public-facing source vocabulary is the frozen
compiler vocabulary in the [M1 conformance authoring contract](PHASE_M_CONFORMANCE_AUTHORING_CONTRACT.md).

The compiler remains the sole authority for parsing, language semantics,
semantic and runtime identities, State storage, dependency graphs, action and
effect scheduling, Context resolution, component and Slot planning, Forms,
diagnostics, artifacts, resumability, and optimization. Framework code must
never reinterpret source or replace any of those decisions.

The future Presolve Metaframework remains out of scope. Routing, data loading,
server rendering, build/dev orchestration, hosting, deployment, installation,
project discovery, and `presolve create` are deferred. `create`, `dev`,
`benchmark`, and `doctor` remain reserved exit-6 command families throughout
Phase M.

## Governing authorities

Authority is applied in this order:

1. Frozen compiler, runtime, Context, Components/Slots, Forms, resumability,
   production, CLI, and platform contracts.
2. The [M0 framework constitution](PHASE_M_FRAMEWORK_CONSTITUTION.md).
3. The [M1 conformance authoring contract](PHASE_M_CONFORMANCE_AUTHORING_CONTRACT.md).
4. The current implementation slice below.

No Phase M slice may alter frozen compiler source forms, compiler bytes,
artifact schemas, diagnostics, runtime protocol, or reserved-command status.
When a desired convenience is not a frozen form, the framework declares it
unavailable; it does not emulate it.

## Architectural decisions

| Decision | Phase M direction |
| --- | --- |
| Source language | Preserve the compiler's exact source forms. In particular, component tags remain explicit, State remains `state(initializer)`, and Context/Forms/Slots retain their existing declarations. |
| Type delivery | Start with private `@presolve/framework-types` ambient declarations selected through `tsconfig` `types` and the pinned TypeScript 7.0 native CLI; no application-source aliases, JSX transform, decorator transform, compiler API, or runtime registration. |
| Repository topology | Create `framework/` only in M2. It owns framework packages, conformance fixtures, examples, and docs, but cannot change compiler crates or take ownership of existing compiler packages. |
| Compiler handoff | Invoke only the accepted explicit `presolve` project/configuration path. Inputs stay caller-supplied; there is no source/project discovery, parser, transform, semantic analyzer, product decoder, or alternate compiler route. |
| Runtime | Reuse emitted compiler runtime/resume products unchanged. No framework state store, renderer, hydration layer, Context lookup, scheduler, or artifact writer exists. |
| Diagnostics | Preserve canonical diagnostics exactly; framework advice is optional, separate, and non-authoritative. |
| Package policy | Keep every Phase M package private. Phase M does not reserve or publish a short `presolve` package. |
| Compatibility | Fail closed on unsupported framework/compiler/CLI/product tuples; framework versions cannot reinterpret compiler products. |

## Slice sequence

### M0 — framework constitution and acceptance

**Completed authority:**
[framework constitution](PHASE_M_FRAMEWORK_CONSTITUTION.md).

It accepts the framework/compiler/metaframework boundary, private package
direction, opaque explicit-command handoff, compatibility policy, and
metaframework deferral. It changes no frozen compiler or platform contract.

### M1 — frozen authoring conformance contract

**Completed authority:**
[conformance authoring contract](PHASE_M_CONFORMANCE_AUTHORING_CONTRACT.md).

It maps every initially supported public form to exact compiler syntax and
records unavailable alternatives. It intentionally keeps component tag strings,
`state(initializer)`, compiler-owned Context designators, direct slot syntax,
and frozen Forms syntax.

### M2 — isolated ambient type package

**Completed authority:**
[M2 framework types contract](PHASE_M_M2_FRAMEWORK_TYPES_CONTRACT.md).

Create `framework/packages/framework-types` as a private declaration-only
package. It provides the ambient TypeScript names needed by the Counter form
through an explicit `tsconfig` `types` entry. It emits no JavaScript and cannot
install decorators, a JSX runtime, a renderer, a transform, or reactive
behavior.

Proof: type-resolution fixtures for the exact Counter form; an import/output
audit; and canonical compiler check evidence showing the compiler input and
result remain unchanged.

### M3 — explicit compiler handoff contract

**Completed authority:**
[M3 explicit handoff contract](PHASE_M_M3_EXPLICIT_HANDOFF_CONTRACT.md).

Define and implement the narrow framework handoff over the compiler's existing
artifact-publication boundary. It forwards one caller-supplied source path and
one caller-supplied output directory as `presolve build <source> --out
<directory>`, optionally retaining the compiler-owned production flag. The
handoff returns the executor result unchanged; it does not decode, synthesize,
rewrite, or infer compiler products or artifact locations.

Proof: request-shape fixtures; no-source-retention/dependency audit; unchanged
canonical diagnostics; incompatible version/command failure fixtures.

### M4 — Counter vertical slice

**Audit authority:** [M4 public artifact-publication audit](PHASE_M_M4_PUBLICATION_AUDIT.md).

M4 selects the existing compiler artifact publisher as the one framework build
path. L9 build/check publishes project status and snapshot identities, while
`presolve build <source> --out <directory>` publishes the compiler artifacts
required for browser proof. The framework invokes it but does not reimplement
it or decode its output.

Prove the exact M1 Counter through M2 types and M3 handoff:

```text
unchanged author source
  -> ambient type resolution
  -> explicit canonical compiler command
  -> canonical HTML/runtime artifacts
  -> frozen browser click probe
```

The evidence proves the existing compiler action binding updates the exact
binding without hydration or a framework reactive runtime. Counter does not
invent `@state()` or optional component identifiers.

### M5 — reactive conformance families

Add exact frozen State, Action, Computed, and Effect forms one family at a
time. The framework package remains declarations and opaque handoff only.
Canonical compiler fixtures establish initializer limits, action writes,
direct/captured events, computed dependency/caching behavior, action-batch
ordering, and effect capability diagnostics.

**M5-A authority:** [Computed conformance](PHASE_M_M5_COMPUTED_CONFORMANCE.md).

### M6 — composition conformance families

Add exact frozen component invocation, repeated/keyed instance behavior, Slot
declarations/content/outlets, and Context declarations/providers/consumers.
No inputs, props, callback arguments, Context factory, provider getter,
runtime lookup, Slot forwarding, or framework lifecycle abstraction is added.
Each family requires its existing compiler and browser evidence where runtime
execution is already frozen.

### M7 — Forms and resume/production conformance

Expose only the frozen Form, Field, validation, submit, and explicit host
forms. Then prove the framework uses existing production/resume artifacts
unchanged, including malformed-artifact failure behavior. It adds neither a
new Form API nor a resume/runtime wrapper.

### M8 — framework DX, examples, and compatibility

Add a compiler-backed explanation presentation, an error guide, conformance
examples, and the framework compatibility matrix. All explanation data comes
from existing command/product surfaces; no JavaScript source scan or new editor
intelligence is allowed. Examples begin with Counter and advance only after the
underlying family passes its conformance matrix. The matrix keeps TypeScript
7.0 and a future TypeScript 7.1 compatibility row separate; 7.1 requires an
explicit pinned-toolchain conformance rerun before support is declared.

### M9 — framework freeze and metaframework handoff

Freeze private package exports, declaration behavior, handoff grammar,
supported compiler/CLI/product tuples, examples, diagnostics presentation, and
the explicit unavailable-authoring list. Run the M0–M8 matrix and inherited
Phase L gates from a clean tree. The handoff names the missing authority a
future metaframework must supply; it does not pre-implement it.

## Evidence matrix

| Concern | Required proof |
| --- | --- |
| Frozen-source conformance | exact source-form fixture and cited compiler authority for each supported family |
| Type package honesty | type-resolution fixture, declaration-only/output audit, and no fabricated compiler semantics |
| Compiler handoff | explicit caller inputs, canonical command/diagnostic evidence, no discovery or source interpretation |
| Runtime integrity | inherited browser/runtime and resume fixtures with no framework renderer, hydration, scheduler, or state store |
| Product integrity | opaque artifact locations/versions only; no framework decoder, writer, or canonical-byte change |
| DX integrity | compiler-backed explain/diagnostic presentation with unchanged codes, spans, labels, ordering, and identities |
| Compatibility | fail-closed framework/compiler/CLI/product matrix and migration/rollback documentation |
| Metaframework deferral | dependency/source audit for no router, server, bundler, deployment, generator, package manager, or reserved command |

## Current boundary

M4 is complete. M5-A Computed conformance is complete. M5-B may select only
the existing Effect declaration plus its compiler-backed capability and browser
proof.
