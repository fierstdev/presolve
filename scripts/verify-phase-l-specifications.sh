#!/usr/bin/env bash

set -euo pipefail

readonly specification_dir='docs/specifications/phase-l'
readonly -a specifications=(
  PHASE_L_AUTHORITATIVE_PLATFORM_CONSTITUTION.md
  PRESOLVE_PACKAGE_AND_CLI_SPECIFICATION.md
  PHASE_L_SLICES_L1_L10.md
  PHASE_L_SLICES_L11_L20.md
  PHASE_L_VERIFICATION_AND_RELEASE.md
  PHASE_L_L2_REPOSITORY_CONSTITUTION_AMENDMENT.md
  PHASE_L_L9_RECOVERY_AND_IMPLEMENTATION_CONTRACT.md
  PHASE_L_COMPLETION_EXECUTION_PLAN.md
  PHASE_L_REVISED_ROADMAP.md
  PHASE_L_L11_TOOLING_CAPABILITY_CONTRACT.md
  PHASE_L_L11_TRACE_AND_COST_CONTRACT.md
)

for specification in "${specifications[@]}"; do
  test -f "$specification_dir/$specification"
  rg --quiet --fixed-strings "$specification" "$specification_dir/README.md"
done
