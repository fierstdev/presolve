# Styling, assets, Tailwind, and Vite

Presolve separates semantic compilation from physical presentation tooling:

- Presolve owns application meaning, document composition, the global
  stylesheet link, route publication, and deployment integrity.
- TypeScript owns language, symbol, and signature truth.
- Vite owns bounded browser bundling when the compiler or an explicit adapter
  selects an entry.
- A CSS tool such as Tailwind or PostCSS owns its transformation into ordinary
  CSS.

That separation explains both what works automatically and what requires an
explicit build step.

## Canonical global CSS

Author ordinary global CSS in `app/app.css`:

```css
:root {
  color-scheme: dark;
  --canvas: #07090f;
  --ink: #f7f8fc;
  --accent: #75dcf5;
}

* { box-sizing: border-box; }
body { margin: 0; background: var(--canvas); color: var(--ink); }
:focus-visible { outline: 3px solid var(--accent); outline-offset: 3px; }

.feature-grid {
  display: grid;
  gap: 1rem;
}

@media (min-width: 48rem) {
  .feature-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); }
}
```

Presolve reads that file as bytes. A production build emits byte-identical
`dist/app.css` and `dist/app.<sha256>.css` files and links the immutable file
from every generated route document. The compatibility file exists for hosts
and tooling, but generated HTML uses the content-addressed coordinate so a
returning browser cannot combine new HTML with stale CSS.

Because the file is published directly, browser-standard CSS works: selectors,
custom properties, media and container queries, modern layout, animations,
font faces, and accessibility media features. Presolve does not rename classes
or attach stylesheet nodes to a component subtree.

`app/app.css` is not automatically run through Vite, PostCSS, Sass, or
Tailwind. CSS syntax requiring a transform must first be compiled into this
file or be supplied through an explicit adapter-owned Vite entry.

## Classes in TSX

Use `class` in framework source. `className` remains accepted for JSX ecosystem
compatibility:

```tsx
export class FeatureCard extends Component {
  render() {
    return (
      <article class="feature-card">
        <h2>Complete HTML</h2>
        <p>Useful before JavaScript executes.</p>
      </article>
    );
  }
}
```

Class attributes are application presentation. They do not create component,
state, action, or resume identity.

## Public assets

Place directly addressed files in `public/`:

```text
public/
├── favicon.svg
├── manifest.webmanifest
├── images/
│   └── social-card.png
└── robots.txt
```

Refer to them from the document or TSX using root paths:

```html
<link rel="icon" href="/favicon.svg" type="image/svg+xml">
```

```tsx
<img src="/images/social-card.png" alt="Presolve compiler product flow" />
```

Presolve copies each file to the root of `dist/`, rejects collisions with
compiler artifacts, and records the file in the deployment inventory. A
missing public file therefore cannot be silently omitted from a valid prepared
release.

## Tailwind

Tailwind is supported as a build-time CSS compiler. It adds no browser runtime.
Keep its authored input separate from Presolve's generated input:

```css
/* app/tailwind.css */
@import "tailwindcss";

@theme {
  --color-brand-cyan: #75dcf5;
  --color-brand-purple: #9d8bff;
}

@import "./site.css";
```

Install the official CLI:

```sh
pnpm add -D tailwindcss @tailwindcss/cli
```

Compile the final file before Presolve starts:

```json
{
  "scripts": {
    "css:build": "tailwindcss -i ./app/tailwind.css -o ./app/app.css --minify",
    "css:watch": "tailwindcss -i ./app/tailwind.css -o ./app/app.css --watch",
    "dev": "pnpm css:build && presolve dev",
    "build": "pnpm css:build && presolve build"
  }
}
```

Run `pnpm css:watch` in a second terminal while authoring styles. Use complete,
literal utility class names in TSX so Tailwind's source scan remains
deterministic:

```tsx
<section class="grid gap-4 rounded-xl border p-4 md:grid-cols-2">
  <h2 class="text-brand-cyan">Static CSS, exact publication.</h2>
</section>
```

Do not construct partial utility names such as `"text-" + this.color`; Tailwind
cannot discover values that do not appear as complete candidates in source.

## How CSS reaches a component or route

Presolve does not attach a stylesheet to a component instance. The connection
is the browser's ordinary document cascade:

1. A component or route renders a literal `class` or compatible `className`
   attribute into its static HTML.
2. The application owns matching selectors in `app/app.css`, or runs Tailwind,
   PostCSS, Sass, or another transformer that writes finished CSS to that file.
3. Presolve reads the final bytes, publishes byte-identical `dist/app.css` and
   immutable `dist/app.<sha256>.css` artifacts, and records both in the
   deployment inventory.
4. Every generated route document receives one compiler-owned
   `<link rel="stylesheet">` in its `<head>` pointing at the immutable file.
5. The browser applies those selectors to the complete application shell,
   layouts, routes, and nested components through normal inheritance,
   specificity, cascade layers, media queries, and container queries.

This is global CSS. A route does not need to import `app.css`, and a component
must not render its own global `<link>`. Presolve does not rename selectors,
scope class names, or remove the stylesheet when a component subtree changes.

During `pnpm dev`, a change to finished CSS triggers a compiler rebuild and a
CSS hot swap through `/app.css?presolve-dev=<revision>`. Component state, focus,
scroll position, and the current document remain intact. A TSX, document, route,
public-file, package, or configuration edit rebuilds from compiler authority
and uses a full reload unless a narrower HMR product proves state compatibility.
Compilation errors keep the last good page visible and appear in an accessible
development alert; correcting the source reloads the recovered publication.

For Tailwind or another transformer, keep its watch process writing
`app/app.css`. Presolve observes that completed output—not the transformer's
private source graph—and hot-swaps the resulting browser CSS.

## Why Vite is installed

The standard scaffold includes project-local Vite because Presolve uses it as a
physical bundler for compiler-authorized external browser code:

- named package exports called by an admitted Action;
- Standard Schema validators;
- imported form-submission capability exports.

Presolve first proves the exact module, export, signature, arguments, and
lifecycle. It then asks Vite to bundle only that authorized browser entry and
includes the result in the compiler publication inventory. Vite does not decide
that a call is an Action, that a validator owns a Field, or that a route is
browser eligible.

Run `presolve dev` and `presolve build`, not bare `vite`, for the canonical
application workflow. A bare Vite server has no authority to compose Presolve
route HTML or compiler artifacts.

## The `@presolve/vite` adapter

`@presolve/vite` is a public integration API for adapter and tool authors. It
can transport a digest-verified compiler publication through Vite, expose
versioned virtual artifact modules, host compiler-owned requests during
development, deliver compiler-classified HMR, and build explicit physical
entries:

```js
import { buildPresolveProduction } from "@presolve/vite";

const output = await buildPresolveProduction({
  compilerProduct,
  readArtifact,
  entryArtifactPath: "routes/root/runtime.js",
  viteEntries: [
    { name: "application-ui", path: "assets/ui-entry.js" },
  ],
  vite: {
    root: process.cwd(),
    publicDir: "public",
    build: { outDir: "dist/vite-assets" },
  },
});
```

An explicit entry can import CSS Modules, PostCSS/Tailwind output, fonts, or
media. Those files receive Vite physical identities and hashes, not Presolve
component or route identities. The caller must integrate the returned physical
output into its host; adding `assets/` or a `vite.config.ts` file alone does not
change the canonical CLI build.

## Supported boundary

| Form | Canonical CLI status |
| --- | --- |
| Ordinary global CSS in `app/app.css` | Automatic, byte-exact, content-addressed publication. |
| Root-addressed files in `public/` | Automatic copy and deployment inventory. |
| Tailwind CLI output written to `app/app.css` | Supported documented workflow. |
| Sass/PostCSS output written to `app/app.css` | Supported when the project runs the transformer first. |
| CSS Modules and imported media through `buildPresolveProduction()` | Supported adapter API; not implicit scaffold discovery. |
| `vite.config.ts` changing Presolve routes or semantics | Unsupported. |
| CSS imported directly from a component and assumed globally linked | Not a canonical CLI feature. |
| Runtime CSS-in-JS that invents reactive semantics | Not compiler-understood unless a separate admitted package contract exists. |

Keep the distinction visible: a tool may transform physical bytes without
gaining authority over application meaning.
