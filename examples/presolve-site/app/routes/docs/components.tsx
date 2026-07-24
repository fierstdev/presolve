import { component } from "presolve";

@component()
export class Components extends Component {
  render() {
    return <article><h1>Components and state</h1><p>Use a component declaration for a compiler component root, state() for compiler-owned state, and action() for transactional writes. Inputs, JSX structure, routes, imports, and bindings are inferred from ordinary TypeScript.</p></article>;
  }
}
