#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-m/PHASE_M_M9_FRAMEWORK_FREEZE.md
test -s "$contract"
for phrase in \
  'Frozen framework surface' \
  'Permanently unavailable in Phase M' \
  'every focused M2–M8 verifier' \
  'TypeScript 7.1 is unsupported'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done

./scripts/verify-phase-m-proposal.sh
./scripts/verify-m2-framework-types.sh
./scripts/verify-m3-framework-handoff.sh
./scripts/verify-m4-publication-audit.sh
./scripts/verify-m5-computed-conformance.sh
./scripts/verify-m5-effect-conformance.sh
./scripts/verify-m6-component-slot-conformance.sh
./scripts/verify-m6-context-conformance.sh
./scripts/verify-m7-forms-resume-conformance.sh
./scripts/verify-m8-framework-dx.sh
cargo fmt --all --check
git diff --check
