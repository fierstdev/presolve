@component("profile")
class Profile {
  @form() @serialize("json") profile!: Form;
  @field("profile") name = "";

  @action() @submit("profile")
  save(): void {}

  render() {
    return <form form={this.profile}><input field={this.name} /></form>;
  }
}
