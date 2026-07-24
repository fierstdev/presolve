import { component, Component } from "presolve";

@component()
export class Editor extends Component {
  render() {
    return <article><h1>VS Code</h1><p>Install the Presolve extension and open the project folder. The workspace TypeScript configuration and the presolve package types provide ordinary TypeScript and TSX diagnostics without suppression.</p><p>Use Presolve: Show Workspace Status to confirm the project configuration.</p></article>;
  }
}
