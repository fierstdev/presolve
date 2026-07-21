# PHASE_L_L9_RECOVERY_AND_IMPLEMENTATION_CONTRACT.md

**Status:** Authoritative implementation contract
**Phase:** L
**Slice:** L9 — CLI Platform
**Prerequisites:** L1–L8 accepted and committed through `7d53d09`
**Authority:** Reconstructs the missing implementation-ready L9 and L9-A through L9-A.2 documents from the preserved Phase L constitution, package/CLI specification, verification/release specification, and L9-A.3 amendment.
**Supersedes:** The abbreviated L9 roadmap where this contract is more specific.
**Next boundary:** L10

## 1. Recovery decision

L1–L8 are accepted platform foundation work and shall not be restarted, rewritten, or repartitioned. L9 is the first unstarted slice.

The historic implementation-ready L9 and L9-A through L9-A.2 documents are not present in the repository or supplied contract set. This document is the tracked replacement authority. It is deliberately explicit so L9 work does not infer behavior from the earlier heading-only roadmap.

`PHASE_L_L9_A_3_CONSTRUCTION_BASED_CODEC_PROOF_AMENDMENT.md` remains authoritative over this contract wherever both discuss configuration-codec proof.

## 2. Frozen inputs and ownership

L3 owns `WorkspaceConfiguration`, validation, canonical configuration identity, and its internal canonical serializer. L4 owns durable sessions, L5 owns ephemeral incremental reuse, L6 owns persistent complete-result cache entries, L7 owns serial whole-workspace compilation, and L8 owns caller-driven, process-local watch orchestration.

L9 adds a public process boundary only. It may read caller-authorized project files and observe files for public CLI operations. It shall construct complete requests and invoke L4/L7/L8; it shall not parse, bind, analyze, optimize, or generate artifacts independently. It shall not alter generated output, diagnostics, product identities, service scheduling, L5 reuse, or L6 cache semantics.

The L8 compiler-service boundary remains unchanged: the CLI observer supplies a complete replacement L7 candidate for every watch batch. No filesystem discovery enters the compiler service.

## 3. Public CLI contract

The executable remains `presolve`. Every operation supports deterministic human output; operations yielding compiler products additionally support canonical JSON. Stable exit codes are: success `0`, compilation failure `1`, configuration error `2`, workspace error `3`, compiler internal error `4`, cache error `5`, tooling error `6`, and unexpected platform error `7`.

L9 shall provide these command families in order: (1) `version`, help, common formatting, structured errors, and dispatch; (2) `build`, `check`, and `clean` through complete service/workspace requests; (3) `workspace` and `cache` as L7/L6 projections; (4) `watch` as a filesystem observer adapting to L8, `dev` as its documented development-server adapter, and `create` as deterministic scaffold generation; and (5) `explain`, `inspect`, `graph`, `trace`, `profile`, `benchmark`, and `doctor` as immutable compiler/service-product projections.

No command may claim support before its fixtures, help, exit-code matrix, human/JSON output, and documentation are committed. L9 does not introduce an IDE protocol, remote cache, distributed build, plugin system, or alternate compiler.

## 4. CLI configuration codec

The public `presolve.json` configuration is distinct from L3 internal durable configuration JSON. It is an object with exactly these required fields, in this canonical encoder order:

1. `source_roots`: non-empty array of normalized logical workspace-path strings;
2. `feature_flags`: array of unique strings in lexicographic order;
3. `target_profile`: exactly one of `default`, `development`, or `production`;
4. `platform_options`: array of two-string `[key, value]` tuples, unique by key and ordered lexicographically by `(key, value)`.

The decoder rejects unknown/missing fields, duplicate object keys, aliases, defaults, invalid JSON/object/type/path/tuple/key/value, duplicate roots, flags, or option keys, unsupported target-profile spelling, object-shaped platform options, and non-canonical flag/option ordering. It opens no path and discovers no source; project loading is a separate CLI operation.

The encoder emits all four fields, uses no aliases/default elision, preserves the semantically significant L3 source-root order, and emits canonical flags and options. A Rust input whose flag or option collections are not canonical is normalized by the CLI adapter before L3 validation; this is strict representation normalization, not a new compiler semantic authority.

The CLI codec API is:

```rust
pub fn decode_cli_workspace_configuration_v1(value: &serde_json::Value) -> Result<WorkspaceConfiguration, CliWorkspaceConfigurationDecodeError>;
pub fn encode_cli_workspace_configuration_v1(configuration: &WorkspaceConfiguration) -> Result<serde_json::Value, CliWorkspaceConfigurationEncodeError>;
```

Optional byte decoding is allowed only as a strict JSON front end to the same decoder. Typed errors are CLI/tooling errors and never compiler diagnostics.

## 5. L9-A.3 proof and representation isolation

L9 shall not add a public L3 configuration decoder, durable decoder errors, migration behavior, or a cross-codec decoder. The existing L3 serializer is byte-for-byte frozen and is neither a public authoring format nor a CLI parser. Durable L4/L7 paths shall not import or invoke the CLI codec.

For representative constructed normal Rust configurations, tests shall use accessible L3 validation; serialize with the existing L3 serializer and compare frozen bytes; encode then decode the strict CLI representation; compare decoded and normalized Rust values; derive and compare the existing L3 configuration identity; compare CLI JSON to its fixture; and prove the L3 and CLI serialized shapes intentionally differ.

The L9 verifier must run twenty shuffled CLI object-input repetitions, strict shape rejection, durable-codec isolation, existing L4/L7 restart tests, and all L3–L8 audits. It must not require or test `decode_l3_canonical`, durable codec round trips, or byte equality between L3 and CLI representations.

## 6. Execution slices and completion gate

L9-A implements this codec and construction-based proof. L9-B through L9-G implement the command families in section 3, with one atomic commit each. The final L9-G gate runs command help/exit/error/JSON/human fixture matrices, clean-project installation and creation, L3–L8 audits, formatter, strict lint, artifact parity, browser/runtime tests where CLI execution reaches them, determinism repetitions, `just check`, and `git diff --check`.

L9 is complete only after every listed command is documented, deterministic, and a compiler-product consumer; its verification script is in `just check`; the progress log and handoff contain exact commands/results; and the worktree is clean. Stop before L10.
