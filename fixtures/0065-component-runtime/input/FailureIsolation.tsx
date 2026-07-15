@component("x-safe-leaf") class SafeLeaf extends Component { render() { return <span>Safe</span>; } }
@component("x-failure-page") @route("/failure") class FailurePage extends Component {
  render() { return <main><Missing /><SafeLeaf /></main>; }
}
