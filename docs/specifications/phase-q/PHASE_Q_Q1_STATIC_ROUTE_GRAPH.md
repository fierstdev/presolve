# Phase Q Q1 static route graph

**Status:** in progress.

The compiler now exposes a validated schema-v1 static route graph/manifest over
existing `@route("/path")` component records. Paths are sorted deterministically
and reject dynamic segments and duplicate ownership. This is a compiler
identity product only; it does not yet publish route artifacts or expose a CLI
route command.

The next Q1 slice must bind this manifest to a complete explicit multi-entry
publication request. It may not use source discovery or construct route pages
in JavaScript.
