@component("x-invalid-slots")
class InvalidSlots extends Component {
  @slot("header") argument!: SlotContent;
  @slot() static staticSlot!: SlotContent;
  @slot() missingType!;
  @slot() wrongType!: string;
  @slot() initialized: SlotContent = "bad";
  @slot() notDefinite: SlotContent;
  @slot() @context() conflict!: SlotContent;
  @slot() duplicate!: SlotContent;
  @slot() duplicate!: SlotContent;
  @slot() method() {}
  @slot() get accessor(): SlotContent { return this.value; }
  attach(@slot() content: SlotContent) {}
  render() { return <main />; }
}
