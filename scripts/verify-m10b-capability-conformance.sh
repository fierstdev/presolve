#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-m/PHASE_M_M10_FRAMEWORK_ADOPTION.md
readonly package=framework/packages/framework-types/src/index.d.ts
readonly fixture=framework/tests/jsx-capability-types

test -s "$contract"
test -s "$fixture/tsconfig.json"
test -s "$fixture/tsconfig.invalid.json"
test -s "$fixture/presolve.json"
test -s "$fixture/src/AccessibilityPanel.tsx"
test -s "$fixture/src/InvalidAttributes.tsx"
capability_matrix="$(cargo run -q -p presolve-cli -- explain --capabilities --format human)"
for capability in \
  component component_invocation state serializable_state_replacement \
  static_action_parameters action_parameter_state_types serializable_action_locals \
  structured_serializable_action_locals action computed effect context slot \
  keyed_structural_list jsx_html_attribute_aliases typed_aria_bindings \
  keyboard_action_event form module_bindings advanced_types \
  semantic_package_bindings semantic_package_pure_identity template_interpolation \
  static_index_access boolean_computed_conditional builtin_math_abs \
  builtin_math_min_max builtin_math_rounding semantic_package_exports resources \
  opaque_typescript; do
  rg --fixed-strings --quiet "\`$capability\`" "$contract"
  printf '%s\n' "$capability_matrix" | rg --fixed-strings --quiet "$capability |"
done
for phrase in \
  'className?: string;' \
  'htmlFor?: string;' \
  '"aria-invalid"?: boolean;' \
  '"aria-live"?: string;' \
  'onKeydown?: PresolveActionEventHandler;' \
  'interface PresolveIntrinsicAttributes'; do
  rg --fixed-strings --quiet "$phrase" "$package"
done

typescript_version="$(pnpm exec tsc --version)"
if [[ ! "$typescript_version" =~ (^|$'\n')Version\ 7\.0\.[0-9]+($|$'\n') ]]; then
  echo "M10-B requires the pinned TypeScript 7.0 native CLI, found: $typescript_version" >&2
  exit 1
fi
pnpm exec tsc --project "$fixture/tsconfig.json"
if pnpm exec tsc --project "$fixture/tsconfig.invalid.json" >/dev/null 2>&1; then
  echo 'M10-B must reject invalid typed JSX attributes' >&2
  exit 1
fi
cleanup() {
  if test -d "$fixture/.presolve"; then
    find "$fixture/.presolve" -depth -delete
  fi
}
cleanup
trap cleanup EXIT
cargo run -q -p presolve-cli -- check --config "$fixture/presolve.json" \
  --source accessibility.tsx=src/AccessibilityPanel.tsx --format json | \
  rg --fixed-strings --quiet '"status":"succeeded"'
cargo fmt --all --check
git diff --check
