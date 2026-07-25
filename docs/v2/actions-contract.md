# Actions contract

`presolve_compiler::action_authority` is the authored schema-v1 adapter boundary
for resolved `action(...)` fields. Each fact declares its stable action and
component identity, async/AbortSignal behavior, capture coverage, and browser
artifact environment. Unknown captures and server-only imports reject admission.
Cancellation is explicit only when the action accepts `AbortSignal`; no prior
invocation is implicitly cancelled. This authority does not infer facts from
method names or source spelling.
