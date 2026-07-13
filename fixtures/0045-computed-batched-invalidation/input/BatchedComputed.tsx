@component("x-batched-computed")
class BatchedComputed extends Component {
  count = state(0);

  @computed()
  get doubled() { return this.count * 2; }

  @computed()
  get label() { return this.doubled + 1; }

  @action()
  incrementTwice() {
    this.count += 1;
    this.count += 1;
  }

  render() {
    return <button onClick={this.incrementTwice}>Increment twice</button>;
  }
}
