# Purity and effect contract

`PurityEffectGraphV1` classifies function summaries using only compiler-owned
CFG and call-coverage evidence. Storage writes, Context-slot writes, and
observable IR instructions are impure. Resource reads, explicit unknown calls,
and unavailable call coverage are conservative unknowns. Impurity takes
precedence when a function has both an observed effect and incomplete evidence.

No source spelling, Vite module, or unrepresented call can make a function
pure. The product is a new analysis authority and does not alter the existing
computed/effect products until those slices explicitly adopt it.
