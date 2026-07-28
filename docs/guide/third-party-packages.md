# Third-party packages and opaque boundaries

Presolve cannot and should not infer the internals of every npm package. The
compiler instead needs a declared contract for the way an application uses a
package: its imported capabilities, the allowed boundary, and the values that
cross it.

Use a package normally when the compiler admits the imported value and its use.
For an intentional terminal call that the compiler cannot lower, mark the
method boundary with `@opaque(packageSpecifier, exportName)`. This is a legacy
decorator compatibility form; no decorator-free opaque declaration has been
admitted for the beta.

```tsx
import { trackPurchase } from "@acme/analytics";
import { action, component, opaque, Component } from "presolve";

@component()
export class Checkout extends Component {
  @action()
  @opaque("@acme/analytics", "trackPurchase")
  track(): void {
    trackPurchase();
  }

  render() { return <button onClick={this.track}>Buy</button>; }
}
```

`@opaque` is a declared escape boundary, not a fallback reactive runtime or a
way to hide state reads and writes from the compiler. Keep its body terminal:
it should not create compiler-owned state, derived values, routing, or runtime
identity that Presolve must understand.

If a package is central to an application, define and test a package contract
instead of repeatedly using opaque calls. If no contract or opaque boundary is
appropriate, the source is outside the currently supported semantic surface
and should fail clearly at compile time.
