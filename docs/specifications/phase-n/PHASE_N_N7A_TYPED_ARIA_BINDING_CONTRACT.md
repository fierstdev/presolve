# Phase N N7-A typed ARIA binding contract

N7-A admits twelve compiler-owned dynamic accessibility attributes through the
existing template binding, type diagnostic, artifact, and runtime attribute
path:

- `role`, `aria-label`, `aria-describedby`, `aria-errormessage`,
  `aria-controls`, `aria-current`, and `aria-live` require string expressions;
- `aria-invalid`, `aria-busy`, `aria-expanded`, `aria-pressed`, and
  `aria-hidden` require boolean expressions.

The compiler emits ordinary attribute updates; no accessibility framework,
DOM discovery, string coercion, arbitrary ARIA attribute inference, or custom
component API is introduced. ARIA token grammars, numeric attributes,
ID-reference list semantics, unsupported names such as `aria-owns`, and
non-attribute accessibility behavior remain outside this bounded family until
they receive their own semantic/type/lifecycle proof.

Verification is `scripts/verify-n7a-typed-aria-bindings.sh`.
