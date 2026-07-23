# Phase N N6-C14 resource read design

N6-C14 makes a Resource result visible only as a compiler-owned expression
source. It does not expose a framework signal, Promise, cache, mutable record,
or arbitrary resource callback.

## Admitted source form

Within a `@computed()` getter owned by the same component that declares the
Resource, the compiler may admit exactly one direct static projection:

```tsx
@resource("loadProfile") profile!: Resource<Profile, ProfileError>;

@computed()
get profileName() {
  return this.profile.data;
}
```

The first implementation admits `.data`, `.error`, and `.state` only. The
field must resolve to a valid N6-C13 Resource declaration. Chained paths,
optional access, bracket access, resource reads in render, Actions, Effects,
Context, Forms, and calls on a resource projection are rejected until their
own dependency and lifecycle contracts exist.

## Type and lifecycle rules

The compiler assigns the Resource field its declared `Resource<Data, Error>`
type and gives projections the following exact types:

* `.data`: `Data | null`;
* `.error`: `Error | null`;
* `.state`: compiler-owned lifecycle text (`idle`, `pending`, `ready`,
  `failed`, or `cancelled`).

The initial cold projection is `{ data: null, error: null, state: "idle" }`.
At runtime activation begins before initial computed evaluation, so the first
computed observation is the `pending` lifecycle with null data and error. A
successful endpoint makes only `data` non-null; a failed endpoint makes only
`error` non-null; cancellation preserves null data and error. Package values
must still pass the existing JSON-serializability boundary before `ready`.

## Compiler products

The expression graph records a Resource read against the exact `ResourceId`.
Canonical IR gains `LoadResource { declaration }`; it is not a state storage
read and cannot be synthesized from an arbitrary string. The runtime-computed
artifact gains an exact `load-resource` instruction and a versioned resource
invalidation map from declaration ID to the Computed IDs that directly read it.
All resource dependencies are compiler-issued, sorted, and inspectable.

Each generated runtime evaluation uses its active component-instance identity
to select exactly one activation for the declared Resource. Missing or
ambiguous activation records fail boot. This keeps repeated component instances
isolated and prohibits global resource lookup.

## Scheduler and runtime ordering

The runtime installs Resource activation records before the cold computed plan,
then starts their endpoint work. Each terminal lifecycle transition invalidates
only the exact Computed records named by the compiler artifact for the affected
component instance, runs the existing computed plan, updates ordinary bindings,
then runs any separately admitted downstream scheduler phases. It does not
discover listeners, inspect DOM, or use a framework subscription store.

N6-C14 initially prohibits Resource reads from Context and Effects, so no
additional ordering is implied for those products.

## Resume boundary

Resource result reads are not resumable in N6-C14. A resume artifact containing
a computed Resource read must fail closed with a resource-resume diagnostic
until N6-C15 defines a versioned snapshot codec, integrity verification,
generation handling, and a cold/snapshot restoration policy. This is stricter
than silently re-fetching after a resume because the package contract's
snapshot vocabulary is not yet an executable codec.

## Required proof

Completion needs positive and negative compiler fixtures, exact IR and runtime
artifact assertions, repeated-instance identity proof, malformed activation
failure, terminal state invalidation proof, a generated-page browser proof that
updates a real binding after endpoint resolution, and resume rejection proof.
