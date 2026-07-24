#!/usr/bin/env bash

set -euo pipefail

readonly -a required_directories=(
  .github adr crates docs e2e examples fixtures framework notes packages rfcs schemas scripts site
)
readonly -a required_control_files=(
  Cargo.toml Cargo.lock package.json pnpm-lock.yaml pnpm-workspace.yaml justfile
  rust-toolchain.toml .gitattributes .gitignore README.md SUPPORT.md
)
readonly -a forbidden_directories=(compiler runtime cli)

for directory in "${required_directories[@]}"; do
  test -d "$directory"
  rg --quiet --fixed-strings "\`$directory/\`" docs/repository-layout.md
done

for control_file in "${required_control_files[@]}"; do
  test -f "$control_file"
done

for directory in "${forbidden_directories[@]}"; do
  test ! -e "$directory"
done

test ! -e docs/planning
test -f docs/archive/engineering/README.md
test -f docs/archive/engineering/planning/sprint-zero.md
test -f docs/archive/engineering/resources/index.md
test -f docs/archive/engineering/spikes/accepted/parser-backend-evaluation.md
test -f notes/progress/AGENT_HANDOFF.md
test -f notes/progress/2026-W28.md

./scripts/verify-phase-l-specifications.sh

if rg --line-number 'docs/archive/engineering/' README.md; then
  echo 'historical engineering material must not appear in public README navigation' >&2
  exit 1
fi

if rg --line-number --glob '!verify-repository-layout.sh' 'docs/archive/' Cargo.toml package.json pnpm-workspace.yaml justfile .github scripts; then
  echo 'active automation must not point into the engineering archive' >&2
  exit 1
fi

if find docs/archive/engineering -type d \( -name schemas -o -name fixtures \) -print | grep -q .; then
  echo 'schemas and fixtures must remain active verification assets' >&2
  exit 1
fi

if git ls-files | rg --line-number '(^|/)(target|node_modules|dist|\.astro|test-results)/|(^|/)\.env($|\.)|\.(pem|key)$|credentials'; then
  echo 'tracked generated output, machine-local state, or credentials found' >&2
  exit 1
fi

readonly expected_roots=$'.gitattributes\n.github\n.gitignore\nCHANGELOG.md\nCODE_OF_CONDUCT.md\nCONTRIBUTING.md\nCargo.lock\nCargo.toml\nLICENSE\nREADME.md\nSECURITY.md\nSUPPORT.md\nadr\ncrates\ndocs\ne2e\nexamples\nfixtures\nframework\njustfile\nmetaframework\nnotes\npackage.json\npackages\npnpm-lock.yaml\npnpm-workspace.yaml\nrfcs\nrust-toolchain.toml\nschemas\nscripts\nsite'
actual_roots="$(git ls-files | awk -F/ '{print $1}' | sort -u)"
if [[ "$actual_roots" != "$expected_roots" ]]; then
  echo 'tracked root ownership differs from docs/repository-layout.md' >&2
  diff -u <(printf '%s\n' "$expected_roots") <(printf '%s\n' "$actual_roots") >&2 || true
  exit 1
fi
