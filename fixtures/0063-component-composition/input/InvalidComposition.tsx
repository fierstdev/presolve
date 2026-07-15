@component("x-empty") class Empty extends Component {
  @slot() children!: SlotContent;
  render() { return <div><slot /></div>; }
}
@component("x-missing") class Missing extends Component {
  @slot() header!: SlotContent;
  render() { return <div />; }
}
@component("x-duplicate") class Duplicate extends Component {
  @slot() header!: SlotContent;
  render() { return <div><slot name="header" /><slot name="header" /></div>; }
}
@component("x-cycle-a") class CycleA extends Component { render() { return <CycleB />; } }
@component("x-cycle-b") class CycleB extends Component { render() { return <CycleA />; } }
@component("x-invalid-page") class InvalidPage extends Component {
  render() {
    return <main><Empty /><Missing><template slot="header"><h1 /></template></Missing><Empty><template slot="unknown"><p /></template></Empty><Duplicate><template slot="header"><h1 /></template><template slot="header"><h2 /></template></Duplicate><Unknown><CycleA /></Unknown></main>;
  }
}
