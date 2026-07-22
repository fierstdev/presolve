# Presolve public testing contract

**Status:** L15-A authoritative inventory. This maps existing test authority.

| Layer | Existing authority | Public purpose | Local reproduction | Assertion |
| --- | --- | --- | --- | --- |
| Compiler/platform | `crates/ezc_core` tests; `verify-l3` through `verify-l8` | products and lifecycle | `cargo test -p presolve-compiler --lib` | canonical bytes |
| Tooling/products | `fixtures/tooling*`; `verify-l10` through `verify-l12c` | schemas and projections | relevant verifier | JSON, SHA, order |
| CLI | `l9_cli_commands`; L9 verifiers | explicit command contracts | CLI test command | output and exit |
| Runtime/browser | `runtime_browser`; H--K fixtures | generated runtime | `pnpm test:e2e` | browser result |
| Editor packages | L12 smoke fixtures | product-only queries | L12 verifier chain | response hashes |
| Repository/public | layout/identity/spec verifiers | public boundary | `just repository-layout` | paths and identity |

Compiler fixtures remain in their owning Rust crate and CLI fixtures remain in `crates/ezc_cli`. Package fixtures may consume but never copy, rewrite, or decode them. SHA fixtures are canonical output commitments, not performance baselines. Browser probes require Chrome and are a separate runtime lane.

Deterministic contracts are correctness gates. Browser/runtime is a correctness gate where an existing fixture applies. Examples become a gate only after L14. Observation work is report-only: host time, memory, CPU, or machine identity can never affect a pass/fail result. L15 utilities must delegate to existing commands and cannot recreate compiler semantics.
