# Object-storage portability proof

Presolve's object-storage proof extends portability validation beyond key-value storage.

The claim under test is:

> One immutable application release can request provider-neutral object storage, resolve against materially different object-storage implementations, and execute unchanged through those implementations while provider configuration remains outside application identity.

The proof entry point is:

```bash
bash ./scripts/check-object-portability.sh
```

## Proof shape

The application is [`examples/objects-demo`](../../examples/objects-demo). It requests only the baseline `presolve:objects/store@0.1.0` capability.

The guest exercises the baseline semantics end to end:

```text
put
 ↓
head
 ↓
list(prefix)
 ↓
streamed get
 ↓
delete
 ↓
post-delete head
```

The proof builds one detached `objects-demo.presolved` release exactly once and records both its canonical Presolve release digest and the SHA-256 of the encoded archive bytes.

Those exact bytes are then resolved against two checked-in environments:

| Environment | Provider | Shape |
| --- | --- | --- |
| `objects-local` | `memory-objects` | in-process memory provider |
| `objects-s3` | `s3-objects` | external S3-compatible HTTP service |

Both environments advertise the same baseline object capability. The application release names neither implementation.

## S3-compatible realization

The S3 adapter is intentionally provider-generic rather than MinIO-specific. It uses path-style S3 HTTP operations and AWS Signature Version 4 authentication.

The following values are realization-time inputs only:

- endpoint
- bucket
- region
- access-key identifier
- secret access key

They are not stored in the Application Contract, Environment Specification, DeploymentPlan, or `.presolved` release.

Public CI realizes the adapter against a pinned, ephemeral S3-compatible service. That container is test infrastructure; it is not part of the Presolve application model and is not a production-provider recommendation.

## Baseline operations

The adapter currently maps the Presolve baseline to S3-compatible operations as follows:

| Presolve operation | S3-compatible realization |
| --- | --- |
| `put` | signed `PUT` object |
| `get` | signed `GET` object, streamed to the runtime |
| `head` | signed `HEAD` object |
| `delete` | `HEAD` followed by signed `DELETE` when present |
| `list(prefix)` | paginated `ListObjectsV2` with URL-encoded keys |

The adapter rejects negotiated object features it does not explicitly implement.

## Semantic differences discovered by the proof

The purpose of adding real providers is to find abstraction boundaries, not hide them.

### Delete existence is synthesized

S3 `DELETE` is idempotent and does not report whether an object existed. Presolve's current baseline `delete` operation returns an existence boolean.

The adapter therefore performs `HEAD` before `DELETE` to preserve the sequential baseline result. That pair is not atomic against concurrent external mutation. Presolve must not represent it as stronger semantics than the provider supplies.

This is a candidate for future semantic negotiation if workloads require stronger delete guarantees.

### Object-key length is provider-constrained

S3-compatible object keys are limited to 1,024 UTF-8 bytes. Presolve's baseline object contract currently validates non-empty keys but does not expose maximum key length as a negotiable capability property.

The S3 adapter enforces the provider constraint locally. The mismatch is evidence that provider limits such as key length may need to become explicit capability semantics if real applications need portability across providers with different limits.

### Initial put implementation buffers before mutation

The provider SDK permits streaming upload bodies. The first S3-compatible adapter buffers the declared body before issuing `PUT` so it can validate the exact declared length before mutating external state and compute the SigV4 payload hash.

This preserves correctness for the current proof but is not the final large-object upload strategy. Multipart/streaming upload behavior should be added only with explicit semantics around limits, retries, and integrity.

## What this proof is intended to establish

A successful run establishes that:

- object-storage demand survives inside one immutable detached release;
- two environments can select different stable ProviderIds for that same demand;
- provider endpoint, bucket, region, and credentials remain realization concerns;
- the same release bytes execute unchanged through in-process and networked object stores;
- baseline `put`, metadata, prefix listing, streamed reads, and delete behavior survive provider substitution; and
- release identity remains unchanged after both executions.

It does not establish that every S3 feature is portable, that all object providers have identical operational characteristics, or that the current S3 adapter is production-ready.

## Why this matters

The KV flagship proves one capability family. Object storage introduces different failure modes and semantics: streamed payloads, metadata, prefix listing, key constraints, external buckets, SigV4 authentication, and idempotent delete behavior.

Those differences make object storage a stronger test of Presolve's central boundary:

```text
application requirement
        +
environment supply
        ↓
semantic resolution
        ↓
provider realization
```

The next deployment-decision coverage targets after this proof are relational SQL semantics and asynchronous queues.