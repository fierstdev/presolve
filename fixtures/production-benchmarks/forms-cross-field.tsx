@component("x-production-form") @route("/")
class ProductionForm extends Component {
  @form() @serialize("json") account!: Form;
  @validate(required()) @field(this.account) password = "";
  @validate(equals(this.password)) @field(this.account) confirmation = "";

  @action() @submit(this.account)
  save(): void {}

  render() {
    return <form form={this.account} onSubmit={this.save}>
      <input field={this.password} />
      <input field={this.confirmation} />
    </form>;
  }
}
