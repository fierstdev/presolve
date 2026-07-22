#!/usr/bin/env bash
set -euo pipefail

cargo test -p presolve-cli cache_commands --lib -- --nocapture
cargo test -p presolve-cli --test l9_cli_commands -- --nocapture
rg --quiet 'run_project_cache_operation_v1' crates/presolve_cli/src/main.rs
if awk '/^#\[cfg\(test\)\]/{exit} {print}' crates/presolve_cli/src/cache_commands.rs | rg -n 'read_dir|WalkDir|glob::|parse_file|build_application_semantic_model|remove_dir_all'; then
  echo 'L9-E must delegate cache ownership to L6 without source discovery or independent deletion' >&2
  exit 1
fi
./scripts/verify-l9d-build-check-contracts.sh
cargo fmt --all --check
cargo clippy -p presolve-cli --all-targets -- -D warnings
git diff --check
