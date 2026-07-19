# Phase K Production Optimization Baseline

K0 freezes a pre-optimization measurement and inventory boundary. It adds no production artifact, report, optimizer, typed optimization identity, chunk topology change, schema change, or runtime behavior.

## Frozen Phase J entry

The committed J21 versions are semantic graph v6, template manifest v4, component artifact v3, Context artifact v2, Forms and Effect artifacts v1, resume manifest v6, resume snapshot v1, resume protocol/registry v1, ASM inspection v11, and check JSON v6.

`fixtures/phase-k-production-baseline.json` is the canonical test-only K0 byte and product-count fixture. It measures the action-counter and component-structural representatives twice, including every emitted artifact. The current common runtime module is 143,282 bytes; the current J eager module is root-owned and carries `runtime-bootstrap`, `runtime-registries`, and `event-delegation` only.

Phase J's `ResumeChunkGraph` deliberately duplicates exact root program closures and forbids shared lazy chunks and lazy-to-lazy dependencies. The K0 representatives have one eager application root, no dependency chunk, and no shared program delivery. K6/K7 may change that only under the fixed Phase K policy and canonical equivalence evidence.

## Runtime inventory

The generated runtime uses compiler-emitted string IDs in `Map` and `Set` indexes for template anchors, component/slot/structural records, resume slot values and definitions, Forms indexes, Context bindings, Effect state, and activation registrations. It uses delegated document listeners, explicit scheduler queues, and compiler-owned restore registries. Existing destruction and release paths remain frozen until K12/K13; K0 neither adds cleanup nor alters listener/queue registration.

The K0 policy constants remain compiler-owned. K18 activates the canonical
ordered public projection for `EZC1112` through `EZC1127`; it emits only from
immutable production failure facts and never invents missing identities or
provenance. `EZASM1385` through `EZASM1512` remain the reserved internal
integrity range.
