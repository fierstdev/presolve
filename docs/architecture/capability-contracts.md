# Capability contracts

A `CapabilityContract` is Presolve's canonical semantic definition of a capability.

It exists above any one programming-language or transport binding.

## Contract model

A capability contract records:

```text
interface
version
operations
guarantees
optional_features
```

For example, an object-storage capability can define a stable operation set and semantic guarantees independently of whether a provider is reached through WIT, a native Rust trait, HTTP, IPC, or another mechanism.

## Why semantics are separate from bindings

A binding answers:

> How is this capability invoked here?

A capability contract answers:

> What does this capability mean?

Those are different questions.

Presolve therefore avoids making WIT, Rust traits, or another transport schema the canonical architecture.

## Canonicalization

Capability contracts canonicalize operation, guarantee, and optional-feature identifiers into deterministic lexical order.

Identifiers use lowercase kebab-case.

A contract must define at least one operation and rejects duplicate semantic members.

Canonical JSON bytes can therefore represent the same semantic contract deterministically.

## Operations

Operations name the actions exposed by every conforming implementation.

Examples:

```text
get
put
delete
list
```

The operation list is part of semantic versioning.

## Guarantees

Guarantees are mandatory semantics every conforming provider must uphold.

Examples from current built-in contracts include concepts such as:

```text
exact-key-identity
read-after-write
put-replaces-object
delete-idempotent
```

A provider cannot claim conformance merely because it exposes similarly named operations.

## Optional features

Optional features define a known semantic vocabulary that individual providers may advertise.

For object storage, examples include:

```text
range-read
conditional-write
user-metadata
```

The contract defines that those features exist. Each provider independently declares which ones it supports.

## Negotiation

The Application Contract divides feature requests into:

```text
required_features
preferred_features
```

Environment providers advertise:

```text
supported features
```

Resolution uses required features as compatibility constraints and preferred features as ranking signals.

## Binding targets

The long-term architecture allows the same semantic contract to drive or validate multiple binding forms:

```text
Capability Contract
      │
      ├── Rust/provider SDK binding
      ├── WIT/WebAssembly binding
      ├── future TypeScript binding
      ├── resolver metadata
      ├── provider conformance
      └── documentation
```

Not every generation path is implemented yet, but the semantic contract is already separate from its current bindings.

## Container capability bridge

OCI containers cannot receive Wasmtime host imports directly, so Presolve now has a second binding target for baseline built-in capabilities:

```text
Capability Contract
      │
      ├── Rust provider trait
      ├── WIT / Component Model
      └── HTTP/JSON capability bridge v0.1
```

The bridge is a transport projection, not a new semantic contract.

The initial protocol uses an authenticated `POST /v1/call` envelope carrying `interface`, `operation`, and operation-specific JSON input. Responses carry either operation-specific JSON output or a stable error category/message.

The current bridge implements baseline `presolve:kv/store@0.1.0` and `presolve:objects/store@0.1.0`. Negotiated optional object-storage features are rejected until their bridge projection is defined.

Object bodies are represented as byte arrays in bridge v0.1. This first projection prioritizes semantic correctness and language-neutrality; a later protocol version may add streaming without changing the underlying object-storage capability contract.

A per-deployment bearer token authorizes access to the node-hosted bridge. Provider credentials are never injected into the application container.
