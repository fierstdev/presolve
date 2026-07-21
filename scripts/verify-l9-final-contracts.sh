#!/usr/bin/env bash
set -euo pipefail

cargo test -p presolve-cli --test l9_cli_commands -- --nocapture
rg --quiet 'run_explicit_watch_once_v1' crates/ezc_cli/src/main.rs
if awk '/^#\[cfg\(test\)\]/{exit} {print}' crates/ezc_cli/src/workspace_commands.rs | rg -n 'read_dir|WalkDir|glob::|parse_file|build_application_semantic_model|emit_production'; then
  echo 'L9 watch/workspace adapter must not discover or compile independently' >&2
  exit 1
fi
./scripts/verify-l9g-command-dispatch-contracts.sh
cargo fmt --all --check
cargo clippy -p presolve-cli --all-targets -- -D warnings
git diff --check
