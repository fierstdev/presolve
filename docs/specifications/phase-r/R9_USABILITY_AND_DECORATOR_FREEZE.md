# Phase R usability and decorator-minimization freeze

## Normal application experience

A new application is created with `npm create presolve <directory>`. It needs
no project configuration, source list, route registry, component-name string,
or artifact command. The normal loop is:

```text
npm run dev
npm run check
npm run build
npm run deploy:prepare
npm run deploy
```

Routes come from `app/routes`, layouts from `app/layout.tsx`, component
identity from `@component()`, and deployment facts from compiler publication.
The public `presolve` import is normal TypeScript 7-native-decorator vocabulary
with no runtime registration, React-style renderer, or semantic package
contract requirement.

## Decorator budget

Presolve has 17 public decorators, but they are not a per-component checklist.
They are separated into a small normal core and explicit boundary vocabulary.

| Kind | Public forms | Inferred instead |
| --- | --- | --- |
| Component/reactivity core | `@component`, `@action`, `@computed`, `@effect`; `state()` is a function | component name, inputs, render structure, event bindings, dependencies, caching, batching, and runtime scheduling |
| Composition boundary | `@slot`, `@context`, `@provide`, `@consume` | file-route layout membership, route hierarchy, component invocation, and slot placement |
| Form boundary | `@form`, `@serialize`, `@field`, `@validate`, `@submit`; `required()` is a rule value | form host binding, Field identity, validation topology, submission serialization, and updates |
| External capability boundary | `@resource`, `@loader`, `@serverAction`, `@opaque` | package import/binding coordinates, dependency graph, cache/runtime policy, and artifact inclusion |

Only the first row is normal reactive component vocabulary. Most static pages
need exactly `@component()`. A stateful component usually adds `state()` and
one `@action()`, and uses no field decorators. Forms, Context, slots, data
loading, and opaque packages carry distinct ownership or capability meaning, so
they remain explicit rather than becoming ambiguous inference rules.

`@serverAction()` is itself an Action boundary. It must not be combined with
`@action()`; its empty method form prevents nested or duplicate transactions.

No new decorator may enter the public package merely to save typing. Admission
requires proof that ordinary TypeScript, imports, JSX, route topology, or an
existing declaration cannot express the needed compiler authority without
ambiguity. Conversely, an existing marker is removed when its information can
be inferred losslessly and a compiler contract, diagnostics, and TypeScript
migration prove the replacement.

## TypeScript 7 contract

The public declarations use standard decorator context types and are verified
with the repository-pinned TypeScript 7.0 compiler. TypeScript 7.1 remains a
compatibility target: before it becomes a supported baseline, the public-package
type matrix and fresh scaffold proof must rerun unchanged or this contract must
be amended with a concrete remediation.

`Component` and branded authoring types such as `SlotContent` are exported from
`presolve` and are imported explicitly in ordinary source. JSX intrinsic tags
remain deliberately broad in the declaration package because compiler JSX
validation—not a second framework DOM typing authority—decides final admission.

## R9 acceptance evidence

The freeze requires all of the following:

* TypeScript 7 validates a public-package component using standard decorators;
* `create-presolve` creates a new project without overwriting an existing path;
* the fresh project passes `check`, `build`, and Cloudflare `--prepare` without
  configuration/manual source lists/manual component identities;
* compiler route and deployment explanations report canonical products;
* the Presolve public-site dogfood example checks and builds through conventional
  layout/file-route publication; and
* Cloudflare preparation validates the immutable artifact inventory before a
  provider command runs.

`scripts/verify-r9-usability-freeze.sh` is the focused reproducible evidence
matrix for these checks.

Server loader/action handoffs are intentionally not claimed as executable in
the first static Cloudflare adapter; any attempt fails closed. A future
executor must extend the compiler capability contract, not change this freeze
into a generic server framework.
