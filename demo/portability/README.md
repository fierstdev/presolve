# Presolve flagship portability demo

This is the shortest public demonstration of Presolve's core claim:

> Build one immutable application release once. Decide where it runs later.

The demo builds one `kv-demo.presolved` release exactly once, records its canonical Presolve release digest and archive SHA-256, resolves those exact bytes against three different environments, and executes the same embedded application through materially different provider implementations.

## What the demo proves

One application release is used throughout the entire run:

```text
kv-demo.presolved
same release digest
same archive bytes
        |
        +-- portability-local
        |      plan -> memory-kv
        |      execute -> in-process provider
        |
        +-- portability-redis
        |      plan -> redis-kv
        |      execute -> Redis over TCP
        |
        +-- portability-cloudflare
               plan -> cloudflare-kv
               execute -> live Workers KV when --cloudflare is supplied
```

No application rebuild occurs between environments. Provider selection, provider realization settings, credentials, and deployment decisions remain outside the `.presolved` release.

## Run it

Prerequisites:

- Rust toolchain from this repository's `rust-toolchain.toml`
- `python3`
- Docker

From the repository root:

```bash
bash ./demo/portability/run.sh
```

The default run is fully credential-free. It:

1. builds the Presolve CLI and runtime checkpoint tooling;
2. builds one immutable `kv-demo.presolved` release;
3. resolves that release against local-memory, network-Redis, and managed-Cloudflare Environment Specifications;
4. starts a pinned `redis:7.4-alpine` container;
5. executes the same release through the in-memory KV provider;
6. executes the same release through Redis over TCP; and
7. verifies both the Presolve release digest and the encoded archive SHA-256 are unchanged.

The Cloudflare environment participates in planning even in the credential-free run, proving that the same release resolves to a third ProviderId without introducing cloud credentials or configuration into application identity.

## Add the live managed-cloud execution

To execute the exact same release through Cloudflare Workers KV as well, provide realization-time credentials and add `--cloudflare`:

```bash
export PRESOLVE_CLOUDFLARE_ACCOUNT_ID="..."
export PRESOLVE_CLOUDFLARE_KV_NAMESPACE_ID="..."
export PRESOLVE_CLOUDFLARE_API_TOKEN="..."

bash ./demo/portability/run.sh --cloudflare
```

These values are consumed only at provider realization time. The script does not print them, persist them into `presolve.toml`, write them into `presolve.env.toml`, put them into `DeploymentPlan`, or include them in the `.presolved` release.

## What to look for

A successful credential-free run ends with output shaped like:

```text
Presolve flagship portability demonstration
==========================================
Claim: build one immutable application release once; decide where it runs later.

[2/5] Build one immutable application release
  release digest: sha256:<one identity>
  archive sha256: <one byte identity>

[3/5] Resolve the same release against three environments
  environment              selected provider  plan
  ------------------------ ------------------ --------
  portability-local        memory-kv          resolved
  portability-redis        redis-kv           resolved
  portability-cloudflare   cloudflare-kv      resolved

[4/5] Realize and execute those exact release bytes
  environment              realized provider  result                   status
  ------------------------ ------------------ ------------------------ --------
  portability-local        memory-kv          value:demo-...           executed
  portability-redis        redis-kv           value:demo-...           executed
  portability-cloudflare   cloudflare-kv      --cloudflare             plan only

[5/5] Verify application identity never changed
  release digest unchanged: yes (...)
  archive sha256 unchanged: yes (...)

PASS — one .presolved release, three plans, two provider realizations, zero application rebuilds.
```

With `--cloudflare`, the third row is also executed and the final line reports three provider realizations.

## Why these three environments matter

The providers are intentionally not three wrappers around the same implementation:

| Environment | Provider | Operational shape |
| --- | --- | --- |
| `portability-local` | `memory-kv` | process-local, no network hop, immediate visibility |
| `portability-redis` | `redis-kv` | external network service over Redis RESP/TCP, immediate sequential visibility |
| `portability-cloudflare` | `cloudflare-kv` | managed HTTP cloud service with eventual visibility |

The guest application still asks only for the same baseline `presolve:kv/store@0.1.0` capability. The environment and provider realization layers decide how that requirement is satisfied.

## Scope of the claim

This demo is evidence for Presolve's application/environment separation and provider substitution model. It is not a claim that every application is automatically portable to every environment.

Presolve still requires the environment to satisfy the application's declared workload, resource, capability, feature, and policy requirements. The resolver exists specifically to prove compatibility or explain why compatibility cannot be established.

The Redis adapter used here is intentionally proof-grade rather than production-complete: connection pooling, TLS, authentication, clustering, and failover are not part of this demo. The live Cloudflare adapter similarly demonstrates a concrete managed-provider realization, not a general production deployment lifecycle.

## Public verification

GitHub Actions runs this same credential-free entry point as the `Flagship portability demo` CI job. That keeps the externally documented demonstration and the repository's public proof path identical.

For the lower-level architecture and acceptance history behind this demo, see [`docs/development/chunk-21-portability-proof.md`](../../docs/development/chunk-21-portability-proof.md).
