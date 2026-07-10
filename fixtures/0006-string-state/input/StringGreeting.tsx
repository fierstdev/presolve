@route("/string-greeting")
@component("x-string-greeting")
class StringGreeting extends Component {
  name = state("Austin & <Zero>");

  render() {
    return <p>Name:{this.name}</p>;
  }
}
