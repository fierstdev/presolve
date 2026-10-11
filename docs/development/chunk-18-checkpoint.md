# Chunk 18 checkpoint: real cloud provider portability

Chunk 18 proves that Presolve's capability/provider model can cross the boundary from local in-memory implementations to external managed infrastructure without changing application identity or embedding provider implementation details into the Application Contract.

## What is proven

The implemented path is:

```text
Application Contract
    requires presolve:kv/store ^0.1
            ↓
Environment Specification
    advertises provider capability supply
            ↓
Environment Inventory
            ↓
Resolver
            ↓
DeploymentPlan
    exact ProviderId
            ↓
ProviderRealizationCatalog
    ProviderId → adapter/settings/secret references
            ↓
ProviderAdapterRegistry
            ↓
CloudflareKvAdapter
            ↓
ProviderRegistry
    same ProviderId → concrete KeyValueProvider
            ↓
Cloudflare Workers KV
```

The live checkpoint has exercised this path against an actual Cloudflare Workers KV namespace.

The application does not know that Cloudflare was selected.

## Provider realization boundary

Chunk 18 introduces a separate node-side realization model rather than adding implementation configuration to resolver-facing environment supply.

```text
ProviderRealizationProfile
├── provider_id
├── adapter
├── non-secret settings
└── secret references
```

This is intentionally distinct from `presolve.env.toml`.

The `ProviderId` selected by resolution is the stable join between declarative environment supply and the concrete provider registered in the node.

Materialization is plan-scoped:

- only providers selected by the `DeploymentPlan` are instantiated
- secret references for unselected providers are not resolved
- adapters cannot silently substitute another `ProviderId`
- selected capability versions/features are revalidated at the adapter boundary

## Configuration-file policy

Chunk 18 deliberately does **not** add another required Presolve configuration file.

The normal developer-facing root file family remains:

```text
presolve.toml
presolve.env.toml
```

Provider credentials, SDK configuration, service endpoints, account identifiers, namespace identifiers, and similar realization data do not belong in either file merely because an adapter needs them.

The preferred direction for future realization configuration is:

```text
CLI-managed .presolve/ state
external secret/config sources
environment/operator integration
```

rather than one new root-level TOML file per subsystem or provider.

No persisted node realization format is standardized by Chunk 18.

## Secrets

Secret values do not live in provider realization profiles.

Profiles contain `SecretReference` values. A `SecretResolver` resolves material only when the selected provider is materialized.

The first resolver source is a process environment variable, but that is an implementation boundary rather than a permanent product decision. Future resolvers may target operating-system keychains, Vault, cloud secret managers, workload identity, or other systems.

`SecretValue` redacts its debug representation and requires explicit exposure by an adapter.

## First external adapter

`presolve-provider-kv-cloudflare` implements Presolve's baseline KV provider interface through Cloudflare Workers KV.

The adapter handles:

```text
get
set
delete
```

while preserving opaque exact keys and string KV values.

Its tests use a local HTTP server to validate request paths, bearer authentication, value writes, missing-value behavior, key validation, token redaction, and error categorization without requiring live credentials.

## Operational characteristics discovered

The first real provider exposed an important distinction:

```text
portable capability semantics
        ≠
identical provider operational characteristics
```

Workers KV has eventual visibility and provider-specific write/propagation constraints. Its delete API also does not directly return the prior-existence boolean required by Presolve's current KV trait, so the adapter implements the sequential semantic result with a read followed by delete.

That read/delete sequence is not atomic against concurrent external mutation.

Chunk 18 does not hide those differences and does not weaken Presolve's canonical KV contract to fit one provider.

It also does not yet add consistency, latency, quota, propagation, or throughput fields to the Application Contract or Environment Specification.

One external provider is not enough evidence to standardize that vocabulary.

A future extension should only add vendor-neutral operational requirements/supply when multiple real providers demonstrate a concrete compatibility need.

## Live proof

The live proof is intentionally opt-in:

```bash
./scripts/smoke-cloudflare-kv.sh
```

It reads:

```text
PRESOLVE_CLOUDFLARE_ACCOUNT_ID
PRESOLVE_CLOUDFLARE_KV_NAMESPACE_ID
PRESOLVE_CLOUDFLARE_API_TOKEN
```

from the process environment.

It generates a unique key, writes through the provider selected by Presolve, polls for bounded eventual visibility, verifies the exact value, and deletes the proof key.

Credentials are not committed and the script does not print the API token.

## Reproducible non-live checkpoint

The normal Chunk 18 regression checkpoint is:

```bash
./scripts/check-cloud-provider.sh
```

It verifies provider realization, the Cloudflare adapter, plan-scoped node materialization, examples, and Clippy without requiring credentials or network access.

The live smoke is separate because it depends on external infrastructure and credentials.

## Explicit non-claims

Chunk 18 does not establish:

- a stable public cloud-provider plugin ABI
- production support guarantees for Cloudflare Workers KV
- a persisted node realization configuration schema
- a general consistency/latency/quota compatibility model
- automatic cloud resource provisioning
- a deployment control plane
- support for every provider that can approximately implement KV semantics

Those remain later product work.

## Result

Chunk 18 validates the central provider portability thesis:

> Resolver-visible capability supply can remain vendor-neutral while node-side materialization selects and configures a concrete external provider under the exact `ProviderId` chosen by the deployment plan.

The application remains unchanged.
