# Production benchmark corpus

`corpus.json` fixes sixteen production correctness cases. `budgets.json`
records committed output ceilings from the repository's pinned toolchain.

The budgets cover generated production JavaScript, the eager module, the packed
artifact, runtime records, and compiler-defined static operation units. The
shared-candidate case also preserves its earlier resumability baseline. The lifecycle case
requires exactly 100 create/destroy cycles with all instance-owned registries
returning to their starting counts.

Wall-clock measurements are informational only. They are never committed as a
normative threshold because host timing is nondeterministic. A baseline may be
updated only with an explained correctness cost; it must not be loosened solely
to make a regression pass.
