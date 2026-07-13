@component("x-computed-diamond")
class ComputedDiamond extends Component {
  count = state(1);

  @computed()
  get doubled() { return this.count * 2; }

  @computed()
  get tripled() { return this.count * 3; }

  @computed()
  get total() { return this.doubled + this.tripled; }

  @action()
  increment() {
    this.count += 1;
  }

  render() {
    return <button onClick={this.increment}>Increment</button>;
  }
}
