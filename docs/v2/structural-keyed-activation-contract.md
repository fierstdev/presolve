# Structural keyed-host activation contract

This contract prepares keyed structural hosts for activation after the admitted
static-conditional slice. It is a compiler-product amendment only: it adds
the exact item-fragment invocation membership required to distinguish a
created keyed occurrence from a malformed or empty fragment. It does not yet
activate keyed rendering.

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

## Future activation boundary

The keyed reconciler will receive only its normalized key and retained
occurrence record. Creation materializes the declared anchors in program
creation order using `keyed:<normalized-key>`; retention and reorder preserve
the same occurrence identity; removal disposes in reverse creation order
before DOM removal. Slot-projected, nested, Effect, cleanup, and resume paths
remain excluded until their exact programs and disposal evidence are admitted.
