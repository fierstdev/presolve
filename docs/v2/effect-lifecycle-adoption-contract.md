# V2 effect lifecycle adoption contract

This contract advances authority-backed V2 `effect(handler)` fields from
source/semantic evidence to browser lifecycle execution. It implements the
base V2 effect requirements without converting a field into a legacy method.

## Canonical inputs

The adapter consumes only a canonical V2 `Effect` declaration and its matching
`ComponentEffectField`. The field retains an ordered main body, an optional
ordered cleanup body, the async fact for both callbacks, and field provenance.
The effect semantic entity records `EffectDeclaration::V2Field`; legacy
decorated effects retain `LegacyMethod`.

No runtime path may infer an effect from its name, import spelling, decorator,
or raw source text.

## Validation and artifact boundary

Both the main and cleanup callbacks must be synchronous and must pass the
existing browser capability/type validation. Cleanup is not an ordinary return
value: its body becomes a separately identified cleanup program. A malformed
cleanup, async callback, unsupported operation, or unavailable program rejects
publication before runtime execution.

`RuntimeEffectArtifact` advances to schema v2 with an optional cleanup program
for each effect. The artifact keeps stable effect IDs and declaration-order
scheduling; it contains executable compiler instructions only, never raw
source or callback objects.

## Browser lifecycle

The browser runtime owns an instance-local cleanup registry keyed by canonical
effect ID. It must:

1. execute eligible main effects only after the instance becomes active;
2. run a prior cleanup before the same effect re-executes;
3. register the cleanup program produced by the latest successful main run;
4. execute cleanup during disposal in reverse field declaration order;
5. execute an eligible effect once after resume activation; and
6. keep effects out of server publication/execution.

Parent activation precedes child activation; child cleanup precedes parent
cleanup. The compiler supplies declaration ordering rather than relying on
object-key iteration.

## Acceptance

- A decorator-free `effect(() => { document.title = this.title; return () => {
  document.title = ""; }; })` retains distinct main and cleanup programs.
- A State-triggered rerun invokes cleanup before the next main program.
- Resume executes each eligible effect once after restoration, with no server
  execution and no duplicate active subscription.
- Disposal invokes cleanups in reverse field order; nested component fixtures
  prove child-before-parent cleanup.
