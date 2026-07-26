# V2 effect-field source contract

The V2 language declares a browser effect as a class field:

```ts
titleEffect = effect(() => {
  document.title = this.title;
  return () => {
    document.title = "";
  };
});
```

This contract establishes the source and authority boundary for that form. It
does not claim runtime adoption until the runtime can preserve the specified
cleanup, declaration-order, activation, and resume behavior.

## Recognition

The parser selects direct, non-static class-field initializer calls without
assigning framework meaning. `effect_field_lowering` may create a canonical
`Effect` declaration only when all of the following are true:

1. the field belongs to a canonical V2 Component;
2. the installed TypeScript authority resolves the initializer callee to the
   canonical `effect` intrinsic; and
3. the response range joins exactly to the parser-retained callee span.

The source spelling of either the import or call is not authority. Aliases are
therefore admitted when the resolved symbol is canonical, while lookalikes and
unresolved calls remain ordinary JavaScript.

## Product and scope

The lowering records the canonical field subject, source provenance, and
resolved intrinsic identity in `CanonicalAuthoredSemanticModelV1`. It adds no
legacy decorator, method, execution carrier, or source translation.

The parser's generic inline-handler facts include a restricted ordered-body
view for block-bodied inline functions. That view remains syntax only until an
authority-backed consumer selects the surrounding call; this recognition slice
neither treats it as cleanup proof nor publishes an executable effect. In
particular, cleanup-return functions must remain outside runtime adoption until
a dedicated semantic and runtime product can execute cleanup before
re-execution and disposal.

## Required later adoption proof

A runtime-adoption amendment must prove, for authority-backed V2 effect fields:

- browser-only execution and no server-publication execution;
- one eligible run after cold activation and after resume;
- dependency-triggered re-execution;
- synchronous cleanup before re-execution and disposal;
- field declaration order, parent-before-child activation, and
  child-before-parent cleanup; and
- rejection of async effect or cleanup callbacks before publication.

Until that amendment lands, a canonical V2 `Effect` declaration is source
evidence only and must not be silently routed through the decorator runtime.
