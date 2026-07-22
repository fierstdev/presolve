@component("x-card") class Card extends Component { @slot() children!: SlotContent; render() { return <slot />; } }
@component("x-diagnostic") class Diagnostic extends Component { render() { return <Card><template slot="missing"><b /></template></Card>; } }
