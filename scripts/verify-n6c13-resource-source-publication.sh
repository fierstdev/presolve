#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N6C13_RESOURCE_SOURCE_PUBLICATION_CONTRACT.md
cargo test -q -p presolve-compiler --lib tests::component_graph_retains_resource_declaration_facts_before_package_resolution -- --exact
cargo test -q -p presolve-compiler --lib tests::resolves_resource_source_designator_through_integrity_checked_package_contract -- --exact
cargo test -q -p presolve-compiler --lib semantic_capability::tests::registry_is_versioned_stable_and_explains_deferred_families -- --exact
RUST_TEST_THREADS=1 cargo test -q -p presolve-cli --test runtime_browser host_bound_resource_endpoint_activates_in_a_real_browser -- --exact
cargo check -q -p presolve-compiler -p presolve-cli
cargo fmt --all --check
git diff --check
