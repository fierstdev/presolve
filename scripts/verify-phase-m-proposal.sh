#!/usr/bin/env bash
set -euo pipefail

readonly roadmap=docs/specifications/phase-m/PHASE_M_PROPOSED_ROADMAP.md
readonly constitution=docs/specifications/phase-m/PHASE_M_FRAMEWORK_CONSTITUTION.md
readonly authoring=docs/specifications/phase-m/PHASE_M_CONFORMANCE_AUTHORING_CONTRACT.md

for document in "$roadmap" "$constitution" "$authoring"; do
  test -s "$document"
done

for heading in 'Product boundary' 'Governing authorities' 'Architectural decisions' 'Slice sequence' 'Evidence matrix' 'Next authorized action'; do
  rg --fixed-strings --quiet "$heading" "$roadmap"
done

for slice in M0 M1 M2 M3 M4 M5 M6 M7 M8 M9; do
  rg --fixed-strings --quiet "$slice" "$roadmap"
done

for phrase in 'conformance-first' 'reserved exit-6' 'Presolve Metaframework' 'state(initializer)' 'does not decode' 'fails closed'; do
  rg --fixed-strings --quiet "$phrase" "$roadmap" "$constitution" "$authoring"
done

for phrase in '@component("x-name")' 'count = state(0)' '@action()' '@computed()' '@effect()' '@context()' '@provide(Theme.theme)' '@consume(Theme.theme)' '@slot()' '@form()'; do
  rg --fixed-strings --quiet "$phrase" "$authoring"
done

for unsupported in \
  '`@state() count = 0`, signals, `.value`, proxies, setters, and framework state storage are unavailable.' \
  '`context<T>()`, global Context handles, and string keys are unavailable.' \
  'JSX factory, `jsx-runtime`, DOM renderer, or TypeScript decorator transform.' \
  'than framework shims over the compiler.'; do
  rg --fixed-strings --quiet "$unsupported" "$authoring"
done

rg --fixed-strings --quiet 'M0/M1 owner-accepted' "$roadmap"
rg --fixed-strings --quiet 'M2 only' "$roadmap"
git diff --check
