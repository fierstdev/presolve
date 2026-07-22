#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-m/PHASE_M_M8_DX_COMPATIBILITY.md

test -s "$contract"
for phrase in \
  'adds no framework `explain` command' \
  'project the compiler result unchanged' \
  'TypeScript 7.1' \
  'No compatibility layer may reinterpret' \
  'no project discovery'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done

for example in \
  examples/counter/src/Counter.tsx \
  framework/tests/computed-types/src/ComputedDiamond.tsx \
  framework/tests/context-types/src/ContextRuntimeMatrix.tsx \
  framework/tests/forms-types/src/FormHost.tsx \
  framework/tests/forms-resume-types/src/ResumeForms.tsx; do
  test -s "$example"
done

cargo run -q -p presolve-cli -- explain --inspect \
  framework/tests/forms-types/src/FormHost.tsx --format json \
  | rg --fixed-strings --quiet '"schema_version"'
git diff --check
