#!/usr/bin/env bash
set -euo pipefail

rehearsal=docs/alpha-rehearsal.md
runner=scripts/run-l19b-clean-room-rehearsal.sh

test -s "$rehearsal"
test -x "$runner"
for phrase in 'presolve create' 'reserved and exits `6`' 'pnpm install --offline' 'cargo build -p presolve-cli' 'verify-l14b-explicit-workspace-example' 'verify-l11c-tooling-commands' 'verify-l11g-artifact-graph-command' 'verify-l11g-trace-command' 'verify-l11g-profile-command' 'Every package remains private' 'release artifact or external side effect'; do
  rg --fixed-strings --quiet "$phrase" "$rehearsal"
done
rg --fixed-strings --quiet 'git worktree add --detach' "$runner"
rg --fixed-strings --quiet 'pnpm install --offline' "$runner"
rg --fixed-strings --quiet 'cargo build -q -p presolve-cli' "$runner"
rg --fixed-strings --quiet 'verify-l11g-profile-command' "$runner"
rg --fixed-strings --quiet 'fixtures/phase-k-benchmarks' "$runner"
bash -n "$runner"
git diff --check
