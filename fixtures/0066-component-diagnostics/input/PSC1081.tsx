@component("x-theme") class Theme extends Component { @context() color!: string; render() { return <div />; } }
@component("x-leaf") class Leaf extends Component { @consume(Theme.color) color!: number; render() { return <span />; } }
@component("x-card") class Card extends Component { @provide(Theme.color) color: string = "blue"; render() { return <Leaf />; } }
@component("x-diagnostic") @route("/") class Diagnostic extends Component { render() { return <Card />; } }
