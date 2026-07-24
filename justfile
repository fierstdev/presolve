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
