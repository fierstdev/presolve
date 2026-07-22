#!/usr/bin/env bash
set -euo pipefail
index=docs/README.md
test -s "$index"
rg --quiet '^# Presolve documentation$' "$index"
rg --quiet 'Current references' "$index"
rg --quiet '^## Guides$' "$index"
rg --quiet 'Ownership and version policy' "$index"
rg --quiet 'presolve-snippet: id=<lowercase-dash-id>; kind=command' "$index"
rg --quiet '^## Archive$' "$index"
rg --quiet 'non-normative' "$index"
./scripts/verify-l14b-production-resume-example.sh
git diff --check
