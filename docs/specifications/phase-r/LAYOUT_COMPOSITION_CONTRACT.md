# Phase R Layout Composition Contract

## Objective

The compiler turns a file-route layout chain into one planned component instance
tree. It must not concatenate pre-rendered HTML or add a JavaScript router.

For a page at `app/routes/blog/[slug].tsx`, the ordered chain is:

```text
app/layout.tsx → app/routes/blog/layout.tsx → page
```

The compiler owns the synthetic composition edges that connect each outer
layout to the next component through its default Slot.

## Admission

Each participating layout must declare exactly one default `@slot()` field.
Its template must resolve that Slot through one ordinary outlet for the child
to materialize; existing canonical Slot diagnostics reject missing, duplicate,
or invalid outlets. Named-only and duplicate default declarations are rejected
for automatic composition. The page remains an ordinary component entry and
does not require generated source or an authored wrapper class.

## Identity and ownership

The compiler issues deterministic virtual invocation IDs from the layout,
route page, and chain position. These are not source decorator IDs and never
appear in ordinary framework types.

Every synthetic child instance has the preceding layout instance as parent.
The existing instance, template, event, Context, form, slot, action, and resume
registries consume that same planned topology. The page's rendered descendants
remain page-owned; only placement is supplied by the layout Slot projection.

## Publication

The file-route publication product materializes only the composed route root.
Its route manifest retains the ordered layout component IDs as evidence, but
the page's artifacts remain standard compiler-generated runtime/resume
products. A route with no layout uses its page as the root.

## Fail-closed behavior

* `PSROUTE1020_LAYOUT_DEFAULT_SLOT_MISSING`
* `PSROUTE1021_LAYOUT_DEFAULT_SLOT_AMBIGUOUS`
* `PSROUTE1022_LAYOUT_COMPOSITION_UNSUPPORTED`

No fallback static wrapping is allowed. A rejected layout cannot silently
produce a page with different instance or resumability semantics.
