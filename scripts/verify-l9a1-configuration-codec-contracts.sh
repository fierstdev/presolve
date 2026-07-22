#!/usr/bin/env bash
set -euo pipefail

cargo test -p presolve-cli configuration_codec --lib -- --nocapture
for fixture in crates/presolve_cli/fixtures/configuration/*-v1.json; do
  test -s "$fixture"
  test "$(tail -c 1 "$fixture")" = "$(printf '\n')"
done
rg --quiet 'L9C017_DUPLICATE_OBJECT_KEY' crates/presolve_cli/src/configuration_codec.rs
rg --quiet 'decode_cli_workspace_configuration_bytes_v1' crates/presolve_cli/src/configuration_codec.rs
if rg -n 'decode_l3_canonical|pub fn decode_(l3_)?canonical_workspace_configuration|pub fn decode_workspace_configuration_v1' crates/presolve_compiler/src; then
  echo 'L9 must not add a public L3 workspace-configuration decoder' >&2
  exit 1
fi
if rg -n 'configuration_codec|decode_cli_workspace_configuration|encode_cli_workspace_configuration' crates/presolve_compiler/src/service.rs crates/presolve_compiler/src/workspace.rs crates/presolve_compiler/src/persistent_cache.rs; then
  echo 'durable L4/L7/L6 code must not invoke the CLI codec' >&2
  exit 1
fi
if rg -n 'std::fs|read_to_string|read_dir|glob|WalkDir' crates/presolve_cli/src/configuration_codec.rs; then
  echo 'the strict codec must not access the filesystem' >&2
  exit 1
fi
./scripts/verify-l3-platform-contracts.sh
./scripts/verify-l4-service-contracts.sh
./scripts/verify-l5-incremental-contracts.sh
./scripts/verify-l6-persistent-cache-contracts.sh
./scripts/verify-l7-workspace-contracts.sh
./scripts/verify-l8-watch-contracts.sh
cargo fmt --all --check
cargo clippy -p presolve-cli --all-targets -- -D warnings
git diff --check
