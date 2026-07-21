#!/usr/bin/env bash
set -euo pipefail

reader=crates/ezc_core/src/tooling_reader.rs

cargo test -p presolve-compiler tooling_reader --lib -- --nocapture
rg --quiet 'read_tooling_product_v1' "$reader"
rg --quiet 'decode_workspace_snapshot_json_v1' "$reader"
rg --quiet 'decode_workspace_graph_json_v1' "$reader"
if awk '/^#\[cfg\(test\)\]/{exit} {print}' "$reader" | rg -n 'std::fs|read_to_(end|string)|File::|parse_file|CompilerServiceHost|PersistentArtifactCache|WatchSession|workspace::'; then
  echo 'L11-B readers must consume supplied bytes without execution, persistence, or source access' >&2
  exit 1
fi
if rg -n 'tooling_reader' crates/ezc_core/src/{service,persistent_cache,workspace,watch}.rs crates/ezc_cli/src; then
  echo 'L3-L8 execution and CLI command dispatch must not depend on L11-B readers' >&2
  exit 1
fi
./scripts/verify-l11a-tooling-capability-contract.sh
cargo fmt --all --check
cargo clippy -p presolve-compiler --all-targets -- -D warnings
git diff --check
