# Capabilities and providers

Capabilities are the main boundary between application requirements and environment-specific implementations.

## Capability

A capability is a semantic service contract identified by a stable interface and semantic version.

Examples currently implemented by Presolve include:

```text
presolve:kv/store@0.1.0
presolve:objects/store@0.1.0
```

Applications depend on capability semantics, not provider products.

## Capability Contract

A canonical `CapabilityContract` defines:

```text
interface
version
operations
guarantees
optional features
```

Operations, guarantees, and optional feature identifiers are canonicalized independently of binding transport.

This means the semantic contract is not defined by a Rust trait, WIT file, HTTP API, or vendor SDK.

Those can be bindings of the contract.

See [Capability contracts](../architecture/capability-contracts.md).

## Capability requirement

The Application Contract can request a capability with a compatible version range:

```toml
[[capabilities]]
interface = "presolve:objects/store"
version = "^0.1"
required_features = ["range-read"]
preferred_features = ["user-metadata"]
```

A required feature is a hard compatibility requirement.

A preferred feature is used to rank otherwise-compatible providers.

## Provider

A provider is an environment-side implementation source.

The Environment Specification advertises exact versions and the optional semantic features the provider actually supports:

```toml
[[providers.capabilities]]
interface = "presolve:objects/store"
version = "0.1.0"
features = ["range-read", "user-metadata"]
```

The canonical contract defines the feature vocabulary. It does not imply that every provider implements every optional feature.

## Built-in vs custom capabilities

During Environment Specification enrichment:

```text
known Presolve interface + known exact version
    → canonical semantic contract attached

known Presolve interface + unknown version
    → rejected

unknown/custom interface + no feature claims
    → retained as a generic capability

unknown/custom interface + feature claims
    → rejected
```

Presolve does not claim to understand feature semantics for an unknown contract.

## Provider conformance

Presolve includes reusable provider conformance testing for implemented built-in capabilities.

The architectural goal is that adding another provider implementation does not change the application-facing semantic contract.

A provider that claims a Presolve capability is expected to uphold its mandatory guarantees.

## Provider identity

The resolver selects provider descriptors by stable `ProviderId`.

The node later uses that exact ID to retrieve a concrete implementation from its provider registry.

This preserves the separation between:

```text
resolution
    symbolic selection

materialization
    concrete implementation
```
