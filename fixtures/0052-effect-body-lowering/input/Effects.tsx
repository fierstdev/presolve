@component("x-effects")
class Effects extends Component {
  @effect()
  sync() {
    document.title = this.title;
    analytics.track("view", this.subtotal + this.tax);
    return;
  }

  @effect()
  invalid() {
    this.count = 1;
    this.increment();
    return () => unsubscribe();
    const title = this.title;
  }

  @effect()
  persistTheme() {
    storage.set("theme", this.theme);
  }

  @effect()
  reportTotal() {
    analytics.track("checkout", this.subtotal + this.tax);
  }

  @effect()
  syncMultipleTargets() {
    document.title = this.title;
    analytics.track("view", this.route);
  }

  @effect()
  explicitCompletion() {
    analytics.track("ready", this.ready);
    return;
  }

  @effect()
  invalidMutation() {
    this.count = 1;
  }

  @effect()
  invalidActionCall() {
    this.increment();
  }

  @effect()
  invalidCleanup() {
    return () => unsubscribe();
  }

  @effect()
  invalidLocal() {
    const title = this.title;
    document.title = title;
  }

  @effect()
  invalidBranch() {
    if (this.enabled) {
      analytics.enable();
    }
  }

  render() { return <p />; }
}
