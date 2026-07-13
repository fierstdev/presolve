@component("x-computed-template")
class ComputedTemplate extends Component {
  @computed()
  get label(): string { return "Ready"; }

  @computed()
  get visible(): boolean { return true; }

  render() {
    return <section title={this.label}>{this.label}{this.visible ? <span>Visible</span> : <span>Hidden</span>}</section>;
  }
}
