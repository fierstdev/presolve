#!/usr/bin/env bash
set -euo pipefail
reference=docs/cli-reference.md
counter=examples/counter
forms=examples/forms
workspace=examples/explicit-workspace
components=examples/components-context-slots
cleanup() {
  rm -rf "$counter/.presolve" "$forms/.presolve" "$workspace/.presolve" "$components/.presolve"
}
trap cleanup EXIT
test -s "$reference"
for id in l13b-version-json l13b-help l13b-check-counter l13b-build-forms l13b-workspace l13b-watch-once l13b-cache; do
  rg --quiet "presolve-snippet: id=$id; kind=command" "$reference"
done
run_presolve() { cargo run -q -p presolve-cli -- "$@"; }
run_presolve version --format json | rg --quiet '"schema":"presolve.cli-version"'
run_presolve help | rg --quiet 'commands: version, build, check'
run_presolve check --config "$counter/presolve.json" --source counter.tsx=src/Counter.tsx --format json | rg --quiet '"command":"check"'
run_presolve build --config "$forms/presolve.json" --source Forms.tsx=src/Forms.tsx --format json | rg --quiet '"command":"build"'
run_presolve workspace --config "$workspace/presolve.json" --source src/main.ts=src/main.ts --format json | rg --quiet '"schema":"presolve.cli-workspace-result"'
run_presolve watch --once --config "$components/presolve.json" --source Composition.tsx=src/Composition.tsx --format json | rg --quiet '"schema":"presolve.cli-watch-once"'
run_presolve cache inspect --config "$counter/presolve.json" --format json | rg --quiet '"schema":"presolve.cache-inspection-report.v1"'
run_presolve cache verify --config "$counter/presolve.json" --format json | rg --quiet '"schema":"presolve.cache-inspection-report.v1"'
run_presolve clean --config "$counter/presolve.json" --format json | rg --quiet '"schema":"presolve.cli-cache-clean"'
cargo test -q -p presolve-cli --test l9_cli_commands l11 -- --nocapture
# The inherited example verifiers own fresh explicit candidates, so hand them
# clean local cache roots after this reference's independent command probes.
cleanup
./scripts/verify-l13a-public-docs-index.sh
git diff --check
