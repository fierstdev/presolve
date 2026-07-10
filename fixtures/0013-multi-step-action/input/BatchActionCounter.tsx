@route("/batch-action-counter")
@component("x-batch-action-counter")
class BatchActionCounter extends Component {
  count = state(1);
  enabled = state(false);

  apply() {
    this.count += 2;
    this.count--;
    this.count = 8;
    this.count++;
    this.enabled = !this.enabled;
  }

  render() {
    return (
      <button onClick={() => this.apply()}>
        Count: {this.count} Enabled: {this.enabled}
      </button>
    );
  }
}
