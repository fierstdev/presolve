#!/usr/bin/env bash
set -euo pipefail

contract=docs/platform-freeze-contract.md
required_files=(
  docs/alpha-support-matrix.md
  docs/frozen-contract-map.md
  docs/production-optimization-contract.md
  docs/reproducibility-lanes.md
  docs/launch-content-contract.md
  docs/alpha-rehearsal.md
  docs/distribution-contract.md
  scripts/verify-l13-l21-continuation-contract.sh
  scripts/verify-public-identity.sh
  scripts/verify-repository-layout.sh
  scripts/verify-l13a-public-docs-index.sh
  scripts/verify-l13b-public-cli-docs.sh
  scripts/verify-l13d-public-surface-matrix.sh
  scripts/verify-l14b-production-resume-example.sh
  scripts/verify-l15c-reproducibility-lanes.sh
  scripts/verify-l17b-release-dry-run.sh
  scripts/verify-l18-launch-content.sh
  scripts/verify-l19a-alpha-support-matrix.sh
  scripts/verify-l19b-clean-room-rehearsal.sh
)

test -s "$contract"
for file in "${required_files[@]}"; do
  test -s "$file"
done

for heading in 'Frozen public platform' 'Reserved-capability disposition' 'Final evidence matrix'; do
  rg --fixed-strings --quiet "$heading" "$contract"
done
for command in create dev benchmark doctor; do
  rg --fixed-strings --quiet "\`$command\`" "$contract"
done
rg --fixed-strings --quiet 'exit-6 command' "$contract"
rg --fixed-strings --quiet 'not deferred implementation promises' "$contract"
rg --fixed-strings --quiet 'just check' "$contract"
rg --fixed-strings --quiet 'Phase L is complete only after' "$contract"

./scripts/verify-l13-l21-continuation-contract.sh
./scripts/verify-public-identity.sh
./scripts/verify-repository-layout.sh
./scripts/verify-l19a-alpha-support-matrix.sh
git diff --check
