import { component, Component } from "presolve";

@component()
export class Composition extends Component {
  render() {
    return <article><h1>Composition</h1><p>Ordinary instance fields express typed component inputs. Slots compose authored content, while Context makes provider ownership explicit through context(), provide(), and consume().</p><p>Presolve preserves instance identity and slot ownership in compiler artifacts instead of resolving them through a generic runtime.</p></article>;
  }
}
