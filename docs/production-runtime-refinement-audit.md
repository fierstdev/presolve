# Production Runtime Refinement Audit

Phase K20 audits the implemented production path without changing its public
products. The compiler has one named authority for each responsibility:

| Responsibility | Authority |
|---|---|
| policy | `ProductionOptimizationPolicyV1` |
| fingerprints | `ExecutableProgramFingerprint::for_canonical_opcode_stream` |
| reachability | `build_production_reachability_graph` |
| ordinal tables | `build_production_runtime_table` |
| emitter/minifier | `emit_production_modules` |
| artifact validation | `validate_production_runtime_pipeline` |
| lifecycle cleanup | `build_production_destroy_plan` |
| scheduling/coalescing | `build_production_patch_schedule` |

The production artifact validates canonical string identities at the trust
boundary, then exposes dense ordinal tables to bootstrap/scheduler products.
Generated production modules contain no source provenance, development branch,
dynamic source evaluation, or source comments. Their only retained string
identity is the compiler-emitted chunk registration identity.

The runtime has one definition and one boot-path invocation for each dedicated
listener/registry installer. Generated runtime helpers are statically checked
for a use beyond their declaration. Exact-owner cleanup is reverse ordered and
the 100-cycle fixture proves instance registries return to baseline while global
module/program caches remain constant.

The K16 budgets remain normative. K20 does not change semantic graph v6,
template manifest v4, component artifact v3, Context artifact v2, Forms/Effect
artifact v1, resume manifest v6, snapshot/protocol/registry v1, production
artifact v1, or either report v1. Development/production parity and deterministic
bytes remain covered by the K19 production fixture matrix.
