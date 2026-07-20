#!/usr/bin/env bash

set -euo pipefail

readonly platform_source='crates/ezc_core/src/platform.rs'
readonly service_source='crates/ezc_core/src/service.rs'
readonly fixture_dir='crates/ezc_core/fixtures/incremental'

test -f docs/incremental-compilation-contract.md
test -f "$fixture_dir/plan-v1.json"
test -f "$fixture_dir/report-v1.json"
for fixture in "$fixture_dir"/*.json; do
  test -s "$fixture"
  test "$(tail -c 1 "$fixture")" = ''
done

rg --quiet 'INCREMENTAL_COMPILATION_PLAN_V1_SCHEMA' "$platform_source"
rg --quiet 'INCREMENTAL_EXECUTION_REPORT_V1_SCHEMA' "$platform_source"
rg --quiet 'plan_incremental_compilation_v1' "$platform_source"
rg --quiet 'compile_workspace_incremental_v1' "$platform_source"
rg --quiet 'IncrementalCompilationPlanV1' "$platform_source"
rg --quiet 'IncrementalExecutionReportV1' "$service_source"
rg --quiet 'L5F009_SOURCE_UNIVERSE_MEMBERSHIP_UNMODELED' "$platform_source"
rg --quiet 'L5F002_CONFIGURATION_CHANGED' "$platform_source"
rg --quiet 'l5_content_edit_reuses_validated_parse_products_and_equals_clean' "$service_source"
rg --quiet 'twenty_run_determinism' "$service_source"

# The service remains a boundary/orchestrator: no source discovery and no
# alternate parser/binder/semantic implementation is permitted here.
! rg --quiet 'read_to_string|read_to_end|read_dir|ezc_parser::|build_application_semantic_model' "$service_source"
! rg --quiet 'SystemTime|Instant|modified\(' "$platform_source" "$service_source"
! rg --quiet 'source.*session\.json|source.*journal|source.*commit\.json' "$service_source"

cargo test -p presolve-compiler --lib l5_ -- --nocapture
cargo fmt --all --check
cargo clippy -p presolve-compiler --all-targets --all-features -- -D warnings
./scripts/verify-l3-platform-contracts.sh
./scripts/verify-l4-service-contracts.sh
git diff --check
