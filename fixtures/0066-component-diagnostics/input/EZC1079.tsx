@component("x-card") class Card extends Component { @slot() children!: SlotContent; render() { return <slot />; } }
@component("x-diagnostic") class Diagnostic extends Component { render() { return <Card><b /></Card>; } }
