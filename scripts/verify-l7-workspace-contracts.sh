#!/usr/bin/env bash
set -euo pipefail
test -f docs/workspace-architecture-contract.md
test -s crates/ezc_core/fixtures/workspace/chain-v1.json
test -s crates/ezc_core/fixtures/workspace/cycle-v1.json
rg --quiet 'presolve.workspace-manifest' crates/ezc_core/src/workspace.rs
rg --quiet 'compile_workspace_v1' crates/ezc_core/src/service.rs
cargo test -p presolve-compiler --lib l7_ -- --nocapture
cargo fmt --all --check
cargo clippy -p presolve-compiler --all-targets --all-features -- -D warnings
./scripts/verify-l3-platform-contracts.sh
./scripts/verify-l4-service-contracts.sh
./scripts/verify-l5-incremental-contracts.sh
./scripts/verify-l6-persistent-cache-contracts.sh
git diff --check
