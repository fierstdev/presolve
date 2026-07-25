import {
  action,
  component,
  field,
  form,
  required,
  serialize,
  state,
  submit,
  validate,
  Component,
  type Form,
} from "@presolve/core";

@component()
export class ResumeForms extends Component {
  @form() @serialize("json") profile!: Form;
  @validate(required()) @field("profile", "identity.name") name = "";
  submitted = state(0);

  @action() @submit("profile")
  save(): void {
    this.submitted += 1;
  }

  render() {
    return <form form={this.profile}><input field={this.name}/><span>{this.submitted}</span></form>;
  }
}
