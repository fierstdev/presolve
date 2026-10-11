# TypeScript authoring

Presolve's TypeScript authoring layer is a language-facing projection of the existing application and capability model. It is not a new application model.

## Source of truth

The canonical application declaration remains:

```text
presolve.toml
```

TypeScript configuration does not replace or duplicate the Application Contract.

This keeps application identity and environment resolution language-neutral.

## SDK boundary

TypeScript application code will use `@presolve/sdk`:

```text
TypeScript source
        ↓
@presolve/sdk
        ↓
typed capability client
        ↓
CapabilityTransport
        ↓
execution-target adapter
```

The SDK kernel defines `CapabilityCall` and `CapabilityTransport`.

A capability call contains only:

```text
interface
operation
input
```

It does not contain:

```text
provider identity
provider credentials
Docker configuration
HTTP endpoints
registry references
WIT implementation details
environment selection
```

Those remain outside application authoring.

## Runtime installation

The low-level `@presolve/sdk/runtime` entry point allows an execution adapter to install one `CapabilityTransport`.

Application code normally does not install transports itself.

This supports multiple future projections without changing capability-facing application code:

```text
@presolve/sdk
      │
      ├── container bridge adapter
      ├── Component/WIT adapter
      └── future compatible transports
```

Chunk 17.1 establishes this boundary. It deliberately does not yet expose KV/object convenience APIs or bind the SDK to the existing HTTP/JSON bridge.

## Package tooling

The repository's existing pnpm workspace hosts TypeScript packages and examples alongside the Cargo workspace.

The Rust workspace remains authoritative for Rust crates and execution internals; pnpm exists only for language-facing packages and examples.
