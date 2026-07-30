# Project structure and ownership

A Presolve application is an ordinary TypeScript project with one conventional
home for each kind of authored responsibility:

```text
my-app/
├── app/
│   ├── app.tsx
│   ├── app.css
│   ├── index.html
│   ├── components/
│   └── routes/
│       ├── index.tsx
│       └── docs/
│           ├── index.tsx
│           └── getting-started.tsx
├── assets/
├── public/
│   └── favicon.svg
├── server/
├── tests/
├── package.json
└── tsconfig.json
```

The structure is not aesthetic convention. It prevents document metadata,
application composition, route topology, global presentation, server values,
and generated artifacts from becoming competing sources of truth.

## Ownership at a glance

| Path | Owner | Purpose |
| --- | --- | --- |
| `app/index.html` | Application + compiler | The application owns stable document framing and metadata. The compiler owns the required insertion points. |
| `app/app.tsx` | Application shell | Shared navigation, providers, theme UI, footer, and the route slot. |
| `app/routes/` | Presolve route graph | File paths declare URLs; exported component classes declare route content. |
| `app/components/` | Application source | Reusable components that do not create URLs. |
| `app/app.css` | Application presentation | One global stylesheet whose exact bytes are published and linked by Presolve. |
| `public/` | Static publication input | Files copied to root URLs and included in the deployment inventory. |
| `assets/` | Explicit Vite integration | Imported CSS, fonts, and media for an adapter-owned Vite entry. Files are not automatically copied merely because they are here. |
| `server/` | Server source | Server-owned values and capability implementations; location alone does not make a server executor exist. |
| `tests/` | Verification source | Application tests and fixtures. |
| `dist/`, `.presolve/` | Compiler output | Rebuild these directories; never edit or commit them as authored source. |

## The document frame: `app/index.html`

This file is a compiler template, not a traditional browser entry module. It
must contain exactly one `{{ head }}`, `{{ app }}`, and `{{ runtime }}`
placeholder:

```html
<!doctype html>
<html lang="en">
<head>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="description" content="My Presolve application">
  <link rel="icon" href="/favicon.svg" type="image/svg+xml">
{{ head }}
</head>
<body>
{{ app }}{{ runtime }}
</body>
</html>
```

The application owns `lang`, stable metadata, icons, manifests, preloads, and
surrounding markup. Presolve fills the placeholders with compiler-generated
head entries, composed application and route HTML, and the exact runtime
artifact for that route. Do not duplicate those generated values beside the
placeholders.

## The application shell: `app/app.tsx`

The shell wraps every route and projects the selected route through its default
slot:

```tsx
import { Component, slot, type SlotContent } from "presolve";

export class App extends Component {
  children: SlotContent = slot();

  render() {
    return (
      <div class="app-shell">
        <header>Shared navigation</header>
        <slot />
        <footer>Shared footer</footer>
      </div>
    );
  }
}
```

The shell does not render `<html>`, `<head>`, or `<body>` because the document
template owns them. It should not render the primary `<main>` either: each route
owns its page landmark, avoiding invalid nested main elements.

## Routes and reusable components

`app/routes/index.tsx` maps to `/`. Nested directories create path segments, so
`app/routes/docs/getting-started.tsx` maps to
`/docs/getting-started/`. There is no parallel router table.

Each route exports a compiler component:

```tsx
import { Component } from "presolve";

export class Home extends Component {
  render() {
    return <main><h1>Hello, Presolve</h1></main>;
  }
}
```

Use ordinary relative TypeScript imports for components under
`app/components/`. Importing one does not turn it into a route; route identity
comes only from the compiler-discovered route file.

## CSS, assets, and server boundaries

`app/app.css` is the canonical global stylesheet. Presolve publishes its exact
bytes at `/app.css`, emits `/app.<sha256>.css`, and links the immutable
coordinate from the generated document head. See
[Styling, assets, and Vite](styling-and-vite.md) for the complete support
boundary and Tailwind workflow.

Files in `public/` are copied to the publication root. For example,
`public/favicon.svg` is available as `/favicon.svg`. An `assets/` file is
different: it is only an input when an explicit `@presolve/vite` integration
imports or declares it.

Source under `server/` is server-owned. This prevents accidental browser value
publication, but it does not invent a loader, action, capability, or server
runtime. Those require their own compiler-admitted products and deployment
support.

## What happens during a build

1. Presolve discovers route and shell source from the canonical application
   paths.
2. TypeScript proves symbols and signatures; the Presolve compiler derives
   route, component, state, action, lifecycle, and publication products.
3. Project Vite is invoked only for admitted external browser bundles such as
   package Actions, Standard Schema validators, and form-submission
   capabilities.
4. Presolve composes each route into complete HTML using `app/index.html`.
5. It publishes content-addressed CSS and runtime files, copies `public/`, and
   writes one integrity-bound inventory under `dist/`.

This division keeps one semantic authority while still using TypeScript for
language truth and Vite for bounded physical bundling.

## Compatibility paths

`app/layout.tsx` and `styles/` remain readable for older beta projects.
`app/layout.tsx` cannot coexist with `app/app.tsx`, and new projects should not
add either compatibility path. Migrate shared UI to `app/app.tsx`, global CSS
to `app/app.css`, and directly served assets to `public/`.
