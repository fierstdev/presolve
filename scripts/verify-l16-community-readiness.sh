#!/usr/bin/env bash
set -euo pipefail
for file in LICENSE CHANGELOG.md CONTRIBUTING.md SECURITY.md CODE_OF_CONDUCT.md .github/ISSUE_TEMPLATE/bug-report.yml .github/ISSUE_TEMPLATE/feature-slice.yml .github/pull_request_template.md; do
  test -s "$file"
done
rg --quiet 'MIT License' LICENSE
rg --quiet 'no hosted service' CONTRIBUTING.md SECURITY.md README.md
rg --quiet 'private' SECURITY.md
rg --quiet 'type: textarea' .github/ISSUE_TEMPLATE/bug-report.yml
./scripts/verify-repository-layout.sh
git diff --check
