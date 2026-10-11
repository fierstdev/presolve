# Provider realization

Presolve separates resolver-visible provider supply from node-side provider implementation configuration.

```text
Environment Specification
    ProviderId + capability supply
            ↓
DeploymentPlan
    exact selected ProviderId
            ↓
node realization
    ProviderId → installed adapter + configuration + secrets
            ↓
ProviderRegistry
    ProviderId → concrete provider trait implementation
```

The same `ProviderId` joins all three layers.

`presolve.env.toml` continues to describe only what the environment can supply. It does not gain adapter identifiers, endpoints, account IDs, namespace IDs, access keys, or API tokens.

Chunk 18.1 introduces an in-memory node-side model:

```text
ProviderRealizationProfile
├── provider_id
├── adapter
├── non-secret settings
└── secret references
```

Secret references contain no secret value. `SecretResolver` resolves them only when a provider is materialized. `SecretValue` always renders as `SecretValue([redacted])` in debug output and requires an explicit `expose_secret()` call by an adapter.

The initial secret reference source is a process environment variable. This defines a boundary; it does not make process environment variables the eventual production secret system.

No canonical persisted node-configuration file format is introduced in 18.1. A file format should not be locked before multiple real adapters have exercised the realization model.

The next step is the first real provider implementation: Cloudflare Workers KV.

## Cloudflare Workers KV adapter

Chunk 18.2 adds `presolve-provider-kv-cloudflare`, the first provider adapter backed by external managed infrastructure.

The adapter implements the existing `KeyValueProvider` trait through Cloudflare's Workers KV REST API:

```text
Presolve get     → GET    /accounts/{account}/storage/kv/namespaces/{namespace}/values/{key}
Presolve set     → PUT    /accounts/{account}/storage/kv/namespaces/{namespace}/values/{key}
Presolve delete  → GET + DELETE
```

The adapter uses a bearer API token supplied as `SecretValue`. Debug output never includes the token.

Account ID, namespace ID, and API token remain realization data. They do not enter the Application Contract, Environment Specification, DeploymentPlan, or `.presolved` release.

### Semantic caveats exposed by the first cloud adapter

Cloudflare Workers KV is intentionally a useful stress test rather than a perfect mirror of Presolve's in-memory provider.

Cloudflare's delete API is idempotent and does not report whether a value previously existed. Presolve's current baseline KV trait does. The adapter therefore performs a read before delete so ordinary sequential calls preserve Presolve's boolean result.

That read/delete sequence is not atomic with respect to concurrent external mutation. The adapter does not claim otherwise.

Cloudflare Workers KV is also eventually consistent and applies operational limits such as a per-key write rate. Presolve's current in-process conformance harness assumes immediate visibility and performs rapid repeated writes to the same key, so Chunk 18.2 does not label the Cloudflare adapter as having passed that harness.

This distinction is intentional:

```text
semantic API compatibility
    !=
identical consistency/operational characteristics
```

A later Chunk 18 step must decide how environment/provider supply advertises characteristics that materially affect compatibility without putting vendor implementation details into application semantics.

### Test boundary

Chunk 18.2 unit tests use a local HTTP server to verify:

- exact Cloudflare REST paths
- percent-encoded opaque keys
- bearer authentication
- raw complete-value writes
- 404-to-missing behavior
- sequential delete existence reporting
- token redaction
- invalid-key validation
- availability/error categorization

No live Cloudflare credentials are required until the live portability proof.

## Adapter registry and plan-scoped materialization

Chunk 18.3 connects symbolic resolver output to installed provider adapters without moving implementation configuration into the Environment Specification.

```text
DeploymentPlan
    exact ProviderId
        ↓
ProviderRealizationCatalog
    ProviderId → realization profile
        ↓
ProviderAdapterRegistry
    adapter ID → installed adapter
        ↓
ProviderAdapter::realize
        ↓
RealizedProvider
        ↓
ProviderRegistry
    exact ProviderId → concrete capability implementation
```

`ProviderCapabilityRequest` carries only the exact capability interface, selected provider version, and negotiated semantic features required by the plan. The realization crate therefore remains independent of the resolver crate.

Adapters validate selected semantics before constructing a provider. The Cloudflare KV adapter accepts only `presolve:kv/store@0.1.0` with no negotiated optional features.

Materialization is strictly plan-scoped. Profiles not selected by a `DeploymentPlan` are not instantiated and their secrets are not resolved.

No new user-authored configuration file is introduced by 18.3. The profile/catalog objects remain node-side runtime state while the shape is exercised by real providers.

## Live Cloudflare KV portability proof

Chunk 18.4 adds an opt-in live checkpoint against a real Cloudflare Workers KV namespace.

The checkpoint does not introduce another Presolve configuration file. Live realization data is supplied only for the proof process:

```text
PRESOLVE_CLOUDFLARE_ACCOUNT_ID
PRESOLVE_CLOUDFLARE_KV_NAMESPACE_ID
PRESOLVE_CLOUDFLARE_API_TOKEN
```

The API token is represented by a `SecretReference` and resolved by `EnvironmentSecretResolver` only after the resolver selects the provider.

The proof exercises:

```text
generic Application Contract
    requires presolve:kv/store ^0.1
            ↓
EnvironmentSpecification
    advertises one provider at 0.1.0
            ↓
inventory enrichment
            ↓
resolver
            ↓
DeploymentPlan
    exact ProviderId
            ↓
ProviderRealizationCatalog
            ↓
ProviderAdapterRegistry
            ↓
CloudflareKvAdapter
            ↓
ProviderRegistry
    same ProviderId
            ↓
real Cloudflare Workers KV
```

A unique key is generated for each run. The checkpoint performs one write, polls until that value becomes visible within a bounded window, and then deletes the key.

The polling is deliberate. Workers KV is eventually consistent, so the proof validates successful portability without pretending that all conforming providers have identical visibility characteristics.

Run the live proof with:

```bash
./scripts/smoke-cloudflare-kv.sh
```

The script never prints the API token.
