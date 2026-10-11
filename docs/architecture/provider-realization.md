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
