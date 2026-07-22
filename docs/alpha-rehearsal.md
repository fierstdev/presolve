# Presolve alpha clean-room rehearsal

**Status:** L19-B recorded rehearsal procedure. It creates no project and does
not publish, sign, upload, deploy, or contact a registry. The supported starter
path is a fresh checkout of this repository because `presolve create` is
reserved and exits `6`.

## Manual starter path

From a clean checkout at the committed alpha revision, install only local
dependencies and build the committed CLI:

```sh
pnpm install --offline
cargo build -p presolve-cli
```

Run the first four examples through their chained explicit verifier, then use
the built CLI for the accepted version, check, build, workspace, watch, cache,
and production/resume paths:

```sh
./scripts/verify-l14b-explicit-workspace-example.sh
target/debug/presolve version --format json
target/debug/presolve check --config examples/counter/presolve.json --source counter.tsx=src/Counter.tsx --format json
target/debug/presolve build --config examples/forms/presolve.json --source Forms.tsx=src/Forms.tsx --format json
target/debug/presolve workspace --config examples/explicit-workspace/presolve.json --source src/main.ts=src/main.ts --format json
target/debug/presolve watch --once --config examples/components-context-slots/presolve.json --source Composition.tsx=src/Composition.tsx --format json
target/debug/presolve cache inspect --config examples/counter/presolve.json --format json
target/debug/presolve cache verify --config examples/counter/presolve.json --format json
target/debug/presolve clean --config examples/counter/presolve.json --format json
target/debug/presolve build examples/production-resume/src/ComputedDiamond.tsx --out examples/production-resume/.presolve --production
```

The final build must contain `production.runtime.json`, `resume.runtime.json`,
and the production directory at their frozen schema versions. The existing
runtime browser proof remains a separate Chrome-owned gate; this rehearsal
checks the committed Phase K artifact shape and corpus inputs without claiming
an environment-independent browser runtime.

Then exercise every accepted L11 product command through its committed fixture
verifier and confirm private package metadata:

```sh
./scripts/verify-l11c-tooling-commands.sh
./scripts/verify-l11g-artifact-graph-command.sh
./scripts/verify-l11g-trace-command.sh
./scripts/verify-l11g-profile-command.sh
```

Every package remains private at `0.1.0-alpha`; the rehearsal does not produce
a release artifact or external side effect.

## Recorded evidence

`./scripts/verify-l19b-clean-room-rehearsal.sh` creates a detached clean
worktree at `HEAD`, performs the recorded offline package, example, command,
product, package-metadata, and Phase K artifact checks, and removes the
worktree—even after a failed check. It is a clean-room replay, not a create,
publication, signing, hosting, or deployment path.
