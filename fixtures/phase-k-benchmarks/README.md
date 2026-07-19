# Phase K production benchmark corpus

`corpus.json` fixes the sixteen normative K16 correctness cases. `budgets.json`
records exact K15-output ceilings from the repository's pinned toolchain.

The budgets cover generated production JavaScript, the eager module, the packed
artifact, runtime records, and compiler-defined static operation units. The
shared-candidate case also preserves its Phase J comparison. The lifecycle case
requires exactly 100 create/destroy cycles with all instance-owned registries
returning to their starting counts.

Wall-clock measurements are informational only. They are never committed as a
normative threshold because host timing is nondeterministic. A baseline may only
be updated before the Phase K freeze to record an explained correctness cost;
it must not be loosened solely to make a regression pass.
