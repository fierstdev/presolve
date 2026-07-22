#!/usr/bin/env bash
set -euo pipefail
manifest=docs/reproducibility-lanes.md
test -s "$manifest"
rg --quiet 'deterministic-contracts' "$manifest"
rg --quiet 'browser-runtime' "$manifest"
rg --quiet 'package-smoke' "$manifest"
rg --quiet 'all five contracted explicit alpha examples' "$manifest"
rg --quiet '| required |' "$manifest"
rg --quiet 'never a gate' "$manifest"
rg --quiet 'cannot compare elapsed time, CPU, memory, machine identity, or benchmark' "$manifest"
./scripts/verify-l15b-testing-package.sh
git diff --check
