# Public CLI Configuration Codec

L9-A defines the strict public `presolve.json` representation. It has exactly four required fields: `source_roots`, `feature_flags`, `target_profile`, and tuple-shaped `platform_options`. This JSON is an authoring format for the CLI; it is not the internal L3 workspace-configuration serializer.

The decoder performs structural validation only, then constructs the existing `WorkspaceConfiguration` and invokes L3 validation and identity APIs. It never opens paths or discovers source files. The encoder emits canonical CLI JSON; source-root order is preserved, while feature flags and platform options are canonicalized.

L3's serializer remains frozen, internal, and source-free. L9 exposes no L3 configuration decoder, durable migration, or cross-codec byte-equality claim. Codec correctness is proven from constructed Rust configurations: unchanged L3 fixture bytes, CLI encode/decode value equality, equal L3 configuration identity, and distinct serialized shapes. Durable L4/L7 state never imports this CLI codec.
