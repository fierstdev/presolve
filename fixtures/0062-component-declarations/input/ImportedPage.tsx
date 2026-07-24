import { Card as ImportedCard } from "./ValidComponents";

@component("x-imported-page")
@route("/imported")
export class ImportedPage extends Component {
  render() {
    return <ImportedCard><template slot="header"><h1>Title</h1></template><p>Body</p><button>Save</button></ImportedCard>;
  }
}
