# Phase Q Q2 navigation topology

**Status:** in progress.

The compiler route manifest now includes a deterministic `parent_path` for
each static URL segment. This is route topology for navigation and inspection;
browser navigation remains normal document navigation.

No authored layout decorator or runtime layout composition is admitted because
the frozen compiler has no authoritative layout declaration/lowering. Dynamic
parameters likewise remain deferred until their compiler identity, typed input,
and artifact selection contracts are authored. Q2 therefore advances nested
static topology without inventing a router runtime.
