#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly audit=docs/specifications/phase-m/PHASE_M_M4_PUBLICATION_AUDIT.md

test -s "$audit"
for heading in Decision 'Counter proof contract' 'Publication ownership' 'Next boundary'; do
  rg --fixed-strings --quiet "$heading" "$audit"
done
for phrase in \
  'presolve build <source> --out <directory>' \
  'legacy compatibility' \
  'browser clicks Counter' \
  'no hydration step, framework renderer, state store,' \
  'The compiler owns every emitted file and all diagnostics'; do
  rg --fixed-strings --quiet "$phrase" "$audit"
done

if rg --line-number 'node:(fs|path|child_process)|\b(fetch|spawn|exec|readFile|writeFile|glob)\b|JSON\.parse' framework/packages/framework/src; then
  echo 'M4 framework handoff must remain path-opaque and runtime-free' >&2
  exit 1
fi

./scripts/verify-m3-framework-handoff.sh
readonly out_dir="$(mktemp -d /private/tmp/presolve-m4-counter.XXXXXX)"
trap 'rm -rf "$out_dir"' EXIT
cargo run -q -p presolve-cli -- build examples/counter/src/Counter.tsx --out "$out_dir" >/dev/null
for artifact in index.html template.manifest.json component.runtime.json resume.runtime.json runtime.js; do
  test -s "$out_dir/$artifact"
done
if rg --ignore-case --quiet 'hydration' "$out_dir"; then
  echo 'M4 Counter artifact unexpectedly contains hydration behavior' >&2
  exit 1
fi
cargo test -q -p presolve-cli --test runtime_browser framework_counter_increments_through_compiler_artifacts_in_a_real_browser -- --nocapture
git diff --check
