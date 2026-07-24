@route("/fragments")
@component("x-fragment-panel")
class FragmentPanel extends Component {
  label = state("Ready");

  render() {
    return (
      <>
        <h1>Title</h1>
        <>
          <p>Status: {this.label}</p>
          <span>Done</span>
        </>
      </>
    );
  }
}
