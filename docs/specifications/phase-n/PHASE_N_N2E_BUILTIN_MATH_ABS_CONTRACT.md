# Phase N N2-E compiler-registered Math.abs contract

`Math.abs(value)` is admitted only in a supported Computed getter with exactly
one compiler-supported numeric operand. The compiler rewrites that exact source
form to `BuiltinPureOperation::MathAbs`, preserves the operand dependency,
lowers canonical unary `Abs` IR, and emits schema-v9 runtime metadata. The
generated runtime evaluates only the `abs` instruction. Other calls, overloads,
callbacks, package code, and dynamic callee selection remain unsupported.
