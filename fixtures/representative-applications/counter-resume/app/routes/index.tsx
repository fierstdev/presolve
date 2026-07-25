@component("x-counter")
class Counter extends Component {
  count = state(1)
  increment() { this.count += 1 }
  render() { return <main><h1>Counter</h1><button onClick={this.increment}>{this.count}</button></main>; }
}
