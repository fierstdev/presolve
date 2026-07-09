# Tooling and Diagnostics

## Tooling thesis

Best-in-class DX is not only syntax. It is understanding.

EdgeZero should make generated behavior inspectable from source to DOM to network chunk to state update.

## CLI surface

```bash
edgezero dev
edgezero build
edgezero check
edgezero explain
edgezero inspect
edgezero analyze
edgezero migrate
edgezero test
edgezero doctor
edgezero size
edgezero a11y
edgezero trace
```

## `edgezero explain`

Explains what the compiler inferred for a file, component, route, or binding.

```bash
edgezero explain src/components/Counter.tsx
```

Example:

```txt
Component: x-counter
Source: src/components/Counter.tsx

Static DOM:
  <button> node n0
  text "Count: "
  dynamic text binding b0

State:
  count: serializable number

Bindings:
  b0 reads count
  update mode: text node patch

Events:
  click on n0
  handler: increment
  lazy chunk: counter.increment.js
  captures: none
  resumable: yes

Client JS:
  initial: 0.8kb loader
  on click: 1.1kb handler chunk

Accessibility:
  no issues

SSR:
  yes

Resumability:
  yes
```

## `edgezero why`

Subcommands:

```bash
edgezero why client-js
edgezero why hydrated x-user-card
edgezero why chunk dashboard.js
edgezero why binding b42 updates
edgezero why action saveUser is server-only
```

Example:

```txt
Why is dashboard.filters.js loaded initially?

Reason:
  Component DashboardFilters marked eager.
  Handler onInput uses browser API localStorage during initialization.

Source:
  src/routes/dashboard.tsx:42:11

Suggestion:
  Move localStorage read into onMount or mark only the preference branch clientOnly.
```

## `edgezero size`

Reports size by route, component, and interaction.

```bash
edgezero size --by-interaction
```

Example:

```txt
Route /checkout
  HTML: 18.2kb
  CSS: 4.1kb
  Initial JS: 0.9kb

Interactions:
  click ApplyCoupon
    coupon.apply.js: 1.2kb
  submit CheckoutForm
    checkout.submit.js: 1.4kb
    payment.validation.js: 2.1kb
```

## `edgezero trace`

Records update causality in development.

```bash
edgezero trace --binding b42
```

Example:

```txt
Binding b42 changed at 14:03:21.102
Source binding:
  src/routes/checkout.tsx:58:14
Reads:
  submit.pending
Cause:
  form submit action checkout started
Patch:
  setProperty(button#pay, "disabled", true)
Chunk:
  checkout.submit.js loaded in 22ms
```

## `edgezero a11y`

Runs compiler-level accessibility checks.

```bash
edgezero a11y --strict
```

Example:

```txt
A11Y001 error src/components/IconButton.tsx:12:5
Button has no accessible name.

A11Y014 warning src/components/Dialog.tsx:39:3
Dialog opens without an explicit focus target.
```

## `edgezero doctor`

Checks project health:

- target adapter versions,
- TypeScript config,
- decorator settings,
- CSP compatibility,
- server/client import leaks,
- package duplication,
- invalid custom-element names,
- compiler cache health.

## `edgezero migrate`

Migration support is a strategic feature. Frameworks lose trust when syntax/API changes become manual archaeology.

Capabilities:

- codemods,
- deprecated API detection,
- explainable transforms,
- dry-run mode,
- generated migration report.

```bash
edgezero migrate v0.4-to-v0.5 --dry-run
```

## LSP/editor support

Editor features:

- template type checking,
- route param inference,
- resource state hints,
- server/client boundary diagnostics,
- accessibility diagnostics inline,
- generated chunk preview,
- “explain this binding” command,
- jump from DOM binding ID to source.

## Browser semantic inspector

DevTools workflow:

```txt
click DOM node
  → source template expression
  → signal dependencies
  → last update cause
  → generated chunk
  → server/client ownership
  → accessibility diagnostics
  → resume/hydration status
```

Inspector panels:

1. Source.
2. DOM/bindings.
3. State graph.
4. Events/chunks.
5. Resources/actions.
6. Accessibility.
7. Styles.
8. Server/client ownership.
9. Trace timeline.

## Diagnostics quality bar

Every diagnostic should answer:

1. What failed?
2. Why does the model reject it?
3. Where is the source?
4. What generated behavior would have been unsafe or inefficient?
5. What should the developer do next?

Bad diagnostic:

```txt
Cannot serialize closure.
```

Good diagnostic:

```txt
Cannot make click handler `editUser` resumable because it captures `db`, imported from server-only module `~/db`.
The browser would need this value after lazy-loading the handler.
Move database access into a server action or mark this branch server-only.
```

## CI output

Machine-readable output should be available:

```bash
edgezero check --format json
edgezero size --format json
edgezero a11y --format sarif
```

SARIF support improves enterprise adoption because it integrates with code scanning systems.
