# Project structure

An ordinary Presolve application is discovered from its project root:

```text
my-app/
├── app/
│   ├── app.tsx
│   ├── app.css
│   ├── index.html
│   └── routes/
│       ├── index.tsx
│       └── docs/
│           └── getting-started.tsx
├── package.json
└── tsconfig.json
```

`app/app.tsx` is the application shell. It composes shared providers,
navigation, and footer around route content without owning document metadata.
`app/app.css` is the global stylesheet: Presolve publishes it as `/app.css`
and inserts its link in the generated document head. `app/index.html` is a
compiler template, not a traditional HTML entry point; it must include exactly
one `{{ head }}`, `{{ app }}`, and `{{ runtime }}` placeholder. The compiler
owns what those placeholders contain.

`app/routes/index.tsx` is `/`. Nested directories create nested path segments,
so `app/routes/docs/getting-started.tsx` is `/docs/getting-started/`. Route
topology is compiler-discovered; applications do not maintain a parallel router
table. `app/layout.tsx` and `styles/` remain beta compatibility paths, but new
applications should use the canonical files above.

Each route exports a compiler component. The following home route is complete:

```tsx
import { Component } from "presolve";

export class Home extends Component {
  render() {
    return <main><h1>Hello, Presolve</h1></main>;
  }
}
```

Use ordinary relative TypeScript imports for shared components. Keep application
source under `app/` unless an explicit compiler integration requires a
hermetic source configuration.
