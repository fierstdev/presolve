#!/usr/bin/env bash
set -euo pipefail

cargo test -p presolve-cli build_check_commands --lib -- --nocapture
cargo test -p presolve-cli --test l9_cli_commands -- --nocapture
cargo check -p presolve-cli
rg --quiet 'run_explicit_build_or_check_v1' crates/presolve_cli/src/main.rs
if rg -n 'read_dir|WalkDir|glob::|parse_file|build_application_semantic_model|generate_|emit_production' crates/presolve_cli/src/build_check_commands.rs; then
  echo 'L9-D must not discover source membership or invoke compiler internals' >&2
  exit 1
fi
./scripts/verify-l9c-compilation-adapter-contracts.sh
cargo fmt --all --check
cargo clippy -p presolve-cli --all-targets -- -D warnings
git diff --check
