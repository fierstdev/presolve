import {
  action,
  component,
  field,
  form,
  serialize,
  state,
  submit,
  Component,
  type Form,
} from "presolve";

@component()
export class FormHost extends Component {
  @form() @serialize("json") profile!: Form;
  @field("profile", "address.street") street = "";
  @field("profile", "address.city") city = "";
  submitted = state(0);

  @action() @submit("profile")
  save(): void {
    this.submitted += 1;
  }

  render() {
    return <form form={this.profile}><input field={this.street}/><input field={this.city}/><span>{this.submitted}</span></form>;
  }
}
