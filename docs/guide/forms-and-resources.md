# Forms and resources

Forms, resources, and their browser artifacts are compiler-owned capabilities.
Use their declarations only where their ownership and endpoint behavior are
clear to the compiler.

## Forms

`@form()` declares a component-owned form. `@field()` associates a stateful
field with it, and `@submit()` declares the action that handles its submission.

```tsx
import {
  action, component, field, form, serialize, submit, Component, type Form,
} from "presolve";

@component()
export class ProfileForm extends Component {
  @form() @serialize("json") profile!: Form;
  @field("profile", "identity.name") name = "";

  @action() @submit("profile")
  save(): void {
    // Persist through an admitted capability boundary.
  }

  render() {
    return <form form={this.profile}><input field={this.name} /></form>;
  }
}
```

Supported serialization formats are `json`, `form-data`, and `url-encoded`.
`required()` creates a validation rule and `@validate(rule)` attaches one to a
field. The alpha does not provide arbitrary custom controls, files, async or
server validation, network submission, or automatic reset behavior.

## Resources

Use `@resource(endpoint)` for a compiler-declared resource field. A resource
has `data`, `error`, and `state` (`idle`, `pending`, `ready`, `failed`, or
`cancelled`). `@loader(endpoint)` and `@serverAction(endpoint)` declare the
corresponding explicit endpoint boundaries.

Endpoints must be present in the compiler's application/package capability
information. The static Cloudflare adapter rejects executable server handoffs;
do not present a resource declaration as a general server-runtime substitute.
