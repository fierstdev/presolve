# Slots contract

`presolve_compiler::slot_projection` is schema v1 of the V2 slot ownership and
composition projection. It consumes the existing resolved slot-binding registry
without reinterpreting source TSX. A record preserves the caller instance, the
callee composition position, the lexical content-owner instance, and any exact
slot, fragment, outlet, or compiler-issued direct child identity.

The content owner remains the caller's lexical scope; projecting it to a callee
does not transfer that ownership. Bound, empty, and blocked states are carried
from canonical binding facts. Dynamic names, invalid wrappers, duplicate
content/outlets, and render-prop behavior remain owned by existing lowering and
are not recategorized here.

Slot capture and resumability are explicitly `unavailable` in this schema. The
projection makes no initialization, effect, DOM reconstruction, or resume claim
until a later capture analysis supplies authoritative coverage.
