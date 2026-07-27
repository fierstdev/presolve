# Computed contract

`presolve_compiler::computed_projection` is schema v1 of the V2 computed
inspection product. It projects existing canonical runtime-computed records:
inferred dependencies, memoized cache slots, and dirty flags. Per-instance cache
qualification, purity diagnostics, cycles, and conditional dependencies remain
owned by the existing computed and runtime lowering products.
