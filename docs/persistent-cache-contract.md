# Persistent complete-result cache (L6)

L6 is a non-authoritative local optimization. It stores only complete,
successful, canonical L3 snapshot/graph results after L4/L5 publication. Clean
L3 compilation and L5 session-local planning remain authoritative.

The explicit cache root is supplied when the compiler service starts. It owns
only `manifest.json` and `entries/<prefix>/<key>/{entry.json,payload.bin}`.
Missing, invalid, corrupt, incompatible, locked, or unavailable cache roots
are cache misses and compilation continues through L5. There is no project
filesystem discovery, remote cache, public cache CLI, or multi-writer support.

Keys are SHA-256 over length-delimited compiler/package, service protocol,
L3/L5 schema, feature/platform, configuration, complete source-universe,
compile-mode, artifact, diagnostic, and payload-codec identities. Entries are
atomically published only after successful service publication. The payload is
restricted to canonical source-free snapshot/graph/response metadata: it never
contains authored source, parser products, ASTs, request frames, or L5
baselines.

The internal service APIs `inspect_cache`, `verify_cache`, and `clean_cache`
operate only on the explicitly configured owned root. `clean_cache` refuses an
unowned root. Cache telemetry is optional (`none`, `summary`, `full`) and is
additive; with `none`, cache use does not alter compiler responses or artifacts.
