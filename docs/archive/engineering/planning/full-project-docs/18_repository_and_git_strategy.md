# Repository and Git Strategy

## Recommendation

Use a **single canonical monorepo** for the core project, but do not manage it as one undifferentiated codebase.

The repository should be organized around **product boundaries**, not language boundaries. Rust and TypeScript should live together because the compiler, runtime, CLI, examples, language tools, tests, and documentation will evolve together during the first several years.

Do **not** split Rust and TypeScript into separate repositories at the beginning. The most important contracts in this project are cross-language contracts:

- compiler output manifests consumed by the runtime,
- generated DOM/update instructions consumed by browser code,
- CLI commands that invoke Rust compiler binaries from npm packages,
- fixture suites that prove TSX/html authoring compiles to correct HTML, runtime behavior, lazy chunks, and Web Component output,
- language-server diagnostics that must match compiler diagnostics,
- docs examples that must be compiled and tested against the current implementation.

Those contracts are easier to protect in one repository with one pull request, one CI graph, and one set of integration fixtures.

The repository can be large. That is acceptable if the repo is structured with strict boundaries, path-aware CI, sparse-checkout support, and separate release lanes.

## Repository shape

Recommended root layout:

```txt
edgezero/
  .github/
    workflows/
      validate.yml
      rust.yml
      typescript.yml
      integration.yml
      size.yml
      release.yml
    CODEOWNERS
  .changeset/
  crates/
    ezc/                       # Rust compiler CLI binary
    ezc_core/                  # compiler orchestration
    ezc_syntax/                # TSX/html template parsing bridge and source spans
    ezc_hir/                   # high-level semantic representation
    ezc_graph/                 # template/reactive/resource/a11y/style graphs
    ezc_ir/                    # lowered compiler IR
    ezc_analyzer/              # diagnostics, a11y, server/client validation
    ezc_codegen_dom/           # DOM/update code generation
    ezc_codegen_wc/            # Web Component output generation
    ezc_codegen_ssr/           # SSR/streaming output generation
    ezc_manifest/              # JSON manifests consumed by runtime/devtools
    ezc_runtime_contract/      # schemas and compatibility checks
    ezc_lsp/                   # language server
    ezc_dev_server/            # dev server integration
    ezc_testing/               # fixture runner and golden snapshot tools
    xtask/                     # repo automation: fixtures, release, docs checks
  packages/
    cli/                       # npm wrapper around native compiler binary
    runtime/                   # browser runtime: scheduler, signals, delegation
    server/                    # JS server adapters and request/runtime glue
    vite/                      # Vite plugin
    rollup/                    # Rollup plugin, if needed separately
    webpack/                   # Webpack/Rspack adapter, later
    language-tools/            # editor integration package wrapper
    devtools/                  # inspector bridge / browser extension source
    create-edgezero/           # project scaffold generator
    eslint-plugin-edgezero/    # optional lint rules that complement compiler checks
  schemas/
    compiler-manifest.schema.json
    trace.schema.json
    a11y-diagnostic.schema.json
  examples/
    counter/
    forms/
    resource-streaming/
    dashboard/
    wc-library/
    server-actions/
  fixtures/
    compiler/
    runtime/
    resumability/
    a11y/
    wc-output/
    server-client-split/
  e2e/
    browser/
    node-server/
    edge-runtime/
  docs/
    src/
    public/
  rfcs/
    0001-semantic-ui-graph.md
    0002-resumability-manifest.md
  adr/
    0001-monorepo.md
  benches/
    compile-time/
    runtime-size/
    hydration-resume/
  scripts/
  Cargo.toml
  Cargo.lock
  rust-toolchain.toml
  pnpm-workspace.yaml
  package.json
  turbo.json                  # optional after task graph becomes useful
  nx.json                     # use either Nx or Turborepo, not both
  justfile
  mise.toml                   # or .tool-versions; choose one
  README.md
```

## First decision: monorepo, but layered

The repo should have four layers.

### Layer 1: compiler core

Lives under `crates/`.

This is the source of truth for:

- parsing and source spans,
- semantic model construction,
- graph extraction,
- diagnostics,
- lowering to IR,
- output manifests,
- SSR/codegen decisions,
- resumability serialization rules,
- accessibility analysis,
- server/client split analysis.

Compiler crates should not depend on TypeScript packages. They can emit schemas, manifests, JS files, source maps, and diagnostics, but they should not import runtime package internals directly.

### Layer 2: runtime and adapters

Lives under `packages/`.

This includes:

- browser runtime,
- npm CLI wrapper,
- dev-server integrations,
- bundler adapters,
- server adapters,
- devtools bridge,
- scaffold generator.

Runtime packages should treat compiler outputs as versioned contracts. They should consume manifest schemas rather than implicit compiler implementation details.

### Layer 3: contract fixtures

Lives under `fixtures/`, `examples/`, and `e2e/`.

This is the most important layer for a compiler-centered framework. Fixtures should prove:

- exact generated HTML,
- exact resumability manifest shape,
- exact lazy chunk boundaries,
- exact accessibility diagnostics,
- exact server/client split errors,
- exact Web Component output behavior,
- runtime behavior in real browsers.

Golden tests are not optional. They are the product safety net.

### Layer 4: docs, RFCs, and design history

Lives under `docs/`, `rfcs/`, and `adr/`.

The docs should remain in the main repo until the framework is mature because examples must compile against `main`. A separate marketing site repo can come later, but the technical docs should stay close to the implementation.

## Workspace management

Use both native workspaces:

```txt
Cargo workspace for Rust crates.
pnpm workspace for TypeScript packages.
```

The root should be the control plane, not a build product.

Root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
  "crates/ezc",
  "crates/ezc_core",
  "crates/ezc_syntax",
  "crates/ezc_hir",
  "crates/ezc_graph",
  "crates/ezc_ir",
  "crates/ezc_analyzer",
  "crates/ezc_codegen_dom",
  "crates/ezc_codegen_wc",
  "crates/ezc_codegen_ssr",
  "crates/ezc_manifest",
  "crates/ezc_runtime_contract",
  "crates/ezc_lsp",
  "crates/ezc_dev_server",
  "crates/ezc_testing",
  "crates/xtask",
]

[workspace.package]
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/edgezero/edgezero"

[workspace.dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
tracing = "0.1"
```

Root `pnpm-workspace.yaml`:

```yaml
packages:
  - "packages/*"
  - "examples/*"
  - "docs"
```

Root `package.json`:

```json
{
  "private": true,
  "packageManager": "pnpm@latest",
  "scripts": {
    "check": "just check",
    "test": "just test",
    "build": "just build",
    "fixtures": "cargo xtask fixtures",
    "size": "cargo xtask size"
  },
  "devDependencies": {
    "typescript": "latest"
  }
}
```

Use `just` or `cargo xtask` as the human-facing task layer. Developers should not need to memorize different commands for Rust, pnpm, docs, fixtures, and browser tests.

Recommended command surface:

```sh
just setup
just check
just test
just test-rust
just test-ts
just fixtures
just e2e
just docs
just size
just release-dry-run
```

## Build orchestration

Start with native tools and a thin task layer:

```txt
Cargo + pnpm + just + GitHub Actions path filters.
```

Do not start with Bazel. Do not start with a custom build system. Do not make Nx or Turborepo a foundational dependency on day one.

Add a monorepo task orchestrator only when one of these becomes painful:

- CI repeatedly rebuilds unchanged packages,
- fixture runs become too slow,
- many packages need dependency-aware task scheduling,
- remote caching saves real time,
- affected-package detection becomes hard to maintain manually.

If the repo remains mostly TypeScript-heavy at the adapter/docs/examples layer, Turborepo is enough. If the repo becomes a broad polyglot workspace with many non-JS tasks, Nx is more appropriate. Choose one. Do not run Nx and Turborepo together unless there is a very specific reason.

The initial task graph should look like this:

```txt
format
  ├─ rustfmt
  └─ prettier

lint
  ├─ clippy
  ├─ ts typecheck
  └─ eslint, if used

test
  ├─ cargo test
  ├─ pnpm test
  ├─ fixture snapshots
  └─ browser e2e

build
  ├─ cargo build --release -p ezc
  ├─ pnpm build packages/*
  ├─ docs build
  └─ examples build
```

## CI strategy

Use path-aware workflows, but keep one required top-level status check called something like `validate`.

Recommended workflows:

```txt
validate.yml
  Fast universal checks:
    - formatting
    - manifest/schema validation
    - workspace dependency sanity
    - generated-file drift checks

rust.yml
  Runs when crates/**, Cargo.*, rust-toolchain.toml, fixtures/**, schemas/** change.

typescript.yml
  Runs when packages/**, examples/**, docs/**, package.json, pnpm-lock.yaml change.

integration.yml
  Runs compiler/runtime fixtures and browser tests.
  Required for changes touching compiler, runtime, server adapters, or fixtures.

size.yml
  Tracks runtime size, initial JS size, generated chunks, compiler binary size.
  Required on PRs after the MVP stabilizes.

release.yml
  Only runs on release PRs/tags.
```

Path filters are an optimization, not a correctness boundary. Any change to `schemas/`, `fixtures/`, `crates/ezc_manifest`, `packages/runtime`, or `packages/cli` should trigger integration tests because these areas define cross-language contracts.

## Git workflow

Use trunk-based development with protected `main`.

Recommended rules:

- all work happens in pull requests,
- `main` is always releasable or close to releasable,
- squash merge by default,
- require linear history,
- require `validate` and relevant affected checks,
- require review from CODEOWNERS for compiler/runtime/release changes,
- require changeset or release note marker for public package changes,
- allow draft PRs for design exploration,
- use RFCs for large semantic model, syntax, and runtime contract changes.

Branch naming:

```txt
feat/semantic-graph-bindings
fix/a11y-label-diagnostic
refactor/ir-source-spans
chore/update-rust-toolchain
rfc/resumability-manifest-v1
```

Commit style:

Use Conventional Commits for commits that affect public release notes:

```txt
feat(compiler): infer lazy event boundary for form submit
fix(runtime): preserve delegated event target during resume
perf(codegen): avoid emitting static text patch operation
docs(forms): clarify native fallback behavior
```

Do not be dogmatic about every internal commit if squash merging. The squash commit title should follow the convention.

## Versioning and release policy

Use a **single release train** before 1.0.

That means official crates and npm packages that are part of the supported product move together:

```txt
edgezero v0.4.0
@edgezero/runtime v0.4.0
@edgezero/cli v0.4.0
@edgezero/server v0.4.0
ezc v0.4.0
```

This is less elegant than independent versioning, but it is simpler for early adopters and simpler for cross-language compatibility. A compiler-centered framework has many moving contracts; avoid making users solve compatibility matrices while the system is young.

Before 1.0:

- prefer one product version,
- publish only the packages users need,
- keep most Rust internals unpublished unless external plugin authors truly need them,
- publish nightly/canary builds from `main`,
- publish stable prereleases manually or through a release PR,
- write migration notes for every breaking authoring or manifest change.

After 1.0:

- keep the main framework packages synchronized,
- allow adapters to version independently,
- allow experimental packages under `@edgezero-labs/*`,
- stabilize compiler manifests with explicit compatibility versions.

Recommended release lanes:

```txt
stable
  Semver release from a release PR.

canary
  Every successful main build, tagged with commit SHA/date.

experimental
  Opt-in packages or feature flags. No compatibility guarantee.
```

Recommended tags:

```txt
v0.4.0
v0.4.0-canary.20260707.shaabcdef
```

Avoid per-package tags at the beginning. They are useful later but add early coordination cost.

## Publishing strategy

### npm

Publish:

```txt
edgezero                  # optional convenience CLI package, if name is available
@edgezero/cli             # primary CLI package
@edgezero/runtime         # browser runtime
@edgezero/server          # server/runtime glue
@edgezero/vite            # first bundler adapter
@edgezero/create          # scaffold package
@edgezero/language-tools  # editor tooling wrapper
```

The npm CLI package should either:

1. download or include a platform-specific native compiler binary, or
2. depend on optional platform packages such as `@edgezero/cli-darwin-arm64`, `@edgezero/cli-linux-x64`, etc.

The second model is usually better for installation speed and reproducibility.

### crates.io

Publish fewer Rust crates initially.

Good initial candidates:

```txt
ezc                 # compiler CLI, if useful outside npm
edgezero_manifest   # stable manifest schema library, only if plugin authors need it
edgezero_lsp        # language server, if distributed independently
```

Keep internal crates private until APIs stabilize. Public compiler internals create semver obligations too early.

### VS Code / editor extensions

Keep source in the monorepo. Publish from CI. The extension should consume a released language server binary or package from the same release train.

### Docs site

Keep technical docs source in the monorepo. A separate deployment repository is unnecessary unless the hosting platform requires it.

## Dependency policy

Use central dependency policy, but avoid over-centralizing prematurely.

Rust:

- define common versions in `[workspace.dependencies]`,
- use `workspace = true` for shared dependencies,
- keep dependency features narrow,
- audit proc macros and build scripts carefully,
- deny unused dependencies in CI once the crate graph stabilizes.

TypeScript:

- use pnpm workspaces,
- use a single lockfile,
- use dependency catalogs if the workspace benefits from shared version ranges,
- keep runtime dependencies minimal,
- keep devtool/build dependencies out of browser runtime packages.

Cross-language:

- schemas are contracts,
- generated files must be checked for drift,
- compiler/runtime compatibility should be tested by fixtures,
- the runtime should reject incompatible manifest versions with clear errors.

## Ownership model

Use CODEOWNERS by subsystem:

```txt
/crates/ezc_graph/             @edgezero/compiler
/crates/ezc_analyzer/          @edgezero/compiler @edgezero/a11y
/crates/ezc_codegen_*/         @edgezero/compiler
/packages/runtime/             @edgezero/runtime
/packages/server/              @edgezero/server
/packages/vite/                @edgezero/adapters
/fixtures/                     @edgezero/compiler @edgezero/runtime
/schemas/                      @edgezero/compiler @edgezero/runtime
/docs/                         @edgezero/docs
/.github/workflows/release.yml @edgezero/release
```

The ownership model should reflect review expertise, not corporate hierarchy.

## RFC and ADR policy

Use RFCs for decisions that affect users or long-term architecture:

- authoring syntax,
- graph model semantics,
- resumability manifest format,
- server/client split rules,
- accessibility error policy,
- plugin APIs,
- output target contracts,
- package/versioning policy.

Use ADRs for implementation and operating decisions:

- monorepo decision,
- pnpm vs npm/yarn,
- release automation,
- native binary packaging,
- fixture snapshot format,
- CI provider decisions.

RFC lifecycle:

```txt
Draft → Accepted → Implementing → Stabilized → Superseded
```

Each RFC should include:

- problem,
- goals,
- non-goals,
- proposed design,
- alternatives rejected,
- compiler implications,
- runtime implications,
- accessibility implications,
- migration strategy,
- testing strategy.

## Testing model

The project should be fixture-driven.

### Rust unit tests

Use for parser, graph extraction, analyzer, codegen helpers, source maps, and manifest compatibility.

### Compiler golden tests

Input:

```txt
component source
build target
compiler flags
```

Expected output:

```txt
HTML
manifest
chunks
warnings/errors
source maps
explain output
```

### Runtime tests

Run in browser-like environments for small behavior and in real browsers for resumability/event tests.

### Integration tests

Compile examples, run them, and verify:

- initial HTML,
- no unnecessary client JS,
- lazy chunks load only on interaction,
- form fallback works without JavaScript,
- enhanced form works with JavaScript,
- accessibility diagnostics are stable,
- Web Component output works in a plain HTML page.

### Size tests

Track:

- baseline loader size,
- per-interaction chunk size,
- runtime package size,
- compiler binary size,
- generated HTML size,
- manifest size.

A compiler-first framework needs size regressions treated as product regressions.

## Large-repository controls

A monorepo becomes painful if it accumulates generated artifacts, binary fixtures, and stale examples.

Rules:

- do not commit build output,
- do not commit benchmark traces unless intentionally curated,
- store large binary artifacts outside Git,
- use Git LFS only for unavoidable binary fixtures,
- keep fixtures text-based wherever possible,
- prune obsolete examples aggressively,
- add sparse-checkout documentation for contributors who only work on docs, runtime, or compiler.

Recommended sparse-checkout examples:

```sh
# Compiler-only checkout
git sparse-checkout set crates fixtures schemas Cargo.toml Cargo.lock rust-toolchain.toml justfile

# Runtime/adapters checkout
git sparse-checkout set packages examples fixtures schemas package.json pnpm-lock.yaml pnpm-workspace.yaml justfile

# Docs-only checkout
git sparse-checkout set docs examples README.md package.json pnpm-lock.yaml pnpm-workspace.yaml
```

## When to split into separate repositories

Do not split because the repo feels conceptually large. Split only when there is a real ownership, release, or security boundary.

Good split candidates later:

```txt
edgezero.dev marketing site
community examples gallery
third-party adapters
experimental visual builder
cloud service / hosted inspector backend
large benchmark corpus
playground infrastructure
```

Bad early split candidates:

```txt
compiler
runtime
CLI
language tools
fixtures
docs reference examples
```

Those should remain together until contracts stabilize.

## Anti-patterns to avoid

### Language-based repos

Bad:

```txt
edgezero-rust
edgezero-js
edgezero-docs
```

This creates coordination overhead exactly where the project needs tight coupling.

### Submodules for first-party packages

Avoid Git submodules for core packages. They add operational friction and make atomic cross-package changes harder.

### Publishing all internal crates

Do not publish every compiler crate. It turns internal architecture into public API before the semantic model is ready.

### Independent versions too early

Independent versions make maintainers feel sophisticated and users feel confused. Use one product version until compatibility boundaries are stable.

### Generated-code drift

If schemas, manifests, docs examples, or golden outputs can drift from source, CI must detect it.

## Initial implementation plan

### Week 1: repository skeleton

- create monorepo,
- add root Cargo workspace,
- add pnpm workspace,
- add `justfile`,
- add Rust and TypeScript formatting,
- add `validate.yml`,
- add CODEOWNERS,
- add `adr/0001-monorepo.md`.

### Week 2: compiler/runtime contract spine

- add minimal compiler CLI crate,
- add manifest schema,
- add runtime package stub,
- add one fixture that compiles `Counter.tsx` to HTML + manifest,
- add generated-file drift check.

### Week 3: package and release skeleton

- add npm CLI wrapper,
- add canary publishing dry run,
- add version metadata command,
- add release notes placeholder flow,
- add first docs build.

### Week 4: integration safety net

- add browser e2e runner,
- add size baseline,
- add `fw explain` golden fixture,
- add CI matrix for Linux/macOS/Windows compiler binary smoke tests.

## Final policy

The right repository model is:

```txt
One monorepo.
Two native workspaces.
Strict product boundaries.
Compiler/runtime contracts as schemas and fixtures.
One release train before 1.0.
Path-aware CI.
Sparse-checkout support.
Split only when ownership or release independence becomes real.
```

This keeps the project coherent while still allowing Rust and TypeScript specialists to work in their own areas without forcing everyone to load the entire system mentally.
