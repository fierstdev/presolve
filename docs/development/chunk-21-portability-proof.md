# Chunk 21 portability proof

Chunk 21 moves Presolve from internal architectural checkpoints toward evidence that an external developer can inspect and reproduce.

## 21.3 thesis

The flagship proof is stricter than building the same source for multiple targets:

> Build one immutable application release once. Resolve and realize those exact bytes in multiple environments without rebuilding the application.

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

## 21.3b — same-release realization

21.3b connects the detached release directly to runtime execution.

The proof path is:

```text
one kv-demo.presolved archive
        ↓
decode + verify bundle
        ↓
reconstruct resolver input from release manifest
        ↓
resolve against selected Environment Specification
        ↓
DeploymentPlan with exact ProviderId
        ↓
realize that provider implementation
        ↓
read the embedded Component from the same verified release bytes
        ↓
assemble runtime from the DeploymentPlan + ProviderRegistry
        ↓
execute the exact bundled Component
```

The checkpoint intentionally does not rebuild from source after the `.presolved` archive is produced.

### Provider-specific realization remains outside the release

The local path realizes the plan-selected KV provider with Presolve's in-memory implementation.

The managed path realizes the plan-selected provider through the existing `CloudflareKvAdapter`. Cloudflare operational inputs remain process-local:

```text
PRESOLVE_CLOUDFLARE_ACCOUNT_ID
PRESOLVE_CLOUDFLARE_KV_NAMESPACE_ID
PRESOLVE_CLOUDFLARE_API_TOKEN
```

These values do not enter:

- `presolve.toml`;
- `presolve.env.toml`;
- `DeploymentPlan`;
- the `.presolved` release; or
- application release identity.

### Visibility-aware KV demo protocol

The same guest Component must run against both the immediate in-memory provider and eventually consistent Cloudflare Workers KV without claiming stronger semantics than the managed provider offers.

For that reason the `kv-demo` guest exposes simple `set`, `get`, and `delete` commands. The checkpoint performs one write, then:

- requires immediate visibility for the local in-memory realization;
- polls the same guest `get` operation for the Cloudflare realization until the value becomes visible within a bounded timeout; and
- runs guest-side cleanup after visibility is confirmed.

The application bytes remain identical. Only the selected environment and provider realization differ.

### Public non-live checkpoint

Run:

```bash
bash ./scripts/check-release-realization.sh
```

This path is suitable for public CI. It:

1. builds `kv-demo.presolved` exactly once;
2. records its canonical release digest and archive SHA-256;
3. resolves the detached release against the fixed local-memory environment;
4. realizes the exact plan-selected ProviderId with the in-memory KV implementation;
5. reads and executes the embedded Component directly from the verified `.presolved` release bytes;
6. proves guest KV write/read/delete behavior through the realized provider; and
7. verifies that the `.presolved` archive bytes remain unchanged after execution.

No extracted `.wasm` sidecar is passed into the runtime checkpoint. The detached `.presolved` release is the sole application artifact supplied to execution.

### Live same-release Cloudflare proof

With Cloudflare realization variables set, run:

```bash
bash ./scripts/check-release-realization.sh --cloudflare
```

The script still builds the release exactly once. It first executes that release through the local-memory environment, then executes the **same file** through the managed Cloudflare environment and real Cloudflare KV adapter. It verifies the same release digest in both outputs and requires the archive SHA-256 to remain unchanged after both executions.

The proof therefore has the intended shape:

```text
same kv-demo.presolved bytes
same release digest
        |
        +-- local Environment Specification
        |       ↓
        |   DeploymentPlan
        |       ↓
        |   memory-kv realization
        |       ↓
        |   Component execution
        |
        +-- managed Cloudflare Environment Specification
                ↓
            DeploymentPlan
                ↓
            CloudflareKvAdapter realization
                ↓
            real Workers KV
                ↓
            same Component execution
```

Public CI exercises the credential-free local detached-release realization path. The live Cloudflare half remains opt-in because Presolve does not place production credentials in the repository or release artifact.

## 21.3b completion criterion

21.3b is complete when the live `--cloudflare` path has been run successfully with real Cloudflare credentials and both realizations report the same detached release identity while the archive hash remains unchanged.

That acceptance run completed successfully on 2026-10-13. The credential-free evidence record is stored in `docs/proofs/chunk-21-3b-live-cloudflare.md`.

Presolve therefore has evidence for the stronger flagship claim: one immutable application release can be resolved **and actually executed** through materially different provider implementations without changing the release.

## 21.3c — proof diagnostics

The live acceptance run exposed a useful diagnostic weakness: the guest demo originally reduced provider errors to only `unavailable`, `invalid-key`, or `internal`, discarding the provider-supplied message carried by the WIT error payload.

The demo now preserves that message in its textual proof protocol. This does not change capability semantics; it makes failures such as authorization, throttling, timeout, or provider-side status errors observable during a live proof without exposing credentials.

## 21.3d — direct release execution

The original 21.3b script extracted the Component to a temporary `.wasm` file before invoking the runtime checkpoint. The extracted bytes were verified against the release manifest, so the proof remained sound, but the sidecar weakened the shape of the demonstration: execution appeared to accept both the release and an additional application artifact.

21.3d removes that ambiguity.

The runtime checkpoint now accepts only:

```text
<realization mode> <release.presolved> <presolve.env.toml> <application input>
```

It decodes and verifies the release, identifies the single Component artifact from the verified manifest, reopens the same in-memory release bytes, reads that embedded artifact, resolves the release, realizes the selected provider, assembles the runtime, and executes the Component.

The shell checkpoint no longer extracts or passes a `.wasm` sidecar. The `.presolved` file is therefore the sole application release input across the complete proof path.
