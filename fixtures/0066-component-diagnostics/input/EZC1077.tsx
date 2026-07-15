@component("x-card") class Card extends Component { @slot() header!: SlotContent; render() { return <article />; } }
@component("x-diagnostic") class Diagnostic extends Component { render() { return <Card><template slot="header"><b /></template></Card>; } }
