#!/usr/bin/env bash
set -euo pipefail

products=crates/ezc_core/src/tooling_products.rs
fixture=crates/ezc_core/fixtures/tooling/query-snapshot-v1.json

rg --quiet 'QUERY_SNAPSHOT_TOOLING_SCHEMA_V1' "$products"
rg --quiet 'build_tooling_query_snapshot_v1' "$products"
rg --quiet 'decode_tooling_query_snapshot_v1' "$products"
rg --quiet 'query-semantic-v1' "$products"
test -s "$fixture"
if rg --quiet 'src/|QueryFixture|x-query-fixture' "$fixture"; then
  echo 'L12-C query-snapshot fixture must remain source-free' >&2
  exit 1
fi
cargo test -p presolve-compiler platform --lib -- --nocapture
cargo test -p presolve-compiler tooling_products --lib -- --nocapture
cargo test -p presolve-compiler tooling_schema --lib -- --nocapture
cargo clippy -p presolve-compiler --all-targets -- -D warnings
./scripts/verify-l12b-query-snapshot-amendment.sh
git diff --check
