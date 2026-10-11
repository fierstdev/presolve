# Application portability

Presolve's core thesis is:

> Build software once. Decide where it runs later.

This does not mean every execution environment is identical. It means an application's identity and semantics should not be defined by one particular realization environment.

## The boundary

A Presolve application describes:

- its identity and release version
- its workloads
- application-internal interfaces and relationships
- capabilities it requires
- minimum resources
- network requirements

It does not describe:

- AWS, Cloudflare, Kubernetes, or another vendor
- a specific database product
- provider credentials
- infrastructure resource names
- deployment placement
- concrete provider implementations

Those choices belong on the environment and deployment side.

## Requirements, not vendors

An application can request:

```text
presolve:kv/store ^0.1
presolve:objects/store ^0.1
```

instead of embedding:

```text
one vendor's KV SDK
one vendor's object-store SDK
```

The environment advertises compatible providers. The resolver selects a compatible provider and records that decision in a `DeploymentPlan`.

The application release remains unchanged.

## Portability is semantic

Presolve does not define portability as "all providers have similarly named methods."

A capability has a semantic contract:

```text
identity
version
operations
mandatory guarantees
optional features
```

A provider is compatible only when it can satisfy the required semantics.

This is why provider conformance and semantic capability contracts are central to the architecture.

## Portability does not require one universal runtime

WebAssembly Components are the first implemented workload class, but Presolve's outer application model is not intended to be synonymous with WebAssembly.

The model separates:

```text
application semantics
        from
workload execution substrate
        from
deployment environment
```

Future workload kinds can be added without changing the rule that application identity remains independent of a particular deployment.

Only the `component` workload kind is implemented today.

## Avoiding an M × N adapter model

Without an abstraction boundary, each application tends to accumulate deployment-specific integrations:

```text
application A × provider X
application A × provider Y
application B × provider X
application B × provider Y
...
```

Presolve instead creates two contracts:

```text
application → capability semantics
environment → capability supply
```

The resolver joins them.

That does not eliminate provider integrations. It moves them to a reusable provider boundary rather than repeating them throughout application code.

## What Presolve does not promise

Presolve does not currently promise that:

- every application can run in every environment
- every capability has an implementation everywhere
- all workload substrates are interchangeable
- resource capacity is a full multi-tenant scheduler
- provider configuration is portable by definition

Instead, incompatibility is expected to be explicit and diagnosable.

See [Resolution and deployment](resolution-and-deployment.md).
