#!/usr/bin/env bash
set -euo pipefail

contract=docs/specifications/phase-l/PHASE_L_L10_TOOLING_SCHEMA_IMPLEMENTATION_CONTRACT.md
compatibility_manifest=crates/ezc_core/fixtures/tooling-schema/l3-l8-canonical-fixtures-v1.sha256
rg --quiet 'Canonical schema registry v1' "$contract"
rg --quiet 'Negotiation v1' "$contract"
rg --quiet 'presolve.build-trace' "$contract"
rg --quiet 'byte-for-byte unchanged' "$contract"
rg --quiet 'PHASE_L_L10_TOOLING_SCHEMA_IMPLEMENTATION_CONTRACT' docs/specifications/phase-l/README.md
test -s docs/tooling-capability-inventory.md
test -s "$compatibility_manifest"
cargo test -p presolve-compiler tooling_schema --lib -- --nocapture

expected_paths="$(awk '{print $2}' "$compatibility_manifest" | LC_ALL=C sort)"
actual_paths="$({
  find crates/ezc_core/fixtures/platform -name '*.json' -type f
  find crates/ezc_core/fixtures/service -name '*.json' -type f
  find crates/ezc_core/fixtures/incremental -name '*.json' -type f
  find crates/ezc_core/fixtures/persistent-cache -name '*.json' -type f
  find crates/ezc_core/fixtures/workspace -name '*.json' -type f
  find crates/ezc_core/fixtures/watch -name '*.json' -type f
  printf '%s\n' crates/ezc_cli/fixtures/configuration/minimum-l3-v1.json
  printf '%s\n' crates/ezc_cli/fixtures/configuration/nonempty-l3-v1.json
} | LC_ALL=C sort)"
if [[ "$expected_paths" != "$actual_paths" ]]; then
  echo 'L10 compatibility manifest must cover exactly the L3-L8 canonical fixture corpus' >&2
  exit 1
fi
shasum -a 256 -c "$compatibility_manifest"

if rg -n 'use crate::(service|persistent_cache|workspace|watch)' crates/ezc_core/src/tooling_schema.rs; then
  echo 'L10 negotiation must remain independent from L3-L8 execution and durable products' >&2
  exit 1
fi
if rg --quiet 'tooling_schema' crates/ezc_core/src/{platform,service,persistent_cache,workspace,watch}.rs crates/ezc_cli/src; then
  echo 'L3-L8 products and CLI dispatch must not depend on the L10 registry' >&2
  exit 1
fi
if rg -n 'fn decode_.*(build.*trace|compile.*cost|artifact.*graph)' crates; then
  echo 'reserved L11 tooling schemas must not have a decoder before a producer contract exists' >&2
  exit 1
fi
./scripts/verify-l9-final-contracts.sh
cargo fmt --all --check
cargo clippy -p presolve-compiler --all-targets -- -D warnings
git diff --check
