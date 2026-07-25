# Project structure

An ordinary Presolve application is discovered from its project root:

```text
my-app/
├── app/
│   ├── layout.tsx
│   └── routes/
│       ├── index.tsx
│       └── docs/
│           └── getting-started.tsx
├── package.json
└── tsconfig.json
```

`app/routes/index.tsx` is `/`. Nested directories create nested path segments,
so `app/routes/docs/getting-started.tsx` is `/docs/getting-started/`. An
optional `app/layout.tsx` composes the route content. Route topology is
compiler-discovered; applications do not maintain a parallel router table.

Each route exports a compiler component. The following home route is complete:

```tsx
import { component, Component } from "@presolve/core";

@component()
export class Home extends Component {
  render() {
    return <main><h1>Hello, Presolve</h1></main>;
  }
}
```

Use ordinary relative TypeScript imports for shared components. Keep application
source under `app/` unless an explicit compiler integration requires a
hermetic source configuration.
