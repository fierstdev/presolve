# Structural keyed-host activation contract

This contract admits keyed structural hosts after the static-conditional
slice. It consumes compiler-issued item membership only; nested, Slot,
Effect, cleanup, and resume lifecycles remain excluded.

## Schema v16 product

Every `keyed_host_fragments` record carries `item_invocations`: the ordered,
duplicate-free invocation IDs stamped in its compiler-rendered
`item_template_html`. Each ID must belong to the enclosing structural
program's ordered occurrence set. The runtime validates the list before it
may consume the fragment and, at a later activation, must require the observed
compiler anchors in a newly rendered item fragment to equal it exactly.

The compiler derives this list from its generated fragment, never from source
spelling or runtime DOM. A missing, duplicate, unknown, or substituted ID is
an artifact failure; it may not select a fallback item template.

## Activation boundary

The keyed reconciler receives only its normalized key and retained
occurrence record. Creation materializes the declared anchors in program
creation order using `keyed:<normalized-key>`; retention and reorder preserve
the same occurrence identity; removal disposes in reverse creation order
before DOM removal. Cold boot replaces the otherwise inert server-rendered
item rows once through this same path; it does not treat them as live
occurrences. Slot-projected, nested, Effect, cleanup, and resume paths remain
excluded until their exact programs and disposal evidence are admitted.

Every rendered item is validated detached before insertion. A failed anchor,
materialization, binding, or event registration rolls back the new item and
does not retain a partially live keyed occurrence.
