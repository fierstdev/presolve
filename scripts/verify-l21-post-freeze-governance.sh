#!/usr/bin/env bash
set -euo pipefail

document=docs/post-freeze-governance.md
test -s "$document"
for heading in 'Versioning and compatibility' Amendments 'Security and release authority' Deprecation 'Next-roadmap intake' 'Freeze evidence'; do
  rg --fixed-strings --quiet "$heading" "$document"
done
for phrase in 'non-feature governance' 'authorizes no implementation' 'owner-approved amendment' 'external release authority' 'No implementation begins' 'reserved exit-6'; do
  rg --fixed-strings --quiet "$phrase" "$document"
done
for file in docs/platform-freeze-contract.md docs/alpha-support-matrix.md SECURITY.md scripts/verify-l20-platform-freeze.sh; do
  test -s "$file"
done
./scripts/verify-l20-platform-freeze.sh
git diff --check
