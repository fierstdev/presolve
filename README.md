# EdgeZero Starter Repository

EdgeZero is a compiler-centered web authoring system. This repository starts with the smallest useful vertical slice:

```txt
source file
  -> source summary
  -> spans and basic declarations
  -> diagnostics
  -> explain output
  -> fixture
```

This is not the real TSX compiler yet. It is the first learning and infrastructure slice. The purpose is to establish repository layout, CLI shape, fixture discipline, documentation discipline, and source-location handling before committing to a parser backend.

## First commands

```sh
# after installing Rust and pnpm
cargo test --workspace
cargo run -p ezc_cli -- explain fixtures/0001-source-summary/input/Counter.tsx
cargo run -p ezc_cli -- explain fixtures/0001-source-summary/input/Counter.tsx --format json
cargo run -p ezc_cli -- check fixtures/0001-source-summary/input/Counter.tsx
```

## Check policy

Use `ezc_cli check` to compile one or more source files, assemble the canonical
Application Semantic Model (ASM), and report parser, compiler, and ASM
validation diagnostics.

```sh
ezc_cli check <file> [file...] \
  [--format text|json] \
  [--category parser|compiler|validation] \
  [--fail-on error|warning|info]
```

The project default is `--fail-on error`: parser errors fail the command, while
parser warnings and informational diagnostics remain visible without failing it.
`--fail-on warning` also fails on warnings, and `--fail-on info` fails on every
parser diagnostic. Compiler diagnostics and ASM validation diagnostics always
fail `check`, regardless of the selected parser threshold.

Repeat `--category` to limit diagnostic detail in text or JSON output. Category
filters never change summary counts or the command's exit status. JSON output
includes the selected parser threshold in `fail_on` so automation can record
the effective policy. Each parser diagnostic includes `labels` with source
`line`, `column`, `start`, and `end` coordinates; text output prints the same
file-qualified label locations below the diagnostic. Compiler diagnostics with
reliable source locations include optional JSON `provenance` using `path`,
`line`, `column`, `start`, and `end`; diagnostics without a reliable source
location omit that field. Check policy is currently selected per command; no
project configuration file is interpreted yet.

Computed diagnostics use stable compiler codes: `EZC1034` purity violations,
`EZC1035` dependency cycles, `EZC1036` invalid declarations, `EZC1037`
unsupported bodies, `EZC1038` unresolved reads, `EZC1039` declared-return type
mismatches, and `EZC1040` non-serializable results. Every computed diagnostic
is derived from canonical compiler products and includes source provenance.

## ASM entity inspection

Inspect one canonical semantic entity with its ownership, provenance,
containment, relations, and overlap-based compiler diagnostics:

```sh
ezc_cli asm <file> --entity <semantic-id> [--format text|json]
ezc_cli asm <file> --source <path> --offset <byte> [--format text|json]
ezc_cli explain <file> --entity <semantic-id> [--format text|json]
ezc_cli explain <file> --source <path> --offset <byte> [--format text|json]
```

Use `ezc_cli asm <file> --format json` to discover the available semantic IDs.
The selected-entity document includes the entity itself, direct child IDs,
nearest-first parent IDs through the application root, descendant count, incoming and outgoing references,
and related compiler diagnostics. This is the canonical semantic-navigation
surface: ownership is navigated through parents and direct children, while
relations are navigated through ordered incoming and outgoing references. An
unknown semantic ID fails explicitly.

Source selection chooses the uniquely narrowest semantic span covering the
given byte offset. No match or tied narrowest spans fail explicitly; `--entity`
cannot be combined with `--source` or `--offset`.

`ezc_cli explain <file>` retains its legacy source-summary output. Supplying
an entity selector (or an entity filter) activates the same read-only,
canonical ASM inspection path as `ezc_cli asm`, including its text/JSON
document, source selection, deterministic filtering, and explicit failures.

For a computed entity, the inspection record additionally contains the
compiler-owned computed type, transitive dependencies and dependents,
zero-based evaluation-order and update-batch positions, purity,
serializability, and canonical IR function identity. Unplanned values use
`null` for their evaluation positions or IR function identity; the CLI does not
infer or discover this metadata at runtime.

Computed inspection is frozen at JSON schema v2. The computed record fields
(`computed_type`, `dependencies`, `dependents`, `evaluation_order`,
`evaluation_batch`, `purity`, `serializability`, and `ir_function`) are ordered
compiler products; any incompatible inspection change requires a new schema
version.

Selected entity inspection supports optional filters:

```sh
ezc_cli asm <file> --entity <semantic-id> --child-kind method
ezc_cli asm <file> --entity <semantic-id> --reference-kind action-state
```

`--child-kind` accepts `component`, `state-field`, `method`, `action`,
`event-handler`, `template`, or `template-entity`. `--reference-kind` accepts
`action-state`, `computed-state`, `computed-computed`, `event-method`,
`template-state`, `template-computed`, or `template-local`. Filters require an
entity selector and affect only the returned children or relations.

## Computed fixture suite

The Phase E computed fixtures span `fixtures/0043` through `fixtures/0051`:
template bindings, runtime chains, batched invalidation, arithmetic, diamond
dependencies, cycles, immutable constant folding, serialization, and
multi-file identity. The real-browser cases exercise the runtime chain,
batching, and diamond fixtures; the remaining cases validate canonical compiler
products and diagnostics.

## Context contract

Phase G Context semantics, compiler authority, runtime ordering, frozen schema
versions, and explicit unsupported behavior are documented in
[`docs/context-contract.md`](docs/context-contract.md). Runtime Consumer access
uses compiler-generated Context slot identities only; it performs no Provider
lookup, Context-name matching, ancestry traversal, dependency reconstruction,
or binding reselection.

## Component contract

Phase H Component and Slot syntax, canonical identities, instance Context,
caller-owned Slot content, compiler-planned runtime ordering and structural
updates, frozen schemas and diagnostics, and unsupported behavior are documented
in [`docs/component-contract.md`](docs/component-contract.md). The runtime
consumes closed compiler-ID tables and performs no component or Slot discovery,
Provider or parent search, authored-name lookup, or virtual-DOM diffing.

## Resumability contract

Phase J same-build continuation, snapshot/manifest versions, restore order,
exact lazy activation, diagnostics, and runtime no-discovery invariants are
frozen in [`docs/resumability-contract.md`](docs/resumability-contract.md).

## Computed values: supported contract and limits

`@computed()` currently applies only to synchronous getters with one supported
return expression. The compiler supports literals, direct `this.<state>` and
`this.<computed>` reads, arithmetic, comparison, logical and nullish operators,
supported unary operators, and supported member access. It owns dependency
resolution, scheduling, IR lowering, cache metadata, and runtime evaluation
plans.

State mutation, actions, effects, async work, resources, arbitrary calls, and
nondeterministic operations are rejected through computed purity diagnostics.
Unsupported bodies, unresolved reads, incompatible declared returns, cycles,
and non-serializable results remain explicit compiler diagnostics. The runtime
does not discover dependencies, parse getters, or infer cache identity.

Current runtime limitations remain explicit: computed template bindings are not
refreshed from cache changes, computed cache values are not yet persisted or
restored, and unsupported getter syntax does not receive partial execution.

## Semantic graph export

Export the canonical ASM as a stable JSON graph:

```sh
ezc_cli asm <file> [file...] --format graph
```

The graph schema has a version, ordered application roots, typed semantic nodes
with source provenance, and ordered edges. Ownership edges point from parent to
child; resolved action/state, event/method, and template/state edges retain
their canonical direction. The export intentionally excludes parser facts,
backend-local node IDs, manifests, runtime artifacts, and diagnostics. Entity
selection and filters are not accepted for whole-application graph export.

## Constant expression state initializers

The compiler recognizes constant expressions inside `state(...)` initializers.
It lowers the authored expression into a compiler-owned constant-expression
model, evaluates it during compilation, and retains the canonical expression
for ASM inspection and ComponentGraph output:

```tsx
total: number = state((1 + 2) * 3);
ready: boolean = state(((1 + 2) * 3) >= 9);
```

Arithmetic supports `+`, `-`, `*`, `/`, and `%`. Comparisons accept numeric
arithmetic operands and support `===`, `!==`, `<`, `<=`, `>`, and `>=`; they
evaluate to a boolean. Logical `&&` and `||` compose boolean literals and
comparisons with compiler-time short-circuit semantics. Division or remainder
by zero, invalid numeric literals, and non-finite arithmetic results report
`EZC1022` for arithmetic initializers, `EZC1023` for comparisons, or `EZC1024`
when reached through a logical initializer. This is deliberately not general
JavaScript evaluation: state reads, local variables, calls, coercions,
truthiness, unary operators, and action expressions are outside this slice.

Nullish coalescing `??` selects between supported constant primitives and B1-B3
expressions. It evaluates left-to-right at compile time and only evaluates the
right side when the left result is `null`; a reached invalid arithmetic branch
reports `EZC1025`.

Unary `!`, `+`, and `-` are evaluated by the compiler for supported boolean and numeric constant expressions.

Methods may declare supported serializable local constants. They are compiler-owned lexical metadata, visible through ASM inspection, and do not yet participate in render bindings, action execution, or runtime evaluation.

Methods may also declare supported identifier parameters. The compiler lowers their
names and source provenance into the owning method's ASM metadata in authored
order. Parameters do not execute, close over values, resolve bindings, or
support destructuring, defaults, or rest declarations.

Supported `render()` locals may be referenced by exact identifier in normal
template bindings and dynamic attributes. The compiler resolves those references
to canonical local-variable ASM entities, emits `template-local` edges, and uses
the known serializable value for initial static HTML. Those existing serializable
local entities also receive inferred canonical semantic types. List-item scopes,
duplicate local names, member access, calls, runtime updates, closure capture,
and non-serializable local expressions remain unresolved.

Supported identifier method parameters are likewise canonical method-owned ASM
entities. Their authored annotations lower to declared semantic types with
stable identity and provenance; defaults, destructuring, rest parameters,
optionality semantics, and call-site validation remain future work.

Method return annotations lower to declared semantic result types, while the
current top-level serializable `return` forms infer a result type when unannotated.
JSX, state/local expressions, async promises, branching, and return diagnostics
remain future work.

Constant state expressions are lowered without evaluation, then folded by an
immutable compiler pass over the ASM. The pass produces a new canonical model
with folded state values and diagnostics, which backend products consume. The
parser and browser runtime do not evaluate these expressions.

The ASM owns a canonical expression graph: state fields resolve to stable graph
roots, and folding and inspection read the same graph nodes rather than
reinterpreting field-local expression trees. Every graph node retains canonical
source provenance, including its file path and authored expression span. ASM
queries provide deterministic expression lookup, direct dependencies and
dependents, owner state fields, and path/offset provenance selection.

## Semantic type foundation

The ASM owns a compiler-defined `SemanticType` algebra independent of raw
TypeScript spelling. Current Phase C lowering covers primitive and literal
annotations, arrays, tuples, structural objects, unions, local/imported type
aliases, inferred direct state values, state-initializer compatibility, and the
existing canonical state-initializer expression graph. Each assignment retains
its stable subject identity, origin, declared-or-inferred status, and source
provenance. Arithmetic, comparison, logical, unary, and nullish operators use
explicit compiler-owned operand and result contracts. Operator diagnostics,
state reads, locals, templates, actions, normalization, and the final general
assignability engine remain later Phase C work.

Each future type assignment has a stable canonical identity, a semantic origin,
declared-or-inferred status, and source provenance. C2 establishes this metadata
shape without lowering authored annotations into assignments yet.

## Repository rules

- No major feature without an issue.
- No durable architecture decision without an ADR.
- No user-facing syntax without an RFC.
- No compiler behavior without a fixture.
- No compiler inference without explain output.
- No area switch until the current slice has exit criteria recorded.

## Current slice

See [`fixtures/0001-source-summary/README.md`](fixtures/0001-source-summary/README.md).
