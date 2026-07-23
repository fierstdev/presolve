# Phase N N2-G compiler-registered Math rounding contract

N2-G admits exact one-argument `Math.floor(value)`, `Math.ceil(value)`, and
`Math.round(value)` calls in a supported `@computed()` getter. The operand must
already satisfy the compiler's numeric expression boundary. Member aliases,
overloads, extra arguments, generic Math dispatch, and arbitrary calls remain
unsupported.

The compiler recognizes only those exact source forms as registered builtin
operations, derives their existing operand dependencies, lowers `Floor`,
`Ceil`, or `Round` unary IR, and emits the corresponding compiler instruction.
The generated runtime checks that its operand is numeric and invokes only the
declared primitive; it never evaluates authored source.

N2-G introduced `computed.runtime.json` schema version `11`. N6-C14 later
advances the artifact to schema version `12` for compiler-owned Resource read
instructions and invalidation records; the rounding instruction contract is
retained unchanged.

Verification is `scripts/verify-n2g-builtin-math-rounding.sh`.
