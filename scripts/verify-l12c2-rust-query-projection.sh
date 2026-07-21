#!/usr/bin/env bash
set -euo pipefail

module=crates/ezc_core/src/language_service.rs
test -s "$module"
rg --quiet 'decode_tooling_query_snapshot_v1' "$module"
rg --quiet 'query_snapshot_v1' "$module"
rg --quiet 'unknown_source_unit' "$module"
rg --quiet 'offset_out_of_range' "$module"
rg --quiet 'unknown_query_semantic_id' "$module"
rg --quiet 'Unsupported' "$module"
if rg --quiet 'std::fs|std::net|std::time|Command|PathBuf|URI|uri|source_text' "$module"; then
  echo 'L12-C-2 query projection must remain host and source free' >&2
  exit 1
fi
cargo test -p presolve-compiler language_service --lib -- --nocapture
cargo clippy -p presolve-compiler --all-targets -- -D warnings
./scripts/verify-l12c-wasm-binding-contract.sh
git diff --check
