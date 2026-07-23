#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

readonly contract=docs/specifications/phase-o/PHASE_O_O2_EXPLICIT_WORKSPACE_DEVELOPMENT_HANDOFF.md
readonly package=metaframework/packages/application
test -s "$contract"
for phrase in 'createApplicationWorkspaceInvocation' 'createApplicationWatchOnceInvocation' 'does not open a file watcher' 'presolve watch --once'; do
  rg --fixed-strings --quiet "$phrase" "$contract" "$package/src"
done
node --test "$package/test/application-build-handoff.test.mjs"
rg --fixed-strings --quiet '"workspace" => run_l9_workspace' crates/presolve_cli/src/main.rs
rg --fixed-strings --quiet '"watch" => run_l9_watch' crates/presolve_cli/src/main.rs
cargo fmt --all --check
git diff --check
