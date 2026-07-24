#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"

node --test packages/create-presolve/test/create-presolve.test.mjs
node_modules/.bin/tsc -p framework/tests/public-package-types/tsconfig.json --pretty false
cargo test -p presolve-compiler binding_table::tests::accepts_the_public_presolve_authoring_import_without_a_package_capability_contract --lib -- --exact
cargo test -p presolve-cli --test ergonomic_project fresh_scaffold_needs_no_configuration_source_list_or_component_identity -- --exact
cargo test -p presolve-cli --test ergonomic_project explain_projects_compiler_route_and_prepared_deployment_facts -- --exact
cargo test -p presolve-cli --test ergonomic_project deploy_prepare_projects_compiler_artifacts_to_cloudflare_workers_static_assets -- --exact
cargo build -p presolve-cli
(
  cd examples/presolve-site
  ../../target/debug/presolve check
  ../../target/debug/presolve build
  test -f dist/routes/root/index.html
  test -f dist/routes/segment-docs/segment-getting-started/index.html
  test -f dist/routes/segment-compare/index.html
)
