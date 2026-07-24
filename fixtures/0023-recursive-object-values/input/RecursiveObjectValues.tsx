@route("/recursive-object-values")
@component("x-recursive-object-values")
class RecursiveObjectValues extends Component {
  profile = state({
    name: "North",
    settings: {
      enabled: true,
      tags: ["compiler", { rank: 2, name: "runtime" }]
    }
  });

  replace() {
    this.profile = {
      name: "South",
      settings: { enabled: false, tags: ["runtime"] }
    };
  }

  render() {
    return (
      <section>
        <button onClick={() => this.replace()}>Replace</button>
        <p>{this.profile}</p>
      </section>
    );
  }
}
