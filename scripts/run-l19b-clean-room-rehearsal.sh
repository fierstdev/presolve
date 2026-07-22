#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
rehearsal=docs/alpha-rehearsal.md
clean_room="$(mktemp -d /tmp/presolve-l19b.XXXXXX)"

cleanup() {
  cleanup_status=$?
  trap - EXIT
  cd "$repo_root"
  git worktree remove --force "$clean_room" >/dev/null 2>&1 || rm -rf "$clean_room"
  git worktree prune
  exit "$cleanup_status"
}
trap cleanup EXIT

test -s "$rehearsal"
for phrase in 'presolve create' 'reserved and exits `6`' 'pnpm install --offline' 'cargo build -p presolve-cli' 'verify-l14b-explicit-workspace-example' 'verify-l11c-tooling-commands' 'verify-l11g-artifact-graph-command' 'verify-l11g-trace-command' 'verify-l11g-profile-command' 'Every package remains private' 'release artifact or external side effect'; do
  rg --fixed-strings --quiet "$phrase" "$rehearsal"
done

git diff --check
git worktree add --detach "$clean_room" HEAD >/dev/null
test -z "$(git -C "$clean_room" status --porcelain)"
cd "$clean_room"

pnpm install --offline --force >/dev/null
cargo build -q -p presolve-cli
bin=target/debug/presolve
"$bin" version --format json | rg --quiet '"schema":"presolve.cli-version"'
./scripts/verify-l14b-explicit-workspace-example.sh
"$bin" check --config examples/counter/presolve.json --source counter.tsx=src/Counter.tsx --format json | rg --quiet '"command":"check"'
"$bin" build --config examples/forms/presolve.json --source Forms.tsx=src/Forms.tsx --format json | rg --quiet '"command":"build"'
"$bin" workspace --config examples/explicit-workspace/presolve.json --source src/main.ts=src/main.ts --format json | rg --quiet '"schema":"presolve.cli-workspace-result"'
"$bin" watch --once --config examples/components-context-slots/presolve.json --source Composition.tsx=src/Composition.tsx --format json | rg --quiet '"schema":"presolve.cli-watch-once"'
"$bin" cache inspect --config examples/counter/presolve.json --format json | rg --quiet '"schema":"presolve.cache-inspection-report.v1"'
"$bin" cache verify --config examples/counter/presolve.json --format json | rg --quiet '"schema":"presolve.cache-inspection-report.v1"'
"$bin" clean --config examples/counter/presolve.json --format json | rg --quiet '"schema":"presolve.cli-cache-clean"'

production_out=examples/production-resume/.presolve
"$bin" build examples/production-resume/src/ComputedDiamond.tsx --out "$production_out" --production >/dev/null
test -s "$production_out/production.runtime.json"
test -s "$production_out/resume.runtime.json"
test -d "$production_out/production"
rg --quiet '"schemaVersion":1' "$production_out/production.runtime.json"
rg --quiet '"schema_version":6' "$production_out/resume.runtime.json"
rm -rf "$production_out"

./scripts/verify-l11c-tooling-commands.sh
./scripts/verify-l11g-artifact-graph-command.sh
./scripts/verify-l11g-trace-command.sh
./scripts/verify-l11g-profile-command.sh

node --input-type=module <<'NODE'
import { readFile } from 'node:fs/promises';

for (const name of ['compiler-wasm', 'language-service', 'lsp', 'vscode', 'testing', 'runtime']) {
  const manifest = JSON.parse(await readFile(`packages/${name}/package.json`, 'utf8'));
  if (manifest.private !== true || manifest.version !== '0.1.0-alpha') {
    throw new Error(`${name}: expected private 0.1.0-alpha package metadata`);
  }
}
for (const file of ['README.md', 'corpus.json', 'budgets.json', 'forms-cross-field.tsx', 'large-synthetic.tsx']) {
  await readFile(`fixtures/phase-k-benchmarks/${file}`);
}
NODE

git diff --check
