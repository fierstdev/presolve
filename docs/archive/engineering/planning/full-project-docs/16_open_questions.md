# Open Questions

## Product and positioning

1. Should the public name be EdgeZero, EdgeZero Compiler, or EdgeZero.dev?
2. Is the first market app frameworks, design systems, or form-heavy SaaS?
3. Should the first public demo focus on forms, streaming dashboards, or Web Component export?
4. Should EdgeZero position against React directly or avoid replacement framing early?
5. What is the smallest credible claim that still feels generational?

## Authoring model

1. Are class components mandatory in v1 or one supported authoring form?
2. Should decorators be required, or should there be non-decorator APIs for teams avoiding decorators?
3. How much deep reactivity should be supported?
4. Should TSX and `html`` templates have full parity in v1?
5. What syntax marks server actions: directive strings, decorators, helper wrappers, or inferred imports?
6. How much route declaration should be decorator-based vs filesystem-based?

## Compiler architecture

1. Is the compiler implemented in Rust, TypeScript, or a hybrid architecture?
2. Which parser stack should be used for TypeScript/TSX?
3. How stable/public should the IR be?
4. How much type information is required for v1?
5. Can accessibility analysis work without full TypeScript type checking in fast dev mode?
6. What is the source-map strategy for generated bindings and chunks?

## Runtime

1. What is the exact signal implementation model?
2. Should updates be sync by default or batched by default?
3. What is the maximum acceptable initial loader size?
4. How should event markers be represented in HTML?
5. How does resumability interact with browser back/forward cache?
6. How should shadow DOM event retargeting work for delegated handlers?

## Resumability

1. What is the serialization format?
2. How are server-only values proven absent from payloads?
3. How much state is serialized by default?
4. When should the compiler prefetch lazy handlers?
5. What is the fallback when handler chunks fail to load?
6. How should resumability work across nested components and slots?

## Forms/actions/resources

1. Which schema adapter ships first?
2. How are action endpoints named and secured?
3. What CSRF integration is required per adapter?
4. How are resource cache keys declared or inferred?
5. What invalidation can be inferred safely?
6. How should optimistic updates rollback across nested resources?

## Accessibility

1. Which checks are errors by default?
2. How strict should first-party components be?
3. How should dynamic content and opaque interop be represented in diagnostics?
4. Should accessibility diagnostics generate fixes/codemods?
5. How does the compiler handle localized accessible names?

## Styling

1. Should scoped CSS be first-party?
2. How does styling work across WC shadow DOM and light DOM targets?
3. How much dead CSS elimination is realistic in v1?
4. Should the compiler emit design-token manifests?
5. How are Tailwind-like utility systems analyzed?

## Interop

1. What React interop story is acceptable for v1?
2. Should EdgeZero consume custom-elements manifests?
3. How are opaque subtrees surfaced in explain output?
4. Can Web Component output avoid runtime dependency for static components?
5. What npm package formats are required: ESM only, CJS, types?

## Tooling

1. Should `edgezero explain` be implemented before DevTools?
2. What should machine-readable explain metadata look like?
3. Should SARIF be supported for a11y/check output?
4. What editor integration ships first?
5. How should migration tools be versioned?

## Business and ecosystem

1. What license should the framework use?
2. Will there be hosted services, or is this open-source only?
3. What governance model builds trust?
4. What design partners would validate the wedge?
5. What benchmarks and sample apps matter to serious teams?
6. How will the project avoid “toy framework” perception?
