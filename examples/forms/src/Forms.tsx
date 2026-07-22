@component("profile")
class Profile {
  @form() @serialize("json") profile!: Form;
  @field(this.profile) name = "";

  @action() @submit(this.profile)
  save(): void {}

  render() {
    return <form form={this.profile}><input field={this.name} /></form>;
  }
}
