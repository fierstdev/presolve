#!/usr/bin/env bash
set -euo pipefail
repo_root="$(git rev-parse --show-toplevel)"; cd "$repo_root"
test -s docs/specifications/phase-n/PHASE_N_N4B_JSX_HTML_ATTRIBUTE_ALIAS_CONTRACT.md
rg --fixed-strings --quiet 'className' docs/specifications/phase-n/PHASE_N_N4B_JSX_HTML_ATTRIBUTE_ALIAS_CONTRACT.md
cargo test -q -p presolve-compiler --lib lowers_jsx_html_attribute_aliases_before_html_and_manifest_generation
cargo test -q -p presolve-compiler --lib normalizes_jsx_html_attribute_aliases
cargo test -q -p presolve-compiler registry_is_versioned_stable_and_explains_deferred_families --lib
cargo check -q -p presolve-compiler
cargo fmt --all --check
git diff --check
