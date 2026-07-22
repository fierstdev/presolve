#!/usr/bin/env bash

set -euo pipefail

readonly legacy_identity_pattern='Edge''Zero|edge''zero|edge-''zero|@edge''zero|E''ZC[0-9]+|E''ZASM[0-9]+|E''ZR_|data-''e''z-|__EDGE''ZERO|#e''z-'
readonly legacy_path_pattern='edge''zero|edge-''zero|e''zc|(^|/)e''z([._-]|$)'
readonly retired_command_pattern='presolve'' asm|fn run_''asm\('
readonly archived_docs_glob='!docs/''archive/**'
readonly -a active_surfaces=(
  Cargo.toml
  crates
  packages
  fixtures
  examples
  e2e
  schemas
  scripts
  docs
  package.json
  README.md
  justfile
  .github
)

if rg --line-number --ignore-case --regexp "$legacy_identity_pattern" "${active_surfaces[@]}" \
  --glob "$archived_docs_glob" \
  --glob '!notes/progress/**' \
  --glob '!target/**' \
  --glob '!node_modules/**'; then
  echo 'legacy active identity found; see docs/presolve-identity-migration-contract.md' >&2
  exit 1
fi

if rg --files "${active_surfaces[@]}" | rg --ignore-case "$legacy_path_pattern"; then
  echo 'legacy active identity found in a path' >&2
  exit 1
fi

rg --quiet '^repository = "https://github.com/fierstdev/presolve"$' Cargo.toml
rg --quiet '"name": "presolve-monorepo"' package.json
rg --quiet '"name": "@presolve/runtime"' packages/runtime/package.json
rg --quiet '^name = "presolve-cli"$' crates/presolve_cli/Cargo.toml
rg --quiet '^name = "presolve"$' crates/presolve_cli/Cargo.toml
rg --quiet 'presolve explain <file>' README.md
rg --quiet 'PRESOLVE_CHROME' .github/workflows/ci.yml
rg --quiet '"asm" => l9_command_error\("asm", "retired: use presolve explain", 6\)' crates/presolve_cli/src/main.rs
if rg --line-number "$retired_command_pattern" README.md docs crates packages scripts \
  --glob "$archived_docs_glob" \
  --glob '!notes/progress/**'; then
  echo 'retired inspection command is still active' >&2
  exit 1
fi
