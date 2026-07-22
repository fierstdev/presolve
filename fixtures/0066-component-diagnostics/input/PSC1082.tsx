@component("x-card") class Card extends Component { render() { return <article />; } }
@component("x-diagnostic") class Diagnostic extends Component { visible = state(true); render() { return <main>{this.visible ? <Card /> : <span />}</main>; } }
