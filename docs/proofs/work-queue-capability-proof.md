# Leased work-queue capability proof

Presolve's queue proof establishes the current `presolve:queue/work-queue@0.1.0` semantic and runtime boundary with one concrete provider.

It is intentionally **not** labeled cross-provider queue portability yet. The next proof milestone is to add a materially different network or managed queue implementation and execute the same immutable release through both.

The current proof entry point is:

```bash
bash ./scripts/check-queue-portability.sh
```

## Contract under test

The baseline operations are:

```text
send
receive
ack
retry
```

Mandatory semantics are:

- opaque byte message bodies
- at-least-once delivery
- finite leased/in-flight delivery
- explicit acknowledgement
- unacknowledged delivery becomes eligible for redelivery
- explicit retry preserves the logical message body
- baseline delivery order is provider-defined rather than FIFO

Optional features include:

```text
delayed-send
delayed-retry
delivery-attempt-count
dead-letter-routing
configurable-lease-duration
ordered-receive
```

Workload triggering is deliberately outside this capability. A queue provider describes message-delivery semantics; mapping a queue to Worker/Lambda/container invocation topology is a separate deployment concern.

## Ownership model

The provider SDK returns an owned delivery object:

```text
WorkQueueProvider
    send(...)
    receive(...) -> WorkQueueDelivery

WorkQueueDelivery
    body()
    delivery_attempt()
    ack(self)
    retry(self)
```

`ack` and `retry` consume the delivery, which makes double settlement impossible through the type boundary.

Dropping an unsettled delivery does **not** acknowledge it. Provider lease recovery is therefore allowed to make the logical message visible again.

The WIT projection uses the same ownership model through an opaque delivery resource. Provider-native message IDs, receipt handles, pending-entry IDs, and lease tokens are not application semantics and remain hidden.

## Reference provider

The current concrete provider is the process-local in-memory leased queue.

It models:

- active lease hiding
- abandoned-delivery redelivery
- explicit retry
- delivery-attempt counting
- stale delivery-generation rejection
- opaque byte preservation

Each receive increments an internal generation. A stale delivery object therefore cannot acknowledge or retry a newer redelivery of the same logical message.

This is the reference implementation for the semantic model, not a durable production queue.

## Reusable conformance harness

The queue conformance suite deliberately tests only baseline promises.

It verifies:

- opaque binary and empty bodies survive send/receive unchanged
- deliveries can be explicitly acknowledged
- dropping an unacknowledged delivery permits observable redelivery
- explicit retry preserves the logical body

It does **not** assert FIFO ordering or exactly-once behavior. At-least-once providers may legally produce duplicate deliveries.

This is important for future providers such as SQS, Cloudflare Queues, or Redis Streams: the conformance suite must not accidentally turn one reference provider's operational behavior into stronger portable semantics.

## Detached-release proof

The application is `examples/queue-demo`.

The proof builds one immutable queue release exactly once, records both its canonical Presolve release digest and encoded archive SHA-256, resolves it against the checked-in queue environment, realizes the selected provider, executes the Component through the queue WIT/runtime path, and then verifies both identities again.

The successful public proof recorded:

```text
release digest: sha256:7f848c878310c14995b7963e8e2f27ddd5173e1379de13e6627e5b532a235511
archive sha256: 4a20e2f0e353ea53371c3bd19bb1ddbf105bb665deae31e788a02db7d767dd9c
```

Resolution selected:

```text
environment: queue-memory
provider: memory-queue
```

The guest-visible result was:

```text
queue:queue-2142
```

The checkpoint then verified:

```text
release digest unchanged: yes
archive sha256 unchanged: yes
```

## Guest sequence

The demo exercises settlement semantics through the Component binding rather than calling the provider directly:

```text
send
  ↓
receive leased delivery
  ↓
retry
  ↓
receive again
  ↓
ack
  ↓
no settled message remains visible
```

The provider-native lease/receipt identity never enters the application release.

## What this proves

The current queue checkpoint establishes that:

- the work-queue capability contract is represented in an immutable detached release
- resolution can select a stable queue `ProviderId`
- node materialization supplies the corresponding `WorkQueueProvider`
- delivery ownership survives the WIT Component boundary
- retry and acknowledgement settlement semantics execute through the real runtime path
- provider-native receipt/lease identity remains opaque to the application
- the exact `.presolved` archive and logical release identity remain unchanged after execution

## What this does not prove

This checkpoint does not yet establish:

- cross-provider queue substitution
- SQS, Cloudflare Queues, Redis Streams, Kafka, or another external queue implementation
- FIFO ordering
- exactly-once delivery or processing
- portable dead-letter behavior
- portable retention, message-size, delay, or lease-duration limits
- workload-trigger topology such as Lambda event-source mappings or Cloudflare Worker queue triggers
- production durability or high availability

These are intentional non-claims.

## Next portability step

The next useful queue proof is not another in-memory variant. It is a materially different queue implementation whose native model stresses the baseline.

Candidates include:

```text
Redis Streams consumer groups
SQS receipt handles + visibility timeouts
Cloudflare Queues pull consumers / acknowledgement semantics
```

The target proof shape is:

```text
one queue-demo.presolved
        │
        ├── environment A -> provider A -> execute
        │
        └── environment B -> provider B -> execute

release digest unchanged
archive SHA-256 unchanged
```

That is the point at which Presolve should call queue **cross-provider portability** proven.

## Related documentation

- [Work-queue capability semantics](../architecture/work-queue-capability.md)
- [Capability contracts](../architecture/capability-contracts.md)
- [Provider realization](../architecture/provider-realization.md)
- [Architecture overview](../architecture/overview.md)
