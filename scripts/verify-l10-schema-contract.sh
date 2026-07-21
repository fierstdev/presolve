#!/usr/bin/env bash
set -euo pipefail

contract=docs/specifications/phase-l/PHASE_L_L10_TOOLING_SCHEMA_IMPLEMENTATION_CONTRACT.md
rg --quiet 'Canonical schema registry v1' "$contract"
rg --quiet 'Negotiation v1' "$contract"
rg --quiet 'presolve.build-trace' "$contract"
rg --quiet 'byte-for-byte unchanged' "$contract"
rg --quiet 'PHASE_L_L10_TOOLING_SCHEMA_IMPLEMENTATION_CONTRACT' docs/specifications/phase-l/README.md
cargo test -p presolve-compiler tooling_schema --lib -- --nocapture
if rg -n 'use crate::(service|persistent_cache|workspace|watch)' crates/ezc_core/src/tooling_schema.rs; then
  echo 'L10 negotiation must remain independent from L3-L8 execution and durable products' >&2
  exit 1
fi
cargo fmt --all --check
cargo clippy -p presolve-compiler --all-targets -- -D warnings
git diff --check
