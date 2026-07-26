# V2 action-field runtime adoption contract

V2 `action(handler)` fields are recognized through canonical authored
semantics, but the existing runtime action product is based on decorated
methods and parser-derived state-update statements. This contract prevents an
implicit handler translation while defining the next adapter boundary.

## Inputs

The adapter accepts a canonical V2 Action declaration only when all of these
facts are present:

1. the source-AST field initializer has been joined to resolved `action`
   identity by `action_field_lowering`;
2. its owning class is a canonical V2 Component;
3. a parser-owned handler-body product identifies supported updates and source
   spans; and
4. capture, async/cancellation, and server-import facts are admitted by
   `ActionAuthorityV1`.

No adapter may infer an action from a property name, initializer spelling, or
decorator. A handler whose body lacks the required parser/authority evidence
is rejected before runtime or publication.

## Parser handler boundary

`presolve_parser::ParsedInitializerCall::inline_handler` is the sole
parser-owned input for the handler-body condition above. It is emitted only
for a direct initializer call with exactly one inline arrow or function
argument. It records the function and body spans, async and expression-body
form, each existing `ParsedStateUpdate`, and the span of every other non-empty
statement.

The parser never treats the call as `action` and never lowers a handler. An
expression-bodied handler, an unsupported statement, a missing handler, or a
handler with no later authority admission is rejection evidence. The later
projection may consume the retained update sequence, but must not reparse
source text or treat an empty unsupported list as capture proof.

## Product

A versioned action-field projection maps admitted handler operations to the
existing `ComponentAction` runtime product while retaining the canonical
component and field identities. It must not synthesize a legacy
`ComponentMethod`, alter state semantics, or parse JavaScript independently.
The endpoint identity and downstream action-batch migration are governed by
the [action endpoint identity contract](action-endpoint-identity-contract.md).

## Acceptance

- An aliased canonical `action` field updates canonical V2 State with exact
  source provenance in cold execution.
- The resumed path retains the same component/state/action identities and
  observes the same update result.
- Unsupported handler syntax, missing capture evidence, server imports,
  lookalike calls, and malformed authority output fail before publication.
- Decorated methods remain alpha compatibility evidence and are not invoked by
  the V2 action-field projection.
- Parser retention alone is not runtime adoption: the action projection and
  its cold/resume browser evidence remain required before this contract's
  runtime acceptance is met.
