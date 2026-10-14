# Flagship portability walkthrough

Presolve's flagship demonstration tests one narrow architectural claim:

> Build one immutable application release once, then decide where it runs later without rebuilding or mutating the application.

The public entry point is:

```bash
bash ./demo/portability/run.sh
```

The same command is executed by public CI.

## What the demo contains

The application is `examples/kv-demo`.

Its application requirement is provider-neutral: it needs the Presolve KV capability contract. The application does not name Redis, Cloudflare, a network endpoint, credentials, or a local in-memory implementation.

The proof uses three checked-in environments:

| Environment | Selected provider | Provider shape |
| --- | --- | --- |
| `portability-local` | `memory-kv` | in-process |
| `portability-redis` | `redis-kv` | networked RESP/TCP |
| `portability-cloudflare` | `cloudflare-kv` | managed cloud service |

## Stage 1: build Presolve tooling

The script first builds the Presolve CLI and the release-realization checkpoint used by the demonstration.

This tooling build is not an application rebuild. It prepares the software that will build, inspect, resolve, and execute the demo release.

## Stage 2: build the application exactly once

Presolve builds `kv-demo` into one detached `.presolved` release.

The script immediately records two identities:

1. the canonical Presolve release digest; and
2. the SHA-256 of the encoded `.presolved` archive bytes.

The first identifies the logical Presolve release. The second proves that the exact file passed through the rest of the demonstration did not change.

No environment is consulted while constructing the application release.

## Stage 3: resolve the same release against three environments

The script calls `presolve plan` against each environment using the same detached release.

The expected provider selections are:

```text
portability-local       -> memory-kv
portability-redis       -> redis-kv
portability-cloudflare  -> cloudflare-kv
```

This stage demonstrates the separation between application demand and environment supply.

The release says what it requires. Each environment says what it can provide. The resolver determines whether the requirement can be satisfied and emits an inspectable plan containing the selected stable ProviderId.

No provider is instantiated by the resolver.

## Stage 4: realize providers and execute the exact release bytes

The local path realizes the in-memory KV implementation.

The Redis path starts a pinned Redis container and realizes the Redis adapter over RESP/TCP.

Both executions receive the same `.presolved` archive produced in Stage 2. No component sidecar is extracted and passed as the application artifact; the runtime checkpoint reads and verifies the Component from the detached release itself.

The resulting application behavior is equivalent even though the provider implementation and operational topology are materially different.

The Cloudflare path is plan-only in credential-free public CI. Running:

```bash
bash ./demo/portability/run.sh --cloudflare
```

with the documented realization-time Cloudflare credentials promotes that third row to live Workers KV execution without rebuilding the application.

Cloudflare account identity, namespace identity, API credentials, and provider realization settings remain outside the `.presolved` release.

## Stage 5: verify identity did not change

After the environment-specific work has completed, the script recomputes both identities captured in Stage 2.

The proof fails unless both are unchanged:

```text
release digest unchanged: yes
archive sha256 unchanged: yes
```

A successful run ends with:

```text
PASS — one .presolved release, three plans,
two provider realizations, zero application rebuilds.
```

## Why Redis matters

Memory and Cloudflare alone can look like a special-case local/cloud split.

Redis adds a materially different third provider shape:

- it is an external process;
- it communicates over a network socket;
- it uses Redis protocol semantics rather than in-process calls or Cloudflare's HTTP API; and
- its address is supplied only when the plan is realized.

The application still requests the same Presolve KV capability contract.

That makes the demo evidence about Presolve's abstraction boundary rather than about one particular provider pair.

## What this proves

The demonstration currently proves that:

- one detached release can reconstruct the application requirements needed for resolution;
- application release identity is independent of environment selection;
- one provider-neutral KV requirement can resolve to different stable ProviderIds;
- the exact release bytes can execute through both an in-process and a networked provider implementation;
- managed-cloud realization can remain external to application identity; and
- the resolver's output is inspectable before realization.

## What this does not prove

The demonstration does **not** claim that:

- Presolve is production-ready;
- all current APIs or schemas are stable;
- every workload can run in every environment;
- all providers have identical behavior beyond the semantics represented by their capability contracts;
- Presolve automatically provisions arbitrary infrastructure;
- Presolve is a scheduler, service mesh, registry, hosted PaaS, or full deployment lifecycle manager.

Those distinctions are deliberate. The value of the flagship proof is that it isolates the core portability invariant and makes it falsifiable.

## How to evaluate the proof critically

The most useful questions are not whether the command prints `PASS`. They are:

1. Does any environment-specific assumption leak into the application release?
2. Are the semantics in the capability contract strong enough to make provider substitution meaningful?
3. Does the DeploymentPlan expose enough information to understand exactly what the resolver selected?
4. Can another provider be implemented without changing application code or release identity?
5. Which real workloads or infrastructure constraints would break this model?

Those are the questions Presolve needs external engineers to test next.

## Source map

- Demo entry point: [`demo/portability/run.sh`](../../demo/portability/run.sh)
- Demo documentation: [`demo/portability/README.md`](../../demo/portability/README.md)
- Application example: [`examples/kv-demo`](../../examples/kv-demo)
- Environment fixtures: [`examples/portability-proof`](../../examples/portability-proof)
- Provider realization architecture: [`docs/architecture/provider-realization.md`](../architecture/provider-realization.md)
- Inspection and planning boundary: [`docs/architecture/inspection-and-planning.md`](../architecture/inspection-and-planning.md)
- Application portability concept: [`docs/concepts/application-portability.md`](../concepts/application-portability.md)
