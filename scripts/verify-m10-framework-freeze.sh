#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-m/PHASE_M_M10_FRAMEWORK_ADOPTION.md
test -s "$contract"
for phrase in \
  'M10-A — Resource type conformance' \
  'M10-B — capability disposition and JSX conformance' \
  'M10-C — amendment freeze' \
  'frozen by M10-C aggregate verification' \
  'No M10 declaration is a fallback'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done

./scripts/verify-m9-framework-freeze.sh
./scripts/verify-m10a-resource-conformance.sh
./scripts/verify-m10b-capability-conformance.sh
./scripts/verify-n8a-capability-matrix.sh
cargo fmt --all --check
git diff --check
git diff --quiet
