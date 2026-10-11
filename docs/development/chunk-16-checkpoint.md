# Chunk 16 checkpoint: OCI/container execution

Chunk 16 exists to test whether Presolve's application/environment/deployment model survives a second execution substrate that is not WebAssembly-centric.

The checkpoint is intentionally end to end.

## Proven path

The local proof executes this chain:

```text
Application Contract
    container workload
    presolve:kv/store requirement
        ↓
Environment Inventory
    container execution support
    concrete KV provider supply
        ↓
Resolver
        ↓
DeploymentPlan
    workload = api/container
    KV binding = exact ProviderId
        +
.presolved 0.2
    immutable OCI descriptor
        ↓
Node
    exact ProviderId realization
    authenticated capability gateway
        ↓
Container materialization
    OCI descriptor → environment-local image source
        ↓
ContainerExecutor
        ↓
Docker
        ↓
real OCI workload
        ↓
presolve-http-json/0.1
        ↓
selected KeyValueProvider
```

The workload writes:

```text
presolve.chunk16.checkpoint = container-capability-ok
```

from inside the container. The host harness then reads that same value directly from the exact selected provider implementation.

This proves the capability call crossed the real container boundary rather than being simulated inside the host test process.

## Immutable OCI identity

The smoke workflow creates a temporary local OCI registry and uses BuildKit's registry exporter with OCI media types enabled.

Build metadata supplies the resulting OCI descriptor:

```text
mediaType
digest
size
```

That descriptor is used to construct the `.presolved` release. The registry hostname, repository name, and tag are not part of the release.

The exact digest is then pulled into the local Docker image store so the Docker executor can resolve an environment-local source for the same immutable artifact.

## What Chunk 16 proves

Chunk 16 now demonstrates:

- `container` is a real workload class rather than reserved schema vocabulary
- workload compatibility is resolved through the same Environment model as Components
- OCI artifact identity remains environment-independent
- `.presolved` can describe Component and OCI workload artifacts without storing deployment location
- workload execution is behind a backend-neutral executor boundary
- Docker is an adapter rather than the canonical runtime model
- capability semantics remain above transport
- KV and object capabilities can be projected across a container boundary
- exact resolved `ProviderId` identity survives into provider invocation
- provider credentials are not required inside the application container
- unsupported network/capability combinations fail instead of silently weakening semantics

## What Chunk 16 does not claim

The checkpoint does not make these general product guarantees:

- `presolve dev` does not yet provide the final container developer workflow
- arbitrary container networking policy is not fully realizable by the Docker CLI adapter
- the bridge v0.1 object-body representation is not the final high-throughput transport
- container health, restart, logs, service discovery, ingress, and lifecycle management are not yet a complete deployment control plane
- remote registry discovery and credentials are not yet modeled as general environment realization
- cloud container environments are not yet proven

Those belong to later product/build/deployment work.

## Running the checkpoint

With Docker running:

```bash
./scripts/smoke-container-oci.sh
```

A successful run ends with:

```text
observed provider write: presolve.chunk16.checkpoint=container-capability-ok
Chunk 16 OCI/container checkpoint passed
== Chunk 16 checkpoint passed ==
```
