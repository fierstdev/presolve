# Structural nested-activation contract

Schema v17 publishes `nested_invocations` on every structural occurrence
template: the ordered, duplicate-free compiler invocation IDs stamped in that
occurrence's own `template_html`. Each ID belongs to the enclosing structural
program and is validated against the detached compiler-rendered occurrence
fragment before attachment.

This is the required authority for nested creation. A parent materializer may
create only the listed child anchors, in program creation order, with the
parent opaque occurrence as `parent_scope` and the reconciler-issued local
occurrence unchanged. It must dispose children in reverse order before its own
bindings, DOM attachment, and records. No tag, selector, source, or arbitrary
live-DOM traversal may discover a child component.

Slot-projected nested hosts, Effects, cleanup, and resume remain unavailable.
