#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-o/PHASE_O_O1_EXPLICIT_APPLICATION_BUILD_HANDOFF.md
readonly package=metaframework/packages/application
test -s "$contract"
test -s "$package/package.json"
test -s "$package/src/index.js"
test -s "$package/src/application-build-handoff.js"
test -s "$package/test/application-build-handoff.test.mjs"
rg --fixed-strings --quiet '"name": "@presolve/application"' "$package/package.json"
for phrase in 'createApplicationBuildInvocation' 'invokeApplicationBuild' 'does not read files' 'localeCompare'; do
  rg --fixed-strings --quiet "$phrase" "$package/src"
done
node --test "$package/test/application-build-handoff.test.mjs"

readonly fixture=framework/tests/resource-types
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
cargo fmt --all --check
git diff --check
