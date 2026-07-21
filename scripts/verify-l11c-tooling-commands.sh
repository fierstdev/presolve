#!/usr/bin/env bash
set -euo pipefail

cargo test -p presolve-cli --test l9_cli_commands l11c_projects_only_validated_workspace_products -- --nocapture
rg --quiet 'run_l11_inspect' crates/ezc_cli/src/main.rs
rg --quiet 'run_l11_workspace_graph' crates/ezc_cli/src/main.rs
rg --quiet 'read_tooling_product_v1' crates/ezc_cli/src/main.rs
rg --quiet 'CLI tooling product views' docs/cli-tooling.md
./scripts/verify-l11b-tooling-product-readers.sh
cargo fmt --all --check
cargo clippy -p presolve-cli --all-targets -- -D warnings
git diff --check
