#!/usr/bin/env bash
set -euo pipefail
matrix=docs/public-surface-matrix.md
registry=crates/ezc_core/src/tooling_schema.rs
test -s "$matrix"
help="$(cargo run -q -p presolve-cli -- help)"
for command in version build check clean cache workspace watch dev create explain inspect graph trace profile benchmark doctor; do
  printf '%s\n' "$help" | rg --fixed-strings --quiet "$command"
  rg --fixed-strings --quiet "\`$command\`" "$matrix"
done
output="$(mktemp)"
set +e
cargo run -q -p presolve-cli -- dev >"$output" 2>&1
status=$?
set -e
rm -f "$output"
test "$status" -eq 6
for schema in presolve.workspace-configuration presolve.workspace-snapshot presolve.workspace-graph presolve.compiler-service-protocol presolve.persistent-artifact-cache presolve.cache-inspection-report.v1 presolve.workspace-manifest presolve.watch-session-configuration presolve.watch-change-batch presolve.watch-execution-plan presolve.watch-event presolve.watch-session-snapshot presolve.watch-execution-report presolve.build-trace presolve.compile-cost-report presolve.artifact-graph presolve.query-snapshot; do
  rg --fixed-strings --quiet "$schema" "$registry"
  rg --fixed-strings --quiet "$schema" "$matrix"
done
cargo test -q -p presolve-compiler tooling_registry_is_deterministic_and_approved_entries_negotiate --lib -- --nocapture
for entry in 'compiler-wasm:@presolve/compiler-wasm:./dist/presolve_compiler_wasm.js' 'language-service:@presolve/language-service:./src/index.js' 'lsp:@presolve/lsp:./src/index.js' 'vscode:@presolve/vscode:./src/index.js' 'testing:@presolve/testing:./src/index.js' 'runtime:@presolve/runtime:./src/index.ts'; do
  IFS=: read -r directory name export_path <<<"$entry"
  rg --fixed-strings --quiet "\"name\": \"$name\"" "packages/$directory/package.json"
  rg --fixed-strings --quiet "\".\": \"$export_path\"" "packages/$directory/package.json"
  rg --fixed-strings --quiet "$name" "$matrix"
  rg --fixed-strings --quiet "$export_path" "$matrix"
done
./scripts/verify-l13c-frozen-contract-map.sh
git diff --check
