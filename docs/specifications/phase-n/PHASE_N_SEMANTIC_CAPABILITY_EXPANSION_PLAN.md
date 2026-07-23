# Phase N: Semantic Capability Expansion

**Status:** Active implementation authority. The semantic capability registry
is the current source of truth for each family; only individually completed
slices are admitted.

## Objective

Expand Presolve from a deliberately narrow compiler language into a practical
application-authoring model without turning the framework into a JavaScript
runtime with optional compilation. The framework exposes only capabilities the
compiler can model end-to-end. Common web-development needs therefore become
compiler work first; the framework adopts them only after semantic lowering,
artifact, runtime, resume, diagnostic, and browser proof exist.

Phase N is not a promise to compile all TypeScript. It prioritizes the subset
developers expect to use daily in interactive web applications, with explicit
unsupported boundaries and a later compiler-owned opaque-code escape hatch.

## Governing principles

1. **One semantic authority.** The compiler owns parsing, type interpretation,
   dependencies, identity, storage, scheduling, artifacts, runtime activation,
   resume, diagnostics, and optimization for every compiler-native feature.
2. **Full-path admission.** A source feature is available only after it has:

   ```text
   parsed source
     -> normalized source fact
     -> semantic identity and type/boundary analysis
     -> dependency, ownership, and lifecycle analysis
     -> IR and optimization
     -> artifact/runtime/resume representation
     -> canonical diagnostics and browser proof
   ```

   Parsing or TypeScript type acceptance alone never makes a feature supported.
3. **No implicit fallback.** Unsupported code fails with a canonical diagnostic;
   it is never silently treated as a client-only expression, generic runtime
   callback, hydration boundary, or untracked reactive dependency.
4. **TypeScript is source syntax, not semantic authority.** The compiler may
   consume TypeScript type information through a versioned compiler-owned
   front-end contract, but framework declarations and arbitrary JavaScript
   execution cannot establish Presolve semantics.
5. **Preserve explicit boundaries.** Router, SSR request handling, deployment,
   package-manager discovery, and scaffolding remain metaframework concerns.
   Phase N accepts explicit package-resolution inputs and semantic package
   contracts; it does not resolve or install npm packages itself.
6. **Opaque means explicit.** Arbitrary TypeScript is permitted only through a
   later compiler-owned boundary that makes its lost guarantees visible.

## Capability classes

Every source construct belongs to exactly one class in compiler inspection and
diagnostics:

| Class | Meaning | Framework availability |
| --- | --- | --- |
| `native` | Fully compiler-modeled through artifacts and resume where applicable. | Supported. |
| `bounded` | Modeled only under explicit static restrictions, such as serializable values or known intrinsic calls. | Supported with listed restrictions. |
| `opaque` | Deliberately outside semantic lowering but isolated by a compiler-owned contract. | Deferred until N9. |
| `unsupported` | No trustworthy compiler contract exists. | Rejected with a canonical diagnostic. |

The `native` and `bounded` distinction is important: a serializable object
literal may be native State while an arbitrary class instance remains rejected,
even though both are ordinary TypeScript values.

## Third-party semantic package contracts

Presolve does not need to read a package's implementation to understand the
semantics of its use. It needs a versioned, explicit contract for the package's
public exports. A package contract is a compiler input, resolved from an
application-supplied package-resolution table; package-manager installation and
lockfile discovery remain outside compiler authority.

Each contract must declare:

- package coordinate, resolved version, export path, and content integrity;
- contract schema/version and compatible compiler capability versions;
- exported type signatures and structural input/output boundary schemas;
- semantic kind: `pure`, `capability`, `resource`, `codec`, or precompiled
  `component`;
- dependency behavior, lifecycle, error behavior, scheduling, and allowed
  execution targets;
- serialization, static-evaluation, and resume status; and
- runtime module/chunk identity when executable package code is required.

The compiler trusts the declared public contract for the selected package
version, not inferred package internals. Inspection records the contract
identity, integrity, export, semantic kind, and every application use site.
An integrity mismatch, unsupported contract version, unknown export, or invalid
use is a canonical compiler failure.

| Package kind | What the compiler understands | Where it may be used initially |
| --- | --- | --- |
| `pure` | declared signature, dependency behavior, deterministic operation/lowering, optional static evaluator | bounded expressions, Computed, templates only when static evaluation is declared |
| `codec` | value schema and encode/decode/resume behavior | State/Form/Resource boundaries |
| `capability` | inputs, terminal effects, activation and failure behavior | Effect, Action, or Resource terminal operations |
| `resource` | key, inputs, loading/success/error/cancellation and resume policy | compiler-owned Resource declarations |
| precompiled `component` | typed inputs, Slots, emitted artifact identity and lifecycle contract | compiler component invocation |
| no contract | nothing | rejected in compiler-native code; later N9 opaque boundary only |

Package contracts cannot claim hidden State writes, dynamic dependency discovery,
arbitrary DOM ownership, implicit hydration, or resumability without the
corresponding compiler contract. A package that does not publish a suitable
contract can still be isolated later as opaque; it is not retroactively treated
as native.

An application may publish an adapter contract for a package it controls, but
the adapter is itself a versioned compiler input with integrity, fixtures, and
all declared semantic restrictions. It does not let application TypeScript
pretend that unknown package internals are compiler-proven.

## Developer capability target

Phase N should make the following ordinary application patterns compiler-native
or bounded before it introduces opaque code.

| Need | Target compiler capability | Explicit non-goal in Phase N |
| --- | --- | --- |
| Organize applications | typed local/imported modules, exports, aliases, explicit third-party semantic package contracts | package installation or automatic npm resolution policy |
| Model application data | objects, arrays, readonly data, optional/nullable values, discriminated unions, selected maps/sets only when serializable | arbitrary prototype/class-instance State |
| Derive UI data | property/index access, conditionals, arithmetic/comparison/logical expressions, approved collection transforms, compiler-known pure helpers | arbitrary dynamic evaluation |
| Render real views | conditional blocks, keyed repeated blocks, fragments, common intrinsic attributes/classes/styles, event payload binding | runtime VDOM or DOM diffing |
| Compose components | typed required/optional inputs, defaults, callbacks, child component instances, slots | dynamic component constructors or reflection |
| Update data | assignment, object/array immutable updates, bounded collection mutation, event parameters, nested Action calls, explicit async/resource model | untracked mutation through aliases |
| Use browser integration | expanded compiler-known Effect capabilities, refs/element handles where identity is static, controlled controls | unrestricted DOM traversal or global mutable authority |
| Obtain data | compiler-owned Resources with typed loading/success/failure state, cancellation and resume rules | router loaders, server actions, arbitrary fetch in render |
| Validate/submit forms | reusable validation rules, nested values, controlled common controls, explicit serialization | custom controls inferred from DOM or implicit network transport |

## Phase sequence

### N0 — semantic capability registry and admission contract

Create one public compiler capability registry. Each entry records source form,
capability class, semantic owner, type rules, dependency rules, serialization
and resume policy, diagnostics, artifact impact, and proof fixture. Inspection
must expose the selected capability and rejection reason.

Freeze the admission checklist and a schema-version policy before changing
language support. Add a fixture harness that runs positive source, negative
diagnostic, determinism, artifact, and browser cases for every admitted family.

**N0 authority:** [semantic capability registry contract](PHASE_N_N0_CAPABILITY_REGISTRY_CONTRACT.md).

### N1 — modules, names, and types

Support the module features needed by real applications: local and relative
imports/exports, renamed imports, type-only imports, type aliases, interfaces,
literal types, unions, intersections where structurally meaningful, optional
properties, readonly fields, and generic instantiation for compiler-approved
generic utilities.

The compiler must produce canonical import bindings and type identities. It
must reject unresolved, cyclic-where-unsupported, dynamic, ambient-global, or
package-policy-dependent imports deterministically. Define a versioned
TypeScript front-end integration boundary before relying on checker facts not
present in the current parser model.

**N1 authority:** [module bindings contract](PHASE_N_N1_MODULE_BINDINGS_CONTRACT.md).

### N1-A — package semantic contracts and explicit resolution

Define the canonical package-resolution input and semantic package-contract
schema. Resolve normal import specifiers only through this caller-supplied map,
binding a package export to its exact contract/integrity identity. Add contract
diagnostics and an inspection record before admitting the non-executable
binding. No source fallback or package-manager resolution is permitted.

N1-A1 admits only that binding identity. A contract declaration is never itself
an executable capability, so the binding-table product is the terminal product
for this slice. The registry must distinguish it from package-export use.

N1-A2 admits executable package kinds one at a time. Each kind requires its own
cache key, inspection record, IR/artifact provenance, compatibility check, and
full runtime/resume proof before application code may invoke it.

Implement one end-to-end `pure` contract and one terminal `capability` contract
as vertical slices. Prove that package source is neither parsed nor inspected,
that a changed integrity value invalidates the build, and that the compiler
still derives every application-side dependency, activation, and runtime record
from the declared contract.

### N2 — values, expressions, and pure helpers

Extend the expression graph to support nested object/array literals, property
and optional access, index access with bounded keys, object spread with known
shapes, destructuring in local pure scope, ternaries, nullish operations,
template strings, and selected numeric/string/date-free pure operations.

Introduce compiler-registered pure helpers. A helper has an exact signature,
serializability rule, dependency behavior, constant-folding policy, and IR
operation; arbitrary function calls remain unsupported. Start with common
collection operations (`map`, `filter`, `find`, `some`, `every`, `reduce` under
bounded accumulator rules), string formatting, and stable key extraction.

#### N2-A — template interpolation

Admit untagged template literals in Computed getters as a complete compiler
vertical slice. Preserve cooked segments, compiler-lower every interpolation,
emit a versioned runtime instruction, and prove generated-browser execution.
Do not treat tagged templates, template factories, or interpolation callbacks as
equivalent syntax.

#### N2-B — static index access

Admit bracket access only when its index is a string literal or non-negative
integer literal in a supported Computed getter. Lower the object and literal
index through the canonical expression graph and IR; emit a fail-closed
runtime instruction that reads own properties only. Dynamic keys, negative or
non-integral numbers, optional indexing, prototype traversal, and index access
in Actions or Effects remain outside this admission.

#### N2-C — boolean computed conditionals

Admit `condition ? whenTrue : whenFalse` only in a supported Computed getter.
The condition must have a compiler-known boolean type and each branch must
already be compiler-supported. Lower a pure `Select` instruction; do not apply
JavaScript truthiness, branch callbacks, or control-flow statements.

#### N2-D — optional member access

Admit `value?.property` in a supported Computed getter. Preserve optionality in
the parser, expression graph, IR, and artifact; generated reads remain
own-property and null-safe. Optional calls, optional index access, writes, and
reflection remain outside this admission.

#### N2-E — compiler-registered numeric helper

Admit only exact `Math.abs(value)` as a registered pure helper in a supported
Computed getter. It has one numeric operand, compiler-derived dependencies,
unary IR lowering, and a versioned runtime operation; generic calls remain
outside the language.

### N3 — State, Actions, Computed, and Effects over real data

N3-A admits recursively serializable record/array State and complete-field
Action replacement only. It does not authorize nested writes, alias mutation,
spread updates, collection callbacks, event payload projection, locals, early
returns, or async Actions; each requires its own compiler/lowering/runtime
contract.

Allow State fields to hold structurally serializable records, arrays, optional
values, and discriminated unions. Add compiler-recognized immutable updates,
safe indexed updates, and bounded collection operations. Define alias and
escape analysis so a State-derived mutable object cannot be changed behind the
compiler's back.

Extend Computed to consume the N2 expression vocabulary and pure helpers.
Extend Actions with typed event parameters, nested action batching, early
returns, bounded local variables, and a single explicit async transition model.
Extend Effects only through an audited capability catalog; each new browser
operation needs scheduling, cleanup/lifetime, failure, and resume behavior.

### N4 — structural templates and ordinary DOM semantics

Lower compiler-owned conditional, repeated, empty, and fallback template
regions. Repetition requires explicit stable keys and instance/lifecycle plans;
the compiler must never derive identity from DOM position. Support fragments,
standard intrinsic attributes, class/style values with bounded representations,
accessibility attributes, and event payload projection.

Add static element references only when the compiler can prove identity and
destruction. Dynamic HTML, arbitrary JSX factories, arbitrary render callbacks,
and DOM traversal remain unsupported.

### N5 — typed component API and composition

Add compiler-modeled component inputs: required/optional typed fields, literal
or expression defaults, callback input contracts, and static checked call sites.
Add named component outputs only if their Action ownership and activation are
explicit. Preserve component-instance identity, Context visibility, Slots, and
destruction semantics through all new input forms.

Do not add React-compatible props spreading or untyped children as shortcuts.
Every passed value must have a compiler-known owner, type, serializability, and
reactivity classification.

### N6 — Resources and asynchronous application state

N6-A is complete: the compiler has separate stable identities for a Resource
declaration and each component-instance activation, validated serializable
data/error declaration products, and a generation-scoped lifecycle state
machine. It intentionally does not yet admit Resource source syntax or a
runtime artifact.

N6-B is complete: integrity-checked semantic-package `resource` exports now
require a closed endpoint contract that states execution boundary,
cancellation, and reload/snapshot resume policy. It intentionally stops before
application source selects an endpoint or a package module is loaded.

Introduce a compiler-owned Resource declaration with explicit key, input
dependencies, loading/success/error state, cancellation, retry, invalidation,
serialization, and resume rules. Resources may invoke only registered
capabilities or declared service endpoints; they cannot be arbitrary Promise
code hidden in Computed or render.

Define server/client execution boundaries and artifact activation before adding
fetch-like capabilities. This phase enables application data needs without
silently creating server actions, routing loaders, or a generic client runtime.

### N7 — production-grade forms and browser integration

Build on the frozen Form model with nested value paths, richer native control
sets, reusable compiler-defined validators, field arrays with stable identity,
focus/selection capabilities where statically anchored, and explicit
accessibility/error-message bindings. Extend serialization only through a
versioned boundary schema and compiler-issued submission records.

No Form feature may use DOM discovery as semantic authority or introduce an
implicit HTTP submission implementation.

### N8 — semantic optimization and migration freeze

For every N1–N7 capability, prove source-order determinism, artifact stability,
incremental invalidation, production chunking, resume behavior, lifecycle
destruction, malformed-artifact failure, and diagnostic identity. Publish a
capability matrix, compatibility policy, migration guide, and rejected-syntax
catalog. Freeze the supported semantic subset before adding opaque code.

### N9 — explicit opaque TypeScript boundary

Only after N8, introduce `opaque` as a compiler-recognized boundary. It must
not be a generic escape hatch. An opaque declaration specifies its execution
target, activation mode, typed serializable inputs/outputs, permitted compiler
entry points, lifecycle, error policy, resume policy, and package/runtime
integrity identity when it uses third-party code.

Initial restrictions:

- opaque code may not mutate compiler-owned State, Form, Context, or component
  instance storage directly;
- opaque code may not participate in render dependency inference or static HTML;
- opaque code may not claim resumability unless a separate opaque-resume codec
  contract is accepted;
- opaque code may run only as a terminal Action/Effect/Resource capability with
  compiler-projected inputs and a compiler-recorded activation boundary; and
- diagnostics and inspection must label every affected artifact as opaque.

Later phases may define opaque client-island rendering or opaque server adapters,
but each needs a separate ownership, hydration/resume, and failure contract.

### N10 — Phase N freeze and framework adoption

Freeze the semantic capability registry, source forms, type/front-end version,
artifact schemas, runtime protocol, opaque boundary, test matrix, and migration
policy. Only then may the Presolve Framework expose the admitted Phase N forms
through declarations and examples. Framework adoption is a conformance phase;
it must not reimplement any new semantic family.

## Required proof per capability

Every Phase N admission requires positive/negative source fixtures, exact
diagnostic evidence, identity/ownership and type assertions, deterministic
artifact comparison, incremental invalidation proof, production artifact proof,
browser execution where applicable, resume proof where applicable, and an
explicit unsupported/opaque classification test.

## Explicit exclusions

Phase N does not promise arbitrary npm packages in compiler-native code,
arbitrary classes in reactive State, dynamic `eval`, reflection-based dependency
discovery, arbitrary JSX factories, unrestricted DOM access, untracked
mutation, dynamic code loading, router semantics, deployment, or a second
framework runtime.
