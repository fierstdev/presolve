#!/usr/bin/env bash
set -euo pipefail

readonly roadmap=docs/specifications/phase-m/PHASE_M_PROPOSED_ROADMAP.md
readonly constitution=docs/specifications/phase-m/PHASE_M_FRAMEWORK_CONSTITUTION.md
readonly authoring=docs/specifications/phase-m/PHASE_M_CONFORMANCE_AUTHORING_CONTRACT.md
readonly m2_contract=docs/specifications/phase-m/PHASE_M_M2_FRAMEWORK_TYPES_CONTRACT.md
readonly m3_contract=docs/specifications/phase-m/PHASE_M_M3_EXPLICIT_HANDOFF_CONTRACT.md
readonly m4_audit=docs/specifications/phase-m/PHASE_M_M4_PUBLICATION_AUDIT.md
readonly m5_computed=docs/specifications/phase-m/PHASE_M_M5_COMPUTED_CONFORMANCE.md
readonly m5_effect=docs/specifications/phase-m/PHASE_M_M5_EFFECT_CONFORMANCE.md
readonly m6_component_slot=docs/specifications/phase-m/PHASE_M_M6_COMPONENT_SLOT_CONFORMANCE.md
readonly m6_context=docs/specifications/phase-m/PHASE_M_M6_CONTEXT_LANGUAGE_CONTRACT.md
readonly m7_forms_resume=docs/specifications/phase-m/PHASE_M_M7_FORMS_RESUME_CONFORMANCE.md
readonly m8_dx=docs/specifications/phase-m/PHASE_M_M8_DX_COMPATIBILITY.md
readonly m9_freeze=docs/specifications/phase-m/PHASE_M_M9_FRAMEWORK_FREEZE.md

for document in "$roadmap" "$constitution" "$authoring" "$m2_contract" "$m3_contract" "$m4_audit" "$m5_computed" "$m5_effect" "$m6_component_slot" "$m6_context" "$m7_forms_resume" "$m8_dx" "$m9_freeze"; do
  test -s "$document"
done

for heading in 'Product boundary' 'Governing authorities' 'Architectural decisions' 'Slice sequence' 'Evidence matrix' 'Current boundary'; do
  rg --fixed-strings --quiet "$heading" "$roadmap"
done

for slice in M0 M1 M2 M3 M4 M5 M6 M7 M8 M9; do
  rg --fixed-strings --quiet "$slice" "$roadmap"
done

for phrase in 'conformance-first' 'reserved exit-6' 'Presolve Metaframework' 'state(initializer)' 'does not decode' 'fails closed' 'production language-evolution'; do
  rg --fixed-strings --quiet "$phrase" "$roadmap" "$constitution" "$authoring"
done

for phrase in '@component("x-name")' 'count = state(0)' '@action()' '@computed()' '@effect()' '@context()' '@provide("Theme.theme")' '@consume("Theme.theme")' '@slot()' '@form()' '@field("profile")' '@submit("profile")'; do
  rg --fixed-strings --quiet "$phrase" "$authoring"
done

for unsupported in \
  '`@state() count = 0`, signals, `.value`, proxies, setters, and framework state storage are unavailable.' \
  '`context<T>()`, global Context handles, and Context factories are unavailable.' \
  'JSX factory, `jsx-runtime`, DOM renderer, or TypeScript decorator transform.' \
  'than framework shims over the compiler.'; do
  rg --fixed-strings --quiet "$unsupported" "$authoring"
done

rg --fixed-strings --quiet 'M0/M1 owner-accepted' "$roadmap"
rg --fixed-strings --quiet 'M2 framework types contract' "$roadmap"
rg --fixed-strings --quiet 'M3 explicit handoff contract' "$roadmap"
rg --fixed-strings --quiet 'M4 public artifact-publication audit' "$roadmap"
rg --fixed-strings --quiet 'M6-A authority:' "$roadmap"
rg --fixed-strings --quiet 'M6-B authority' "$roadmap"
rg --fixed-strings --quiet 'M7 authority' "$roadmap"
rg --fixed-strings --quiet 'M8 authority' "$roadmap"
rg --fixed-strings --quiet 'M9 authority' "$roadmap"
git diff --check
