#!/usr/bin/env bash

set -euo pipefail

readonly legacy_identity_pattern='EdgeZero|edgezero|edge-zero|@edgezero|EDGEZERO_'
readonly -a public_surfaces=(
  Cargo.toml
  package.json
  packages/runtime/package.json
  examples/counter/package.json
  README.md
  e2e/browser/README.md
  schemas/source-summary.schema.json
  justfile
  .github/workflows/ci.yml
)

if rg --line-number --ignore-case --regexp "$legacy_identity_pattern" "${public_surfaces[@]}"; then
  echo 'legacy public identity found; see docs/presolve-identity-transition.md' >&2
  exit 1
fi

rg --quiet '^repository = "https://github.com/fierstdev/presolve"$' Cargo.toml
rg --quiet '"name": "presolve-monorepo"' package.json
rg --quiet '"name": "@presolve/runtime"' packages/runtime/package.json
rg --quiet '^name = "presolve-cli"$' crates/ezc_cli/Cargo.toml
rg --quiet '^name = "presolve"$' crates/ezc_cli/Cargo.toml
rg --quiet 'presolve explain <file>' README.md
rg --quiet 'PRESOLVE_CHROME' .github/workflows/ci.yml
