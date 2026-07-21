#!/usr/bin/env bash
set -euo pipefail

cargo test -p presolve-cli command_framework --lib -- --nocapture
rg --quiet 'CliExitCodeV1' crates/ezc_cli/src/command_framework.rs
rg --quiet 'load_explicit_project_envelope_v1' crates/ezc_cli/src/command_framework.rs
if rg -n 'fs::read_dir|fs::read_to_string|glob::|WalkDir|symlink_metadata|env::current_dir' crates/ezc_cli/src/command_framework.rs; then
  echo 'L9-B project envelope must not discover a project or sources' >&2
  exit 1
fi
./scripts/verify-l9a1-configuration-codec-contracts.sh
cargo fmt --all --check
cargo clippy -p presolve-cli --all-targets -- -D warnings
git diff --check
