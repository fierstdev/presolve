# Source maps contract

Vite generates physical production source maps. `@presolve/vite` enables them
for every `buildPresolveProduction` result and returns each emitted map path
with a schema v1 translation product.

The translation product associates only a retained Vite source containing
`virtual:presolve/v1/<artifact-path>` with the exact digest-bound compiler
artifact path in the application-publication manifest. It does not decode VLQ
mappings, choose an authored file, or synthesize a line or column. Vite may
legitimately omit `sources` for a fully compacted generated wrapper; that map
remains a physical packaging map with an empty translation set.

Compiler source provenance and `presolve explain` remain the authority for
authored locations. A source-map consumer must combine a Vite mapping with an
explicit compiler provenance product before presenting an authored location.
Unknown Vite sources are retained without a compiler artifact association.
