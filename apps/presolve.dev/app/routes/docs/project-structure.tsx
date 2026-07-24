import { component, Component } from "presolve";

@component()
export class ProjectStructure extends Component {
  render() {
    return <article><h1>Project structure</h1><p>Routes live in app/routes. An index route becomes its directory URL, nested directories become URL segments, and app/layout.tsx composes route content.</p><pre>{"app/routes/index.tsx\napp/routes/docs/getting-started.tsx"}</pre><p>There is no route registry to maintain for a conventional Presolve application.</p></article>;
  }
}
