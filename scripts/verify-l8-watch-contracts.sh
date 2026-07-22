#!/usr/bin/env bash
set -euo pipefail

# L8 is intentionally a caller-driven state machine: this audit rejects
# filesystem watcher/discovery surfaces and proves fake-clock test coverage.
cargo test -p presolve-compiler watch --lib
for fixture in crates/presolve_compiler/fixtures/watch/*.json; do
  test -s "$fixture"
  test "$(tail -c 1 "$fixture")" = "$(printf '\n')"
  rg -q '"schema"' "$fixture"
done
rg -q '"single-immediate-candidate"' crates/presolve_compiler/fixtures/watch/scenarios-v1.json
rg -q '"l6-restart-hit"' crates/presolve_compiler/fixtures/watch/scenarios-v1.json
rg -q '"l5-content-edit-reuse"' crates/presolve_compiler/fixtures/watch/scenarios-v1.json
if rg -n -i 'fswatch|notify::|recommendedwatcher|inotify|kqueue|read_dir|walkdir|glob::|metadata\(' crates/presolve_compiler/src/watch.rs; then
  echo "L8 watch implementation must not discover filesystem state" >&2
  exit 1
fi
if rg -n 'SOURCE_SENTINEL|watch-authored-source' crates/presolve_compiler/fixtures/watch docs/watch-mode-contract.md; then
  echo "source sentinel leaked into L8 serialized products" >&2
  exit 1
fi
./scripts/verify-l3-platform-contracts.sh
./scripts/verify-l4-service-contracts.sh
./scripts/verify-l5-incremental-contracts.sh
./scripts/verify-l6-persistent-cache-contracts.sh
./scripts/verify-l7-workspace-contracts.sh
cargo fmt --all --check
cargo clippy -p presolve-compiler --all-targets -- -D warnings
git diff --check
