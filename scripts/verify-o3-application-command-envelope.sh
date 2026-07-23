#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-o/PHASE_O_O3_APPLICATION_COMMAND_ENVELOPE.md
readonly package=metaframework/packages/application
test -s "$contract"
for phrase in 'APPLICATION_COMMAND_SCHEMA_VERSION' 'createApplicationCommandInvocation' 'invokeApplicationCommand' 'or manufacture success/failure objects'; do
  rg --fixed-strings --quiet "$phrase" "$contract" "$package/src"
done
node --test "$package/test/application-build-handoff.test.mjs"
cargo fmt --all --check
git diff --check
