#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-m/PHASE_M_M7_FORMS_RESUME_CONFORMANCE.md
readonly forms_fixture=framework/tests/forms-types
readonly resume_fixture=framework/tests/forms-resume-types

test -s "$contract"
for phrase in \
  'production compiler-language refinement' \
  '@field("profile")' \
  '@submit("profile")' \
  'exact sources built by the existing real-browser probes' \
  'No framework-side artifact decoding'; do
  rg --fixed-strings --quiet "$phrase" "$contract"
done

for fixture in "$forms_fixture" "$resume_fixture"; do
  test -s "$fixture/tsconfig.json"
  test -s "$fixture/presolve.json"
done
test -s "$forms_fixture/src/FormHost.tsx"
test -s "$resume_fixture/src/ResumeForms.tsx"
rg --fixed-strings --quiet 'interface Form' framework/packages/framework-types/src/index.d.ts
rg --fixed-strings --quiet 'type FormDesignator = string' framework/packages/framework-types/src/index.d.ts
rg --fixed-strings --quiet 'function form()' framework/packages/framework-types/src/index.d.ts
rg --fixed-strings --quiet 'function field(form: FormDesignator)' framework/packages/framework-types/src/index.d.ts
rg --fixed-strings --quiet 'function submit(form: FormDesignator)' framework/packages/framework-types/src/index.d.ts

CI=true pnpm exec tsc --project "$forms_fixture/tsconfig.json"
CI=true pnpm exec tsc --project "$resume_fixture/tsconfig.json"
cargo test -q -p presolve-compiler \
  lowers_one_submit_plan_with_the_complete_form_rule_order --lib
cargo test -q -p presolve-compiler \
  retains_invalid_decorators_targets_designators_and_owning_forms_without_field_ids --lib
cargo test -q -p presolve-parser \
  retains_normalized_form_field_designators_targets_values_and_provenance --lib
for fixture in "$forms_fixture" "$resume_fixture"; do
  rm -rf "$fixture/.presolve"
done
trap 'rm -rf "$forms_fixture/.presolve" "$resume_fixture/.presolve"' EXIT
cargo run -q -p presolve-cli -- check \
  --config "$forms_fixture/presolve.json" \
  --source forms.tsx=src/FormHost.tsx \
  --format json | rg --fixed-strings --quiet '"status":"succeeded"'
cargo run -q -p presolve-cli -- check \
  --config "$resume_fixture/presolve.json" \
  --source resume-forms.tsx=src/ResumeForms.tsx \
  --format json | rg --fixed-strings --quiet '"status":"succeeded"'
cargo test -q -p presolve-cli --test runtime_browser \
  explicit_form_hosts_submit_only_through_compiler_emitted_records -- --nocapture
cargo test -q -p presolve-cli --test runtime_browser \
  resume_restores_compiler_owned_form_state_and_rejects_active_submission -- --nocapture
git diff --check
