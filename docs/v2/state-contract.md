# State contract

`presolve_compiler::state_projection` is schema v1 of the V2 State inspection
product. It projects existing instance-qualified storage records, preserving
their stable state, storage, component-instance, and declaration-order-derived
initialization identities. It does not make state static or alter JavaScript
field initialization order.

Resume admission is `codec_backed` only when the existing closed resume codec
accepts the canonical semantic type; otherwise it is rejected at the State
declaration boundary. Update coverage delegates to existing canonical lowering:
this projection does not infer deep mutation tracking or introduce a second
invalidation runtime.
