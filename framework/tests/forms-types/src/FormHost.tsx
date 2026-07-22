@component("form-host") class FormHost {
  @form() @serialize("json") profile!: Form;
  @field("profile") name = "";
  submitted = 0;

  @action() @submit("profile")
  save(): void { this.submitted += 1; }

  render() {
    return <form form={this.profile}><input field={this.name}/><span>{this.submitted}</span></form>;
  }
}
