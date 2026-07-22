#!/usr/bin/env bash
set -euo pipefail

readonly cache_source='crates/presolve_compiler/src/persistent_cache.rs'
readonly service_source='crates/presolve_compiler/src/service.rs'
readonly fixture_dir='crates/presolve_compiler/fixtures/persistent-cache'

test -f docs/persistent-cache-contract.md
for fixture in "$fixture_dir"/*.json; do test -s "$fixture"; test "$(tail -c 1 "$fixture")" = ''; done
for schema in presolve.persistent-artifact-cache.v1 presolve.cache-manifest.v1 presolve.cache-entry-envelope.v1 presolve.cache-inspection-report.v1; do rg --quiet "$schema" "$cache_source"; done
rg --quiet 'start_with_cache' "$service_source"
rg --quiet 'l6_persistent_complete_result_cache_hits_after_restart' "$service_source"
rg --quiet 'l6_corruption_and_disabled_cache_fall_back' "$service_source"
! rg --quiet 'presolve_parser::|read_to_string|read_to_end|SystemTime|Instant' "$cache_source"
cargo test -p presolve-compiler --lib l6_ -- --nocapture
cargo fmt --all --check
cargo clippy -p presolve-compiler --all-targets --all-features -- -D warnings
./scripts/verify-l3-platform-contracts.sh
./scripts/verify-l4-service-contracts.sh
./scripts/verify-l5-incremental-contracts.sh
git diff --check
