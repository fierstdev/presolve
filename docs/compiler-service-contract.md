# Compiler service contract (L4)

L4 defines `presolve_compiler::service`, a local-only durable host over L3 v1.
It accepts complete request-owned candidate contents, validates them against
canonical L3 workspace products, and delegates compilation exclusively to the
L3 session API. Source contents are attempt-local and are never retained in
the durable session store.

The service protocol uses strict length-prefixed frames. Durable commits write
canonical L3 snapshot and graph documents into a temporary directory, atomically
rename it, then atomically select `current`. The L3 product cache remains
memory-only. TCP/HTTP, filesystem watching, source discovery, remote caching,
and private Rust serialization are outside this versioned boundary.
