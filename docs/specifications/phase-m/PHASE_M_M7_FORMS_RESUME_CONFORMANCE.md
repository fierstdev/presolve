# Phase M M7 Forms, production, and resume conformance

**Status:** M7 implementation authority.

## Decision

Forms use the compiler's existing Form, Field, validation, serialization,
submit-host, runtime-artifact, and resume products. Phase M adds declaration
types only; it adds no Form controller, transport, validation runtime, DOM
discovery, browser-native `FormData` authority, or resume wrapper.

The production Form language is:

```tsx
@form() @serialize("json") profile!: Form;
@validate(required()) @field("profile") name = "";

@action() @submit("profile")
save(): void { this.submitted += 1; }

render() {
  return <form form={this.profile}><input field={this.name} /></form>;
}
```

`@field("profile")` and `@submit("profile")` are compiler-resolved local Form
designators. This is a production compiler-language refinement, not a
framework workaround: decorators execute in class-definition scope, where
`this.profile` is not a sound TypeScript expression. The compiler validates
the literal identifier, resolves it to the Form declared in the same component,
and preserves canonical diagnostics for malformed, missing, cross-component,
or invalid Form declarations. Existing compiler acceptance of `this.profile`
is retained only as compiler compatibility; framework source documents and
types no legacy spelling.

The render-time `<form form={this.profile}>` host remains unchanged because it
is an instance expression in an instance method. The compiler uses that host
as an explicit marker; it never discovers Form ownership from DOM ancestry.

## Type package

`@presolve/framework-types` declares `Form`, `form`, `field`, `serialize`,
`validate`, `required`, and `submit`. These are ambient authoring declarations
only. In particular, `FormDesignator` is source text passed to the compiler;
it is not a lookup token or a runtime handle.

## Evidence

The two framework fixtures are the exact sources built by the existing
real-browser probes:

1. `FormHost.tsx` proves explicit host submission reaches only compiler-emitted
   records, prevents default under compiler policy, and completes the emitted
   Action batch.
2. `ResumeForms.tsx` proves compiler-owned Form Field state restores exactly,
   reset uses compiled initials, and an active submission snapshot fails closed
   to the prescribed fallback.

The M7 verifier runs TypeScript 7.0, canonical checks for both unchanged
sources, the focused compiler submit-plan test, and both real-browser probes.
No framework-side artifact decoding, submission scheduling, or resume logic is
permitted.

## Boundary

M7 does not add async/server validation, submission parameters, HTTP transport,
automatic resets, custom controls, Form context/hooks, native FormData
semantics, router actions, server actions, or a framework lifecycle API.
