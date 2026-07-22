#!/usr/bin/env bash
set -euo pipefail

products=crates/presolve_compiler/src/tooling_products.rs
rg --quiet 'build_tooling_build_trace_v1' "$products"
rg --quiet 'build_tooling_compile_cost_report_v1' "$products"
rg --quiet 'build_tooling_artifact_graph_v1' "$products"
rg --quiet 'decode_tooling_artifact_graph_v1' "$products"
cargo test -p presolve-compiler tooling_products --lib -- --nocapture
cargo test -p presolve-compiler tooling_reader --lib -- --nocapture
./scripts/verify-l11e-artifact-graph-contract.sh
git diff --check
