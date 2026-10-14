# Chunk 21.3b live same-release realization proof

Date: 2026-10-13

This record captures the manual live acceptance run for Chunk 21.3b. It intentionally excludes credentials, account identifiers, namespace identifiers, temporary filesystem paths, and other environment-specific secrets.

## Claim under test

A single immutable `.presolved` application release can be built once and then resolved, realized, and executed against materially different provider implementations without rebuilding or mutating the release.

For this acceptance run, the two realizations were:

- local in-memory KV
- live Cloudflare Workers KV through Presolve's Cloudflare KV provider adapter

## Command

```bash
bash ./scripts/check-release-realization.sh --cloudflare
```

The following environment variables were supplied externally to Presolve and were not embedded in the Application Contract, Environment Specification, DeploymentPlan, or `.presolved` release:

- `PRESOLVE_CLOUDFLARE_ACCOUNT_ID`
- `PRESOLVE_CLOUDFLARE_KV_NAMESPACE_ID`
- `PRESOLVE_CLOUDFLARE_API_TOKEN`

## Immutable release identity

The release was built exactly once.

```text
release digest: sha256:1ffc6885e405e71fc5d18fca8b55a7768f52ed9328c2c48b1de5061a2db2dee6
archive sha256: 719b6a1ce41715f84096022f4b7546c94a4227e2bf88457d0f614caca0b44577
```

The archive SHA-256 was checked again after both executions and remained unchanged:

```text
archive sha256 unchanged: 719b6a1ce41715f84096022f4b7546c94a4227e2bf88457d0f614caca0b44577
```

## Local realization

```text
environment: portability-local (env_00000000-0000-4000-8000-000000000301)
provider: memory-kv (prv_00000000-0000-4000-8000-000000000311)
mode: local
output: value:<run-id>-local
```

The release digest reported during execution matched the original detached release digest.

## Live Cloudflare realization

```text
environment: portability-cloudflare (env_00000000-0000-4000-8000-000000000302)
provider: cloudflare-kv (prv_00000000-0000-4000-8000-000000000312)
mode: cloudflare
output: value:<run-id>-cloudflare
```

The same detached release digest was reported for the Cloudflare realization. No application rebuild occurred between the local and Cloudflare runs.

## Acceptance result

```text
same-release local + Cloudflare realization proof passed
```

Chunk 21.3b therefore establishes an executable portability proof, not only a resolver proof:

1. Build one immutable release.
2. Decode and verify that release.
3. Reconstruct application requirements from the release manifest.
4. Resolve those requirements against an Environment Specification.
5. Preserve the exact plan-selected `ProviderId` into provider realization.
6. Assemble the runtime with the realized provider implementation.
7. Execute the same application component through local memory KV.
8. Execute the same release through live Cloudflare KV.
9. Verify that the release archive bytes remain unchanged.

This proves the targeted Chunk 21 claim: deployment-provider choice can change after application release creation without changing the application release itself.

## Scope

This proof does not claim that every Presolve capability or provider is portable across every environment. It proves the narrower, concrete property exercised by this fixture: the same immutable KV-capable application release can be realized against two different provider implementations, including a live external provider, without rebuild.
