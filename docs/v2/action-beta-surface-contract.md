# V2 beta Action surface contract

This contract completes the V2 Action gate by fixing the exact beta source
surface. It is intentionally a closed compiler product, not a JavaScript
handler interpreter or a source-translation authority.

## Admitted forms

A decorator-free, TypeScript-authority-proven `action(handler)` field of a
canonical Component is admitted when its handler is synchronous and
block-bodied, has no unsupported statements, no free captures, no server
imports, and only performs ordered updates of State owned by that Component.
The beta surface consists of:

1. existing literal, increment, decrement, add/subtract-assignment, and
   boolean-toggle State operations;
2. primitive (`string`, `number`, `boolean`, or `null`) typed parameters,
   each assigned once to matching primitive State and supplied by exact static
   event arguments; and
3. primitive local literals declared before, and assigned exactly once to,
   matching primitive State.

Parameters lower to the existing `assign_parameter` ordinal operation. Local
literals lower to ordinary literal `assign` operations. Handler source,
parameter names, and local declarations never enter published artifacts.
Action endpoint, batch, State, event, and resume identities remain the
compiler-issued identities already used by the runtime.

## Explicit beta exclusions

Expression-bodied or asynchronous handlers, `AbortSignal`, defaults, rest or
destructured parameters, event-object forwarding, dynamic event arguments,
computed locals, object/array locals, branches, loops, calls, free captures,
server imports, reassignment, and arbitrary statements reject before
publication. These are intentional diagnostics, not compatibility fallbacks
or latent runtime evaluation paths. Any future admission requires its own
source-faithful parser facts, resolved-authority and ownership rules,
artifact operation, cold/resume proof, and contract amendment.

## Completion evidence

`runtime_browser::decorator_free_v2_action_field_runs_through_compiler_artifacts_in_a_real_browser`,
`runtime_browser::static_callback_argument_updates_state_through_compiler_action_parameter`,
and `runtime_browser::serializable_action_local_updates_state_from_compiler_generated_runtime`
prove the ordinary, typed-parameter, and local-literal forms in browser
artifacts. The release dry run executes that full matrix together with
deterministic production, diagnostics, packaging, and packed-scaffold gates.
