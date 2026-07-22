@component("x-diagnostic") class Diagnostic extends Component { @slot() children!: SlotContent; render() { return <main><slot /><slot /></main>; } }
