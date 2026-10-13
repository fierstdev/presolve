# Chunk 21: public proof and release infrastructure

Chunk 21 turns Presolve's existing internal checkpoints into public evidence and begins distributing the CLI as versioned native binaries.

It does not broaden Presolve's application/runtime architecture.

## 21.1 — public CI evidence

Complete.

The public `CI` workflow exposes four independent jobs:

```text
Core
TypeScript
Architecture checkpoints
Docker portability
```

The Docker job exercises the real OCI path through an isolated local registry, immutable digest identity, Presolve materialization, Docker execution, and provider-neutral capability transport. The TypeScript OCI proof runs through the same path.

Live cloud credentials are not required by default CI.

## 21.2 — versioned CLI releases

21.2 is split deliberately:

```text
21.2a  release build/package infrastructure
21.2b  first tagged developer-preview release
```

### 21.2a release artifacts

The release workflow builds the native `presolve` CLI on four GitHub-hosted runner classes:

```text
x86_64-unknown-linux-gnu
 aarch64-unknown-linux-gnu
x86_64-apple-darwin
 aarch64-apple-darwin
```

Each platform produces:

```text
presolve-v<VERSION>-<TARGET>.tar.gz
presolve-v<VERSION>-<TARGET>.tar.gz.sha256
```

The archive contains:

```text
presolve
LICENSE
```

The packaged binary is executed with `--version` before the artifact is uploaded.

Pull requests and manual workflow runs build and upload temporary workflow artifacts only. They do not create GitHub Releases.

### Tag-gated publication

Publication is reserved for pushed `v*` tags.

Before publishing, the workflow requires:

```text
Git tag       = v<workspace.package.version>
CLI --version = presolve <workspace.package.version>
```

A mismatched tag fails instead of publishing an ambiguously versioned binary.

The publication job then downloads all four native archives, independently verifies their SHA-256 checksum files, and creates a GitHub prerelease containing the archives and checksums.

The first actual tag remains 21.2b so the release machinery can prove itself on a pull request before anything is published.

## Deliberate non-goals

21.2a does not add:

- Homebrew distribution
- shell installer scripts
- package-manager publication
- code signing or Apple notarization
- Windows binaries
- automatic stable-release promotion
- compatibility guarantees for the pre-stable CLI

Those can be evaluated after the first developer-preview binary release and external installation feedback.

## Configuration boundary

Release infrastructure adds no Presolve application configuration.

The normal durable application/environment files remain:

```text
presolve.toml
presolve.env.toml
```

GitHub Actions and release-packaging scripts are repository delivery infrastructure, not application semantics.
