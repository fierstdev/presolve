#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-m/PHASE_M_M10_FRAMEWORK_ADOPTION.md
readonly package=framework/packages/framework-types/src/index.d.ts
readonly fixture=framework/tests/resource-types

test -s "$contract"
test -s "$fixture/presolve.json"
test -s "$fixture/tsconfig.json"
test -s "$fixture/src/Profile.tsx"
test -s "$fixture/src/profile-service.d.ts"
test -s "$fixture/profile-service.contract.json"
rg --fixed-strings --quiet 'function resource(endpointDesignator: string)' "$package"
rg --fixed-strings --quiet 'interface Resource<Data, Error>' "$package"
rg --fixed-strings --quiet 'type ResourceState = "idle" | "pending" | "ready" | "failed" | "cancelled"' "$package"
rg --fixed-strings --quiet 'N6-C14 currently permits direct Resource projections only in a same-owner' "$contract"
rg --fixed-strings --quiet 'not a framework Resource runtime' "$contract"

typescript_version="$(pnpm exec tsc --version)"
if [[ ! "$typescript_version" =~ (^|$'\n')Version\ 7\.0\.[0-9]+($|$'\n') ]]; then
  echo "M10-A requires the pinned TypeScript 7.0 native CLI, found: $typescript_version" >&2
  exit 1
fi
pnpm exec tsc --project "$fixture/tsconfig.json"

cleanup() {
  if test -d "$fixture/.presolve"; then
    find "$fixture/.presolve" -depth -delete
  fi
}
cleanup
trap cleanup EXIT
cargo run -q -p presolve-cli -- build "$fixture/src/Profile.tsx" \
  --package-contract "profile-service=$fixture/profile-service.contract.json" \
  --package-runtime 'profile-service=./profile-resource.js' \
  --out "$fixture/.presolve"
test -s "$fixture/.presolve/resources.runtime.json"
rg --fixed-strings --quiet 'profile-service' "$fixture/.presolve/resources.runtime.json"

RUST_TEST_THREADS=1 cargo test -q -p presolve-cli --test runtime_browser \
  host_bound_resource_endpoint_activates_in_a_real_browser -- --exact --nocapture
cargo fmt --all --check
git diff --check
