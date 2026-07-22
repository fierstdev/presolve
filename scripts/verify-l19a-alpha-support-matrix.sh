#!/usr/bin/env bash
set -euo pipefail

matrix=docs/alpha-support-matrix.md
required_files=(
  scripts/verify-public-identity.sh
  scripts/verify-l13b-public-cli-docs.sh
  crates/presolve_cli/tests/explain.rs
  scripts/verify-l11c-tooling-commands.sh
  scripts/verify-l11g-artifact-graph-command.sh
  scripts/verify-l11g-trace-command.sh
  scripts/verify-l11g-profile-command.sh
  scripts/verify-l13d-public-surface-matrix.sh
  scripts/verify-l10-schema-contract.sh
  scripts/verify-l12c3-wasm-binding.sh
  scripts/verify-l12c4-language-service.sh
  scripts/verify-l12d2-lsp-adapter.sh
  scripts/verify-l12e2-vscode-facade.sh
  scripts/verify-l15b-testing-package.sh
  docs/distribution-contract.md
  CONTRIBUTING.md
  SECURITY.md
  SUPPORT.md
)

test -s "$matrix"
for file in "${required_files[@]}"; do
  test -s "$file"
done

for heading in 'Terminology and compatibility' 'Available command families' 'Available compiler products' 'Available editor and package surfaces' 'Support and rollback policy'; do
  rg --fixed-strings --quiet "$heading" "$matrix"
done

for command in 'version' 'help' 'build' 'check' 'clean' 'cache inspect' 'cache verify' 'cache clean' 'workspace' 'watch --once' 'explain' 'explain --inspect' 'inspect workspace-snapshot' 'inspect workspace-graph' 'graph workspace' 'graph artifact' 'trace' 'profile'; do
  rg --fixed-strings --quiet "\`$command\`" "$matrix"
done

for reserved in create dev benchmark doctor; do
  rg --fixed-strings --quiet "\`$reserved\`" "$matrix"
done

for schema in presolve.workspace-configuration presolve.workspace-snapshot presolve.workspace-graph presolve.compiler-service-protocol presolve.persistent-artifact-cache presolve.cache-inspection-report.v1 presolve.workspace-manifest presolve.watch-session-configuration presolve.watch-change-batch presolve.watch-execution-plan presolve.watch-event presolve.watch-session-snapshot presolve.watch-execution-report presolve.build-trace presolve.compile-cost-report presolve.artifact-graph presolve.query-snapshot; do
  rg --fixed-strings --quiet "$schema" "$matrix"
done

for package in @presolve/compiler-wasm @presolve/language-service @presolve/lsp @presolve/vscode @presolve/testing @presolve/runtime; do
  rg --fixed-strings --quiet "$package" "$matrix"
done

rg --fixed-strings --quiet 'exit `6`' "$matrix"
rg --fixed-strings --quiet 'Every manifest is private' "$matrix"
rg --fixed-strings --quiet 'no service-level agreement' "$matrix"
rg --fixed-strings --quiet 'may revert to the last committed matrix-compatible revision' "$matrix"
./scripts/verify-public-identity.sh
git diff --check
