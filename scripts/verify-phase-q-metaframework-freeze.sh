#!/usr/bin/env bash
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root"
test -s docs/specifications/phase-q/PHASE_Q_Q5_COMPATIBILITY_FREEZE.md
rg --fixed-strings --quiet 'build_static_route_publication_v1' crates/presolve_compiler/src/route_graph.rs
rg --fixed-strings --quiet 'build_static_request_handoff_v1' crates/presolve_compiler/src/metaframework_handoff.rs
rg --fixed-strings --quiet 'build_deployable_release_manifest_v1' crates/presolve_compiler/src/metaframework_handoff.rs
rg --fixed-strings --quiet 'createRouteGraphInvocation' metaframework/packages/application/src/index.js
cargo fmt --all -- --check
cargo test -q -p presolve-compiler route_graph::tests --lib
cargo test -q -p presolve-compiler metaframework_handoff::tests --lib
node --test metaframework/packages/application/test/application-build-handoff.test.mjs
git diff --check
