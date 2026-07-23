#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

test -s docs/specifications/phase-p/PHASE_P_P5_COMPATIBILITY_FREEZE.md
rg --fixed-strings --quiet 'presolve-application-publication:1' docs/specifications/phase-p/PHASE_P_P5_COMPATIBILITY_FREEZE.md
rg --fixed-strings --quiet 'application build --config' crates/presolve_cli/src/main.rs
rg --fixed-strings --quiet 'createApplicationPublicationInvocation' metaframework/packages/application/src/index.js

cargo fmt --all -- --check
cargo test -q -p presolve-compiler application_publication::tests --lib
cargo test -q -p presolve-cli --test application_publication
node --test metaframework/packages/application/test/application-build-handoff.test.mjs
git diff --check
