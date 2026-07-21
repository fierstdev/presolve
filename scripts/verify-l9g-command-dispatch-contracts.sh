#!/usr/bin/env bash
set -euo pipefail

cargo test -p presolve-cli --test l9_cli_commands -- --nocapture
rg --quiet 'l9_reserved_command' crates/ezc_cli/src/main.rs
rg --quiet '"version"' crates/ezc_cli/src/main.rs
./scripts/verify-l9f-workspace-contracts.sh
cargo fmt --all --check
cargo clippy -p presolve-cli --all-targets -- -D warnings
git diff --check
