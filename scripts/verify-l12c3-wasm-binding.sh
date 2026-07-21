#!/usr/bin/env bash
set -euo pipefail

binding=crates/ezc_core/src/wasm_binding.rs
package=packages/compiler-wasm/package.json
test -s "$binding"
test -s "$package"
rg --quiet 'wasm_bindgen' "$binding"
rg --quiet 'query_snapshot_v1' "$binding"
rg --quiet '@presolve/compiler-wasm' "$package"
if rg --quiet 'decode_tooling_query_snapshot_v1|serde_json|std::fs|fetch|http|Path|URI|uri|sourceText' packages/compiler-wasm/package.json; then
  echo 'L12-C-3 package surface must not decode or host the compiler product' >&2
  exit 1
fi
./scripts/build-l12c-compiler-wasm.sh
node packages/compiler-wasm/test/smoke.mjs
cargo test -p presolve-compiler language_service --lib -- --nocapture
cargo clippy -p presolve-compiler --all-targets -- -D warnings
./scripts/verify-l12c2-rust-query-projection.sh
git diff --check
