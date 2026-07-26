set shell := ["bash", "-euo", "pipefail", "-c"]

format:
    pnpm run format

lint:
    pnpm run lint

test:
    pnpm run test

check:
    pnpm run check

e2e:
    pnpm run test:e2e

build:
    pnpm run build

release-check:
    pnpm run release:check

application-platform-check:
    pnpm --filter @presolve/vite test
    pnpm --filter create-presolve test
    cargo test -p presolve-cli --test representative_applications
    cargo test -p presolve-cli --test ergonomic_project
    cargo test -p presolve-cli --test runtime_browser decorator_free_v2_action_field_runs_through_compiler_artifacts_in_a_real_browser -- --nocapture
