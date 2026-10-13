# Chunk 21 portability proof

Chunk 21 moves Presolve from internal architectural checkpoints toward evidence that an external developer can inspect and reproduce.

## 21.3 thesis

The flagship proof is stricter than building the same source for multiple targets:

> Build one immutable application release once. Resolve those exact bytes in multiple environments without rebuilding the application.

The proof therefore starts from a `.presolved` release, not from the source `presolve.toml` used to create it.

## 21.3a — detached-release resolution

The first slice proves that the immutable release contains enough canonical application semantics to be resolved independently of its source tree.

`presolve check` and `presolve plan` accept an optional detached release:

```bash
presolve check dist/app.presolved
presolve plan dist/app.presolved

presolve check dist/app.presolved --json
presolve plan dist/app.presolved --json
```

When no release path is supplied, both commands retain their existing nearest-Application-Contract behavior.

The bundle manifest is reconstructed into the resolver's Application Contract input using only canonical semantics stored in the release. Source paths, build configuration, credentials, provider realization settings, and deployment decisions are not consulted.

### Public checkpoint

Run:

```bash
bash ./scripts/check-portability-proof.sh
```

The checkpoint:

1. builds the `kv-demo` application into one temporary `.presolved` release exactly once;
2. records both the canonical Presolve release digest and the SHA-256 of the encoded archive bytes;
3. inspects the detached release;
4. resolves those exact release bytes against `examples/portability-proof/local-memory/presolve.env.toml`;
5. resolves the same release bytes against `examples/portability-proof/managed-cloudflare/presolve.env.toml`;
6. requires both compatibility checks and DeploymentPlans to identify the detached release rather than a source contract;
7. requires the local plan to select `memory-kv` with its fixed EnvironmentId and ProviderId;
8. requires the managed plan to select `cloudflare-kv` with a different fixed EnvironmentId and ProviderId; and
9. recomputes the archive SHA-256 after each resolution and requires it to remain unchanged.

There is no second application build between the two plans.

The expected shape is:

```text
kv-demo.presolved
release digest: sha256:<one canonical identity>
archive SHA-256: <one byte identity>
        |
        +-- portability-local
        |     env_...0301
        |     -> memory-kv / prv_...0311
        |
        +-- portability-cloudflare
              env_...0302
              -> cloudflare-kv / prv_...0312
```

The two Environment Specifications are intentionally checked into the repository with fixed IDs so public CI can prove that environment/provider selection changes while application release identity does not.

## What 21.3a proves

21.3a proves:

- a `.presolved` release is sufficient input for compatibility and planning;
- the original Application Contract does not need to be present at resolution time;
- one immutable release can produce different DeploymentPlans in different environments;
- provider selection belongs to the environment/deployment side of the boundary; and
- resolving a release does not mutate or rebuild it.

## What 21.3a does not prove

The managed Cloudflare fixture in 21.3a is an Environment Specification and provider selection target. It does **not** claim that CI contacted Cloudflare or executed the workload against a live Workers KV namespace.

Live provider realization requires account/namespace/token material that deliberately does not belong in the Environment Specification or `.presolved` release.

That is the next slice:

## 21.3b — same-release realization

21.3b will take the same detached-release model and prove materially different realizations:

- local execution backed by the in-memory KV provider; and
- managed execution/materialization backed by the real Cloudflare KV adapter.

The release must remain byte-identical across both realizations. Any provider credentials or operational settings must remain external to the application release and Environment Specification.

21.3b is not complete until the managed path executes through the real adapter rather than merely resolving to a provider declaration.
