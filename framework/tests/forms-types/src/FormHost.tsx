@component("form-host") class FormHost {
  @form() @serialize("json") profile!: Form;
  @field("profile", "address.street") street = "";
  @field("profile", "address.city") city = "";
  submitted = 0;

  @action() @submit("profile")
  save(): void { this.submitted += 1; }

  render() {
    return <form form={this.profile}><input field={this.street}/><input field={this.city}/><span>{this.submitted}</span></form>;
  }
}
