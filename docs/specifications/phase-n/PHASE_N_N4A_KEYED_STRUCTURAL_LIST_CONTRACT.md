# Phase N N4-A keyed structural list contract

N4-A admits the existing compiler-owned keyed list model: a statically
identified iterable, explicit unique primitive key, compiler-issued list and
item identities, retained keyed roots, and deterministic insertion/removal.
The generated runtime reconciles only the compiler artifact; DOM position is
not semantic identity. Dynamic arbitrary keys, index identity, callbacks beyond
the existing list grammar, and unregistered nested behavior remain excluded.
