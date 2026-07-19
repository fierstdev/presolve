# Examples

## Counter

```tsx
@component("x-counter")
class Counter extends Component {
  @state count = 0;

  increment() {
    this.count++;
  }

  render() {
    return <button onClick={this.increment}>Count: {this.count}</button>;
  }
}
```

Expected compiler explanation:

```txt
Component: x-counter
State:
  count: serializable number
Bindings:
  b0 text reads count
Events:
  click -> increment
  lazy: yes
  resumable: yes
Initial JS:
  loader only
Patch:
  count change -> text node update
```

## User profile route

```tsx
@route("/users/:id")
@component("user-profile")
class UserProfile extends Page {
  user = resource(({ params }) => getUser(params.id));
  @state editing = false;

  save = action(async form => {
    "server";
    await updateUser(this.user.id, form);
    this.editing = false;
  });

  render() {
    return (
      <main>
        <h1>{this.user.name}</h1>
        <Show when={this.editing} fallback={
          <button onClick={() => this.editing = true}>Edit</button>
        }>
          <form action={this.save}>
            <label>
              Name
              <input name="name" value={this.user.name} />
            </label>
            <button disabled={this.save.pending}>Save</button>
          </form>
        </Show>
      </main>
    );
  }
}
```

Expected compiler inference:

```txt
Route:
  /users/:id
Server data:
  getUser(params.id)
Serialized state:
  editing
  user snapshot if needed for client interaction
Initial HTML:
  fully rendered
Client startup:
  no component JS needed until interaction
Click Edit:
  load edit handler chunk
  resume editing state
  patch conditional branch
Submit:
  native form fallback if JS unavailable
  enhanced server action if JS available
  stream validation errors if supported
Accessibility:
  input has label
  buttons have accessible names
```

## Checkout form

```tsx
@component("checkout-form")
class CheckoutForm extends Component {
  cart = resource(getCart);

  submit = action(async form => {
    "server";
    return await checkout(form);
  });

  render() {
    return (
      <form action={this.submit}>
        <CartSummary cart={this.cart} />
        <Field name="email" type="email" required />
        <Field name="address" />
        <button disabled={this.submit.pending}>
          {this.submit.pending ? "Processing…" : "Pay"}
        </button>
      </form>
    );
  }
}
```

Expected compiler explanation:

```txt
Initial JS:
  0.9kb loader
HTML:
  fully server-rendered
Resumability:
  form submit resumable
  cart summary static until invalidated
Accessibility:
  all fields labelled
  pending state announced
Server/client split:
  checkout runs server-side
  pending state client-resumable
Lazy chunks:
  submit handler: 1.4kb
  payment validation: 2.1kb loaded on submit
Fallback:
  native form POST works without JS
```

## Streaming dashboard

```tsx
@route("/dashboard")
class Dashboard extends Page {
  metrics = resource(getMetrics, { stream: true });
  activity = resource(getActivityFeed, { stream: true, staleTime: "30s" });

  render() {
    return (
      <main>
        <h1>Dashboard</h1>

        <Await resource={this.metrics} fallback={<MetricSkeleton />}>
          {metrics => <MetricsGrid metrics={metrics} />}
        </Await>

        <Await resource={this.activity} fallback={<ActivitySkeleton />}>
          {items => <ActivityFeed items={items} />}
        </Await>
      </main>
    );
  }
}
```

Expected compiler inference:

```txt
Streaming regions:
  metrics: flush fallback immediately, replace on resolve
  activity: flush fallback immediately, replace on resolve
Waterfalls:
  none detected
Client JS:
  none unless child components contain interactions
Error handling:
  nearest route error boundary
```

## Web Component export

```tsx
@component("ez-price-card")
class PriceCard extends Component {
  @property plan = "Pro";
  @property price = "$29";

  render() {
    return (
      <article part="card">
        <h2>{this.plan}</h2>
        <p>{this.price}</p>
        <slot name="features" />
        <button part="button">Choose plan</button>
      </article>
    );
  }
}
```

Build:

```bash
edgezero build --target wc-library
```

Expected output:

```txt
Custom element:
  ez-price-card
Properties:
  plan -> attribute plan
  price -> attribute price
Slots:
  features
Parts:
  card, button
Runtime:
  custom-element updater only
```

Usage:

```html
<ez-price-card plan="Team" price="$99">
  <ul slot="features">
    <li>Unlimited projects</li>
  </ul>
</ez-price-card>
```

## Illegal server capture diagnostic

Author code:

```tsx
import { db } from "~/db";

@component("bad-button")
class BadButton extends Component {
  async handleClick() {
    await db.users.delete("123");
  }

  render() {
    return <button onClick={this.handleClick}>Delete</button>;
  }
}
```

Expected diagnostic:

```txt
EZ-SPLIT-001 Cannot compile click handler `handleClick` for the client.

Reason:
  `handleClick` captures `db`, imported from server-only module `~/db`.

Why this matters:
  The browser would need to load a server-only database client to resume this handler.

Fix:
  Move the mutation into a server action:

  deleteUser = action(async () => {
    "server";
    await db.users.delete("123");
  });
```
