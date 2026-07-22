#!/usr/bin/env bash
set -euo pipefail

cargo test -p presolve-cli compilation_commands --lib -- --nocapture
rg --quiet 'CompilerServiceHost::start' crates/presolve_cli/src/compilation_commands.rs
rg --quiet 'WorkspaceSnapshot::from_input' crates/presolve_cli/src/compilation_commands.rs
if awk '/^#\[cfg\(test\)\]/{exit} {print}' crates/presolve_cli/src/compilation_commands.rs | rg -n 'parse_file|build_application_semantic_model|generate_|emit_production|std::fs::read'; then
  echo 'L9-C must delegate compilation without source loading, parsing, or code generation' >&2
  exit 1
fi
./scripts/verify-l9b-command-framework-contracts.sh
cargo fmt --all --check
cargo clippy -p presolve-cli --all-targets -- -D warnings
git diff --check
