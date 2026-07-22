#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly audit=docs/specifications/phase-m/PHASE_M_M4_PUBLICATION_AUDIT.md

test -s "$audit"
for heading in Question Evidence Decision 'Required owner decision'; do
  rg --fixed-strings --quiet "$heading" "$audit"
done
for phrase in \
  'presolve.cli-result' \
  'snapshot identities only' \
  'Legacy compiler commands remain frozen compatibility paths, not new platform adapters.' \
  'Counter browser/runtime proof is blocked' \
  'must not inspect a build directory or invoke compilation'; do
  rg --fixed-strings --quiet "$phrase" "$audit" docs/cli-build-check.md docs/specifications/phase-l/PHASE_L_REVISED_ROADMAP.md docs/specifications/phase-l/PHASE_L_L11_ARTIFACT_GRAPH_CONTRACT.md
done

if rg --line-number --glob '!test/**' --glob '!*.md' -- '--out|run_build|artifact.*path|production.runtime.json' framework/packages; then
  echo 'M4 must not add a framework artifact-publication adapter' >&2
  exit 1
fi

./scripts/verify-m3-framework-handoff.sh
git diff --check
