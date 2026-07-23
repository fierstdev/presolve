# Phase N N7-A typed ARIA binding contract

N7-A admits six compiler-owned dynamic ARIA attributes through the existing
template binding, type diagnostic, artifact, and runtime attribute path:

- `aria-invalid`, `aria-busy`, `aria-expanded`, and `aria-pressed` require
  boolean expressions;
- `aria-label`, `aria-describedby`, and `aria-live` require string expressions.

The compiler emits ordinary attribute updates; no accessibility framework,
DOM discovery, string coercion, arbitrary ARIA attribute inference, or custom
component API is introduced. Other ARIA names remain outside this bounded
family until they receive their own semantic/type/lifecycle proof.

Verification is `scripts/verify-n7a-typed-aria-bindings.sh`.
