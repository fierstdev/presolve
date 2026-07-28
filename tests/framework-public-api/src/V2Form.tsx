import {
  Component,
  defineForm,
  field,
  required,
} from "presolve";

export class V2ProfileForm extends Component {
  profile = defineForm({
    serialization: "form-data",
    fields: {
      name: field({
        initial: "",
        validate: [required()],
      }),
    },
    submit: async ({ value, signal }) => {
      await Promise.resolve({ value, signal });
    },
  });

  render() {
    return <input bind:value={this.profile.fields.name} />;
  }
}
