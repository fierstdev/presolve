# V2 canonical ASM adapter contract

The existing `ApplicationSemanticModel` consumes `ComponentNode` records
created by the alpha decorator graph. This contract defines the narrow V2
adapter required to publish decorator-free source without making that graph a
second recognition authority.

## Input and boundary

The adapter accepts exactly one `CanonicalAuthoredSemanticModelV1` for each
`ParsedFile`, produced by the installed TypeScript-authority bridge and
`lower_v2_authoring_v1`. It uses only canonical `Component`, `State`, and
`Action` records plus parser facts already retained for non-framework syntax.

It must not inspect decorators, component names, raw heritage spelling, import
text, or initializer callee spelling to decide framework meaning. Those
questions are closed before the adapter runs.

## Product

The adapter produces ordinary `ComponentGraph`-compatible records:

1. each canonical Component becomes one module-qualified `ComponentNode`;
2. each canonical State becomes an instance-owned `StateField`; and
3. each canonical Action reaches the existing runtime-action product only
   after its handler body and state-update semantics have a parser-owned path.

It preserves existing `SemanticId` construction, source provenance,
route-source ownership, and downstream route/publication interfaces. It does
not fork `ApplicationSemanticModel`, `RouteGraph`, or publication.

## Adoption and failure rules

File-route assembly receives an explicit per-file V2 model map. For a
decorator-free source selected by the CLI bridge, absence of a corresponding
canonical V2 model is an error; it must never fall back to the decorator graph.
Legacy-decorated files retain their named alpha compatibility path.

The first adapter slice proves Component publication and route reachability.
State and Action publication require separate source-provenance and runtime
evidence. Computed getters remain governed by `computed-source-contract.md`.

## Acceptance evidence

- A decorator-free generated route reaches the normal file-route graph solely
  from canonical V2 component evidence.
- An alias and indirect subclass retain exact TypeScript-authority proof and
  publish under normal file-route ownership rules.
- A missing/mismatched model, duplicate canonical component, or unproven
  State/Action join fails before publication.
- Decorator fixtures still prove compatibility, never the V2 route.
