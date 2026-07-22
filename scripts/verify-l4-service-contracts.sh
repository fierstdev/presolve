#!/usr/bin/env bash

set -euo pipefail

readonly service_source='crates/presolve_compiler/src/service.rs'
readonly platform_source='crates/presolve_compiler/src/platform.rs'
readonly fixture_dir='crates/presolve_compiler/fixtures/service'

test -f "$service_source"
test -f docs/compiler-service-contract.md
for constant in COMPILER_SERVICE_PROTOCOL_VERSION COMPILER_SERVICE_DESCRIPTOR_SCHEMA_VERSION COMPILER_SERVICE_REQUEST_SCHEMA_VERSION COMPILER_SERVICE_RESPONSE_SCHEMA_VERSION DURABLE_SESSION_SCHEMA_VERSION DURABLE_COMMIT_SCHEMA_VERSION SESSION_JOURNAL_SCHEMA_VERSION PERSISTENCE_MANIFEST_SCHEMA_VERSION SERVICE_INSPECTION_SCHEMA_VERSION SESSION_INSPECTION_SCHEMA_VERSION; do
  rg --quiet "pub const $constant: u32 = 1" "$service_source"
done
for fixture in "$fixture_dir"/*.json; do
  test -s "$fixture"
  test "$(tail -c 1 "$fixture")" = ''
done
rg --quiet 'encode_frame' "$service_source"
rg --quiet 'decode_frame' "$service_source"
rg --quiet 'decode_workspace_snapshot_json_v1' "$platform_source"
rg --quiet 'decode_workspace_graph_json_v1' "$platform_source"
rg --quiet 'atomic_write' "$service_source"
rg --quiet 'fs::rename' "$service_source"
! rg --quiet 'TcpListener|TcpStream|hyper|reqwest|http://' "$service_source"
! rg --quiet 'read_to_string|read_to_end' "$service_source"
! rg --quiet 'source:.*session.json|source:.*journal' "$service_source"
