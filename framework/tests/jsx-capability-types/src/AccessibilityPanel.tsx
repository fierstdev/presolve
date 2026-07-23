@component("x-accessibility-panel")
class AccessibilityPanel extends Component {
  label = state("Search");
  invalid = state(false);

  @action()
  clear(): void {}

  render() {
    return (
      <label
        className={this.label}
        htmlFor="search"
        aria-label={this.label}
        aria-invalid={this.invalid}
      >
        <input id="search" onKeydown={this.clear} />
      </label>
    );
  }
}
