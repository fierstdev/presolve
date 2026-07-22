@component("x-card") class Card extends Component { @slot() header!: SlotContent; render() { return <slot name="header" />; } }
@component("x-diagnostic") class Diagnostic extends Component { render() { return <Card><template slot="header"><b /></template><template slot="header"><i /></template></Card>; } }
