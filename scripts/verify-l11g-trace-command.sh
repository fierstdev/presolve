#!/usr/bin/env bash
set -euo pipefail

cargo test -p presolve-cli --test l9_cli_commands l11g_trace_projects_only_a_validated_explicit_product -- --nocapture
rg --quiet 'fn run_l11_trace' crates/presolve_cli/src/main.rs
rg --quiet 'presolve trace' docs/cli-tooling.md
./scripts/verify-l11f-tooling-products.sh
git diff --check
