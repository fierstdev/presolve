#!/usr/bin/env bash
set -euo pipefail

contract=docs/specifications/phase-l/PHASE_L_L10_TOOLING_SCHEMA_IMPLEMENTATION_CONTRACT.md
rg --quiet 'Canonical schema registry v1' "$contract"
rg --quiet 'Negotiation v1' "$contract"
rg --quiet 'presolve.build-trace' "$contract"
rg --quiet 'byte-for-byte unchanged' "$contract"
rg --quiet 'PHASE_L_L10_TOOLING_SCHEMA_IMPLEMENTATION_CONTRACT' docs/specifications/phase-l/README.md
