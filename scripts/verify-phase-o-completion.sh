#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

for contract in \
  docs/specifications/phase-o/PHASE_O_O5_ROUTING_SERVER_INTAKE.md \
  docs/specifications/phase-o/PHASE_O_O6_DEPLOYMENT_DISTRIBUTION_INTAKE.md; do
  test -s "$contract"
done
rg --fixed-strings --quiet 'no routing/server product is admitted' docs/specifications/phase-o/PHASE_O_O5_ROUTING_SERVER_INTAKE.md
rg --fixed-strings --quiet 'no deployment product is admitted' docs/specifications/phase-o/PHASE_O_O6_DEPLOYMENT_DISTRIBUTION_INTAKE.md
node --test metaframework/packages/application/test/application-build-handoff.test.mjs
scripts/verify-p5-application-publication-freeze.sh
git diff --check
