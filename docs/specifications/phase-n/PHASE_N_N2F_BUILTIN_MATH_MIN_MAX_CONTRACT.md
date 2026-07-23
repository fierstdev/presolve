# Phase N N2-F compiler-registered Math.min and Math.max contract

N2-F admits only exact two-operand `Math.min(left, right)` and
`Math.max(left, right)` calls in a supported `@computed()` getter. Each operand
must already be a compiler-supported numeric expression; no overload
resolution, rest arguments, callbacks, member aliases, dynamic callee, or
generic `Math` access is admitted.

The compiler resolves these spellings before package resolution, records a
closed `BuiltinPureOperation`, derives dependencies from both operands, lowers
canonical `Min`/`Max` binary IR, and emits `min`/`max` runtime-computed
instructions. The generated runtime applies only those compiler-emitted
instructions and returns `undefined` for a malformed non-numeric runtime
operand; it never evaluates the authored call.

This advances `computed.runtime.json` to schema version `10`. Schema v10 is
exact-match: a runtime accepting v10 accepts the pre-existing binary
instructions plus `min` and `max`, and rejects every other schema version.

The browser fixture proves initial computed values `-2` and `5` from compiler
generated artifacts. Arbitrary methods, `Math.min(...values)`, `Math.max` with
more or fewer operands, `Math.round`, date helpers, and collection callbacks
remain outside this slice.
