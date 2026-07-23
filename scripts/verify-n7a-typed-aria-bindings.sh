#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N7A_TYPED_ARIA_BINDING_CONTRACT.md
cargo test -q -p presolve-compiler --lib semantic_type::tests::defines_typed_accessibility_attribute_bindings -- --exact
cargo test -q -p presolve-compiler --lib compiler_pass::tests::rejects_string_binding_for_boolean_aria_attribute -- --exact
RUST_TEST_THREADS=1 cargo test -q -p presolve-cli --test runtime_browser typed_aria_attribute_updates_in_a_real_browser -- --exact
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check
