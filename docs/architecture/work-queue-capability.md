# Work-queue capability semantics

Presolve models provider-neutral work distribution separately from provider transport and separately from workload triggering.

The initial canonical interface is:

```text
presolve:queue/work-queue@0.1.0
```

Its purpose is deliberately narrower than "portable messaging". It represents a competing-consumer work queue in which producers publish opaque payloads and consumers receive finite in-flight deliveries that must be settled.

## Why this is a distinct capability

Real queue services expose similar application intent through materially different mechanisms:

- Amazon SQS returns a receipt handle and hides a received message for a visibility timeout.
- Cloudflare Queues pull consumers return lease IDs and require explicit acknowledgement or retry before the visibility timeout expires.
- Redis Streams consumer groups track delivered-but-unacknowledged entries in a Pending Entries List and remove that pending state with `XACK`.

Those mechanisms are transport/provider details. The shared semantic shape is a leased delivery with explicit settlement.

## Baseline operations

The 0.1 contract exposes four mandatory operations:

```text
send
receive
ack
retry
```

Conceptually:

```text
send(body)
    -> accepted

receive()
    -> none | delivery

ack(delivery)
    -> settled

retry(delivery)
    -> eligible for redelivery
```

The eventual provider SDK may add options to these operations where negotiated features make those options meaningful, but the semantic operations themselves are fixed by the capability contract.

## Message body

The baseline message body is an opaque byte sequence.

Presolve does not assign JSON, text, schema, content-type, serialization, or event-envelope semantics to those bytes. Applications may define such conventions above the queue capability.

This avoids making one provider's message-attribute or structured-payload model canonical.

## Delivery model

### At least once

The baseline is at-least-once delivery while a successfully accepted message remains within the selected provider's configured availability and retention boundary.

Duplicates are valid behavior. The baseline therefore does not promise exactly-once processing.

Applications whose effects cannot tolerate duplicates must implement idempotency at the application/protocol level or later request a stronger negotiated semantic feature if Presolve defines one from provider evidence.

### Leased/in-flight delivery

`receive` creates a finite in-flight delivery.

The provider-native representation may be:

- an SQS receipt handle plus visibility timeout,
- a Cloudflare lease ID,
- Redis consumer-group pending state,
- or another provider mechanism.

That native identity is realization/runtime state, not application identity.

### Explicit acknowledgement

Receiving a message does not mark it successfully processed.

The consumer must call `ack` after its work has completed successfully.

### Unacknowledged redelivery

A delivery that is not acknowledged becomes eligible for redelivery after the provider's lease/in-flight window or equivalent recovery mechanism.

The exact timing is not part of the 0.1 baseline.

### Explicit retry

`retry` negatively settles the current attempt and preserves the logical message body for another delivery attempt.

A provider may use a different native message or receipt identity when implementing that retry. Presolve does not require native identifier preservation.

## Ordering

The baseline deliberately has no FIFO guarantee.

The mandatory guarantee is `provider-defined-delivery-order`: Presolve does not add ordering semantics to a provider that does not have them.

A provider may advertise the optional `ordered-receive` feature when it can preserve enqueue order while making new messages available to the configured logical queue.

`ordered-receive` does not imply:

- exactly-once delivery,
- completion ordering across concurrent consumers,
- global ordering across provider partitions/message groups,
- or transactional ordering with external side effects.

Those are distinct semantics.

## Optional features

The initial negotiated features are:

| Feature | Meaning |
| --- | --- |
| `configurable-lease-duration` | application receive options may request the in-flight lease duration |
| `dead-letter-routing` | the environment can route repeatedly failed messages to a configured dead-letter queue |
| `delayed-retry` | explicit retry may defer the next eligible delivery |
| `delayed-send` | a newly published message may be deferred before becoming eligible |
| `delivery-attempt-count` | a received delivery exposes a provider-maintained attempt count |
| `ordered-receive` | new messages can be made available in enqueue order for the configured logical queue |

These are semantic features, not promises that all provider configuration becomes part of the Application Contract.

For example, which physical queue acts as a DLQ is an environment/realization decision. An application may require that dead-letter routing exists without naming that physical queue in its immutable release.

## Work queue versus workload trigger

A critical boundary is intentionally outside this capability:

```text
queue storage/delivery semantics
        !=
workload invocation semantics
```

Cloudflare Queues can push batches into Worker invocations. AWS can connect SQS to event-source mappings. A long-running container may pull messages itself. Those are different ways to schedule or invoke workloads.

`presolve:queue/work-queue@0.1.0` standardizes the queue interaction model. It does not yet standardize:

```text
"invoke workload X whenever queue Y has messages"
```

A future workload-trigger/topology model should express that deployment relationship separately. This keeps scheduler/lifecycle behavior out of the capability provider API.

## Provider-fit matrix

The initial semantic target was derived against three materially different systems:

| Semantic | SQS standard | Cloudflare Queues pull | Redis Streams consumer group |
| --- | --- | --- | --- |
| opaque payload | yes | yes | adapter mapping required |
| at-least-once | yes | yes | yes with consumer-group recovery |
| leased/in-flight state | visibility timeout | visibility timeout + lease ID | pending entry + idle/reclaim state |
| explicit ack | delete by receipt | ack lease | `XACK` |
| unacked recovery | visibility expiry | visibility expiry | pending-entry reclaim |
| explicit retry | visibility release | retry lease | adapter/provider recovery strategy |
| baseline FIFO | no | no | stream order exists but processing semantics differ |
| delayed send | yes | yes | adapter semantics required |
| delayed retry | visibility/delay mechanisms | yes | adapter semantics required |
| DLQ | yes | yes | deployment pattern rather than native equivalent |

The matrix is intentionally about semantics, not API names.

## Why consumer groups are not a baseline feature

Redis Streams can expose multiple independent consumer groups over one stream. That starts to resemble durable pub/sub or log semantics rather than a single logical work queue.

SQS and Cloudflare Queues do not expose the same abstraction through their basic work-queue interfaces.

Presolve therefore does not make multi-group fan-out part of `work-queue@0.1.0`. If real applications require portable durable fan-out, that should be designed as separate messaging semantics rather than hidden behind one boolean feature.

## Why lease extension is not in 0.1

Some providers let consumers extend or change an in-flight visibility window. Others expose different lease models.

Presolve's current `CapabilityContract` IR has mandatory operations plus boolean optional features; it does not yet model operations that exist only when a feature is negotiated.

Instead of making `extend-lease` mandatory or pretending a boolean feature alone defines an unavailable operation, 0.1 leaves lease extension outside the application-facing contract. Environments can configure a suitable default lease window. Provider evidence can later justify an IR extension for feature-gated operations if needed.

## Numeric semantic properties remain unresolved

Queue providers differ in dimensions that are not well represented as booleans:

- maximum message size
- retention duration/range
- default and maximum lease duration
- maximum delay
- receive batch limits
- maximum retry policy values

SQL already raised the same general problem for limits and isolation levels.

The first queue providers should collect these differences explicitly. Presolve should add typed semantic properties only when those concrete provider requirements show what operators/applications actually need to negotiate.

## 0.1 non-goals

The baseline does not claim:

- exactly-once processing
- arbitrary event-stream/log portability
- global FIFO ordering
- multiple independent consumer groups
- portable provider retention configuration
- portable DLQ physical identity
- workload/event trigger portability
- transactionally coupling queue settlement to a database or external side effect
- provider-neutral observability/lag metrics

These may become later capabilities or semantic extensions if real workloads justify them.

## Implementation sequence

The intended validation sequence is:

1. canonical `work-queue@0.1.0` contract
2. provider SDK and conformance harness for send/receive/ack/retry
3. in-memory reference provider
4. one networked queue provider with materially different lease mechanics
5. one managed-provider proof
6. one immutable queue-using application release executed unchanged through those providers
7. use observed differences to decide whether typed semantic properties or feature-gated operations are necessary

The provider count is not the goal. The goal is to falsify or strengthen the semantic boundary.