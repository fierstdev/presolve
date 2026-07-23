# Phase N N3-B static Action parameters contract

N3-B admits a deliberately bounded Action data path. A compiler-recognized
Action may assign one of its declared primitive parameters to a complete State
field, and a compiler-recognized event callback may supply that parameter with
an exact serializable literal:

```tsx
@component("x-parameterized-action")
class ParameterizedAction extends Component {
  label = state("Ready");

  @action() setLabel(value: string) {
    this.label = value;
  }

  render() {
    return <button onClick={() => this.setLabel("Locked")}>{this.label}</button>;
  }
}
```

The parameter must have an explicit `string`, `number`, `boolean`, or `null`
annotation. The callback argument count and each literal's primitive kind must
match the method declaration. An Action state assignment may refer only to a
parameter declared by that method.

The compiler owns the whole path: parser fact, Action parameter identity,
event binding, static argument list, `assign_parameter` Action operation,
ordinary-template-instance event record, template manifest, component runtime
artifact, and one completed Action batch. Generated runtime code reads the
already-emitted argument by its compiler-issued parameter position and writes
the existing component-instance State slot. It neither reads a browser event
payload nor executes the callback closure.

This changes the template manifest to schema version `5` and the component
runtime artifact to schema version `4`. Both carry the static argument list for
ordinary events, and the runtime fails closed when their lists disagree.

`PSC1041` rejects an unknown, untyped, or non-`@action()` parameter used as a
State assignment source. `PSC1042` rejects an event callback whose static argument count differs
from the referenced method. `PSC1043` rejects a statically incompatible
primitive callback argument.

N3-B does not admit browser-event payload projection, arbitrary callbacks,
captured reactive values, object/array parameter annotations, default/rest or
destructured parameters, parameter forwarding, nested property writes, async
Actions, or generic Action-body interpretation. Those forms require their own
compiler contracts.
