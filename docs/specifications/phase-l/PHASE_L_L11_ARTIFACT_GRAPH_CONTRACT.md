# L11-E -- Production artifact-graph contract

**Status:** Authoritative implementation contract

**Prerequisites:** L10 registry; L11-A capability boundary; L11-D trace and structural compile-cost contract; frozen Phase K production chunk graph and runtime artifact.

**Next boundary:** L11-F producers, registry, and readers. This document authorizes no producer, decoder, public command, registry availability change, or persistence change.

## Scope

L11-E defines the future immutable, source-free `presolve.artifact-graph` v1 product. It is a compiler-produced record of the already validated Phase K production chunk topology and its same-build production runtime artifact. It is not an analysis of generated files and it does not alter chunk extraction, module emission, runtime loading, diagnostics, cache authority, or build output.

The product is absent and must remain `reserved` in the L10 registry until L11-F delivers its complete producer, strict reader, fixture, and compatibility proof.

## Exact provenance

A producer may create an artifact graph only from the same successful compilation invocation's validated `ProductionChunkGraph` and `ProductionRuntimeArtifactV1`. The graph must have passed `validate_production_chunk_graph`; the artifact must have passed `validate_production_runtime_artifact` against the graph's build identity. `build_id`, runtime-protocol version, optimization policy, eager chunk identity, chunk records, dependency records, and activation records must agree with those source products exactly.

The provenance is direct typed compiler data before build-output writing. A producer must not read, parse, glob, hash, or reconstruct the graph from `production/` modules, `production.runtime.json`, HTML, reports, output directories, a cache, or a later process. It must not manufacture a graph from an ASM inspection projection. The artifact graph is transient response/product data: it is not an L4 durable-session file, L5 baseline, L6 cache payload/key input, L7 manifest, L8 journal entry, or a new build artifact written by the normal compiler command.

## Canonical artifact graph v1

The document contains exactly `schema`, `version`, `graph_id`, `build_id`, `runtime_protocol_version`, `optimization_policy`, `artifact_checksum`, ordered `chunks`, ordered `dependencies`, and ordered `activations`.

`graph_id` is SHA-256 lowercase hexadecimal over the framed canonical document with `graph_id` omitted. `artifact_checksum` is the existing validated Phase K production-artifact checksum; it is not a new checksum of emitted files.

Each chunk contains exactly `chunk_id`, `kind`, `module_filename`, `activation_roots`, nullable `root_kind`, `program_fingerprints`, and `registration_only`. These are direct facts from one `ProductionChunkRecord`. Chunks are sorted by canonical chunk ID; every nested identity list is sorted and unique. `kind` is exactly one of `eager`, `root`, or `shared` and retains the frozen K7 topology meaning. `module_filename` is the compiler-derived production module name, never an input, host, or source path.

Each dependency contains exactly `dependent_chunk_id` and `dependency_chunk_id`, sorted by that ordered pair and unique. Each activation contains exactly `activation_root_id`, `root_chunk_id`, and `shared_chunk_ids`, sorted by activation root with sorted unique shared chunk IDs. The single eager chunk, root-to-eager/shared dependencies, registration-only shared chunks, and activation reciprocity must satisfy the existing Phase K graph validator; v1 adds no new chunk kind, topology, dependency depth, loading policy, or shared-extraction heuristic.

The graph contains no authored source text, source paths, input filenames, snippets, AST/parser products, semantic entities, source spans, timestamps, durations, host/process/memory measurements, file-system metadata, module contents, generated JavaScript bytes, runtime-table contents, report counters, diagnostics, cache state, or benchmark data.

## Producer, reader, and error boundary

L11-F must implement this product atomically with the L11-D trace and cost products. It must add a canonical encoder, strict decoder, graph-ID recomputation, source-product provenance validation, schema/version rejection, source-free fixtures, reverse-input determinism, and L3--L8/Phase K byte-preservation evidence before transitioning `presolve.artifact-graph` from L10 `reserved` to `available`. A reader uses the L11 explicit product-file boundary only; it never invokes compilation or inspects a build directory.

L11-F reserves `L11T010` for invalid artifact-graph provenance, `L11T011` for artifact/graph topology disagreement, and `L11T012` for noncanonical or incompatible artifact-graph bytes. They use CLI exit code 6 and never become compiler diagnostics.

## Verification and stop rules

L11-E verifies that this contract is indexed and that L10 still keeps the schema reserved. It makes no Rust implementation changes. L11-F starts only after this contract is committed.

Stop rather than extend this contract if a requested field cannot be obtained directly from the validated Phase K graph and same-build artifact; if it requires parsing generated bytes or build output; or if it requires source, timing, host, cache, or lifecycle facts. Such a field requires a separate immutable-product or constitutional amendment.
