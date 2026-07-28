import {
  Component,
  defineForm,
  field,
  required,
} from "presolve";
import { displayNameSchema } from "./V2Schemas.js";

type Uploads = File[];

export class V2ProfileForm extends Component {
  profile = defineForm({
    serialization: "form-data",
    fields: {
      name: field({
        initial: "",
        validate: [required(), displayNameSchema],
      }),
      attachments: field<Uploads>({
        initial: [],
      }),
    },
    submit: async ({ value, signal }) => {
      await Promise.resolve({ value, signal });
    },
  });

  render() {
    return <div>
      <input bind:value={this.profile.fields.name} />
      <input type="file" bind:files={this.profile.fields.attachments} />
    </div>;
  }
}
