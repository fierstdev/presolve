@component("resume-forms") class ResumeForms {
  @form() @serialize("json") profile!: Form;
  @validate(required()) @field("profile", "identity.name") name = "";
  submitted = state(0);

  @action() @submit("profile")
  save(): void { this.submitted += 1; }

  render() {
    return <form form={this.profile}><input field={this.name}/><span>{this.submitted}</span></form>;
  }
}
