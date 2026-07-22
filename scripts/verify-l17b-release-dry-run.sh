#!/usr/bin/env bash
set -euo pipefail
test -x scripts/release-dry-run.sh
test -s .github/workflows/release-dry-run.yml
rg --quiet 'pnpm install --offline' scripts/release-dry-run.sh
rg --quiet 'pnpm -r check' scripts/release-dry-run.sh
rg --quiet 'pnpm --dir .* pack --json' scripts/release-dry-run.sh
rg --quiet 'presolve.release-dry-run' scripts/release-dry-run.sh
if rg --quiet 'publish|npm token|NPM_TOKEN|sign|upload|curl|wget' scripts/release-dry-run.sh .github/workflows/release-dry-run.yml; then
  echo 'release dry run must not publish, sign, upload, or require secrets' >&2
  exit 1
fi
./scripts/verify-l17a-distribution-contract.sh
git diff --check
