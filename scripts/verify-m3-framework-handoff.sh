#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly package=framework/packages/framework

test -s "$package/package.json"
test -s "$package/src/index.js"
test -s "$package/src/project-handoff.js"
test -s "$package/test/project-handoff.test.mjs"

rg --fixed-strings --quiet '"name": "@presolve/framework"' "$package/package.json"
rg --fixed-strings --quiet '"private": true' "$package/package.json"
rg --fixed-strings --quiet 'createArtifactBuildInvocation' "$package/src/index.js"
rg --fixed-strings --quiet 'invokeArtifactBuild' "$package/src/index.js"
rg --fixed-strings --quiet 'executable: "presolve"' "$package/src/project-handoff.js"
rg --fixed-strings --quiet '"build", sourcePath, "--out", outputDirectory' "$package/src/project-handoff.js"

if rg --line-number 'node:(fs|path|child_process)|\b(fetch|spawn|exec|readFile|writeFile|glob)\b|JSON\.parse' "$package/src"; then
  echo 'M3 handoff must remain source-free, product-opaque, and runtime-free' >&2
  exit 1
fi

node --test "$package/test/project-handoff.test.mjs"
./scripts/verify-m2-framework-types.sh
git diff --check
