#!/usr/bin/env bash

set -euo pipefail

readonly platform_source='crates/presolve_compiler/src/platform.rs'
readonly fixture_dir='crates/presolve_compiler/fixtures/platform'

test -f "$platform_source"
test -f docs/compiler-platform-contract.md

for constant in WORKSPACE_GRAPH_SCHEMA_VERSION WORKSPACE_SNAPSHOT_SCHEMA_VERSION COMPILER_SESSION_SCHEMA_VERSION INCREMENTAL_PLAN_SCHEMA_VERSION PRODUCT_CACHE_INSPECTION_SCHEMA_VERSION; do
  test "$(rg --fixed-strings "$constant" "$platform_source" | wc -l | tr -d ' ')" -ge 1
done

for fixture in "$fixture_dir"/*-v1.json; do
  test -s "$fixture"
  test "$(tail -c 1 "$fixture")" = ''
  ! rg --quiet '(^|[^a-z])(/Users/|C:\\)' "$fixture"
done

# Phase L3 is an adapter over the established parser and application model.
rg --quiet 'presolve_parser::parse_file' "$platform_source"
rg --quiet 'build_application_semantic_model_for_unit' "$platform_source"
rg --quiet 'clean_full_equivalence_for_incremental_workspace_changes' "$platform_source"
! rg --quiet 'TcpListener|UnixListener|std::net|tokio|sled|rusqlite|cache file' "$platform_source"
! rg --quiet 'SystemTime|Instant|timestamp|absolute host path' "$platform_source"
rg --quiet 'pub mod platform' crates/presolve_compiler/src/lib.rs
