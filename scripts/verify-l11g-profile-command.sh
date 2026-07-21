#!/usr/bin/env bash
set -euo pipefail

cargo test -p presolve-cli --test l9_cli_commands l11g_profile_projects_only_a_validated_explicit_product -- --nocapture
rg --quiet 'fn run_l11_profile' crates/ezc_cli/src/main.rs
rg --quiet 'presolve profile' docs/cli-tooling.md
./scripts/verify-l11g-trace-command.sh
git diff --check
