# Accessibility Compiler

## Positioning

Accessibility should be a first-class compiler capability, not a plugin afterthought.

The goal is not to replace manual accessibility testing, screen-reader testing, or design review. The goal is to make common accessibility failures structurally hard and immediately visible.

## Compiler-enforced checks

### Accessible names

Fail or warn for:

- `<button>` without accessible name,
- icon-only buttons without `aria-label` or labelled content,
- form controls without labels,
- images without required `alt`,
- links without useful names.

Example diagnostic:

```txt
A11Y001 Button has no accessible name.
File: src/components/IconButton.tsx:14:7
Fix: add text content, aria-label, or aria-labelledby.
```

### Form relationships

Validate:

- `label for` matches control ID,
- nested label contains exactly intended control,
- field errors are associated through `aria-describedby` or equivalent,
- required/invalid states are exposed,
- custom fields preserve native semantics or implement ARIA correctly.

### ARIA validity

Validate:

- invalid `aria-*` attribute names,
- incompatible ARIA roles,
- required ARIA attributes missing for roles,
- ARIA used where native HTML is better,
- invalid `aria-hidden` on focusable content.

### Keyboard interaction

Warn for:

- clickable non-interactive elements,
- custom controls without keyboard handlers,
- roving-tabindex components without complete behavior,
- drag/drop interactions without keyboard alternatives where required.

### Focus management

Warn/fail for semantic components:

- modal without escape handling,
- modal without focus return,
- popover without focus policy,
- focus trap without exit path,
- route transition without focus restoration policy.

### Live regions

Validate:

- pending/error states needing announcement,
- dynamic form errors,
- toast notifications,
- async region completion where relevant.

## Semantic component knowledge

The compiler should understand first-party primitives:

```tsx
<Form />
<Field />
<Errors />
<Dialog />
<Popover />
<Menu />
<Tabs />
<Switch />
```

These should compile to accessible behavior and metadata rather than relying on runtime conventions alone.

## Severity levels

```txt
error    provable accessibility failure likely to block use
warning  likely issue or incomplete semantic proof
info     improvement or best-practice recommendation
```

Default CI policy:

- errors fail build,
- warnings fail only when configured,
- infos never fail.

## Config

```ts
export default {
  a11y: {
    level: "strict",
    failOnWarnings: false,
    rules: {
      "button-name": "error",
      "clickable-div": "error",
      "missing-alt": "error",
      "modal-focus": "warning"
    }
  }
};
```

## Explain output

```txt
Accessibility:
  Field email
    label: <label> Email -> input#email
    errors: #email-error via aria-describedby
    required: native required + schema required
  Submit button
    accessible name: "Save"
  Issues:
    none
```

## DevTools inspector

The semantic inspector should show:

- accessible name computation,
- role,
- label source,
- described-by regions,
- focusability,
- keyboard handlers,
- compiler diagnostics,
- source file mapping.

## Limits

The compiler cannot fully prove:

- visual contrast in all dynamic themes,
- actual screen-reader user experience,
- cognitive accessibility,
- content quality,
- focus behavior of opaque third-party widgets,
- every dynamic runtime DOM mutation.

For opaque zones, the compiler should mark reduced confidence:

```txt
A11Y-W_OPAQUE Third-party widget ChartPanel mutates DOM outside compiler graph.
Static accessibility guarantees do not apply inside this subtree.
```

## Why this is a business differentiator

Serious teams care about accessibility because it affects compliance, procurement, usability, and brand risk. A framework that catches accessibility failures during compilation has a stronger enterprise story than a framework that delegates accessibility entirely to linting and manual QA.
