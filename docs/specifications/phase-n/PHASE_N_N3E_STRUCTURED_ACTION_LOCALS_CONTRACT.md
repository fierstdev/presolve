# Phase N N3-E structured serializable Action locals contract

N3-E extends the N3-D compiler-resolved local form to recursively serializable
record and array literals. A local record such as
const next = { name: "Locked", roles: ["writer", "admin"] }; may replace a
complete State field in an @action() method.

The compiler compares the local literal against the state(...) initializer:
object keys must match exactly, primitive leaves must match, and non-empty
arrays must be homogeneous with compatible element shapes. The accepted local
lowers to the existing complete-field assign operand. Generated runtime neither
executes the local declaration nor keeps an alias to the record.

PSC1045 rejects a local without @action() or an incompatible/unknown structured
boundary. Spreads, State reads, computed calls, mutation through a local alias,
partial/nested writes, heterogeneous arrays, and structural TypeScript checker
semantics remain outside the admission.
