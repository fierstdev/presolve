#!/usr/bin/env bash
set -euo pipefail

cargo test -p presolve-cli workspace_commands --lib -- --nocapture
cargo test -p presolve-cli --test l9_cli_commands -- --nocapture
if awk '/^#\[cfg\(test\)\]/{exit} {print}' crates/ezc_cli/src/workspace_commands.rs | rg -n 'read_dir|WalkDir|glob::|parse_file|build_application_semantic_model|emit_production'; then
  echo 'L9-F must not discover packages/sources or invoke compiler internals' >&2
  exit 1
fi
./scripts/verify-l9e-cache-clean-contracts.sh
cargo fmt --all --check
cargo clippy -p presolve-cli --all-targets -- -D warnings
git diff --check
