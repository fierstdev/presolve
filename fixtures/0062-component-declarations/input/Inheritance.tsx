class PlainBase {
  render() { return <div />; }
}

class SemanticBase {
  value = state(0);
  render() { return <div />; }
}

@component("x-inherited-plain")
class InheritedPlain extends PlainBase {
  render() { return <main />; }
}

@component("x-inherited-semantic")
class InheritedSemantic extends SemanticBase {
  render() { return <main />; }
}
