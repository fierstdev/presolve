# EdgeZero Documentation Set

Working name: **EdgeZero**  
Domain: **EdgeZero.dev**  
Fallback domain/name: **blokd.dev / Blokd**  
Core thesis: **a compiler-centered web authoring system where the compiler is the product and the runtime is deliberately small.**

This documentation set converts the product thesis into a buildable technical and strategic baseline. It is not a final public specification. It is intended for founders, compiler/runtime engineers, framework designers, developer-relations leads, and early design partners.

## Recommended positioning

> Write ordinary components. Ship HTML first. Load JavaScript only when the user needs it. Get precise updates without manual memoization. Export standards-native components.

The system is not positioned as “Web Components plus TSX plus signals.” That is too small. The intended category is:

> **A compiler-first framework for authoring resumable, standards-native web interfaces with TSX or HTML templates, fine-grained reactivity, and Web Component output.**

The stronger internal frame is:

> **A web authoring compiler with a framework surface.**

## Documentation map

1. [Name decision](./00_name_decision.md)
2. [Product strategy](./01_product_strategy.md)
3. [Architecture overview](./02_architecture_overview.md)
4. [Semantic graph model](./03_semantic_graph_model.md)
5. [Authoring model](./04_authoring_model.md)
6. [Compiler pipeline and IR](./05_compiler_pipeline_and_ir.md)
7. [Runtime model](./06_runtime_model.md)
8. [Resumability and delivery](./07_resumability_and_delivery.md)
9. [Forms, resources, and actions](./08_forms_resources_actions.md)
10. [Accessibility compiler](./09_accessibility_compiler.md)
11. [Tooling and diagnostics](./10_tooling_and_diagnostics.md)
12. [Interop and output targets](./11_interop_and_output_targets.md)
13. [MVP roadmap](./12_mvp_roadmap.md)
14. [Risk register](./13_risk_register.md)
15. [Examples](./14_examples.md)
16. [Competitive notes](./15_competitive_notes.md)
17. [Open questions](./16_open_questions.md)
18. [Source references](./17_source_references.md)
19. [Repository and Git strategy](./18_repository_and_git_strategy.md)
20. [Pre-development readiness review](./19_pre_development_readiness.md)
21. [Learning and development protocol](./20_learning_and_development_protocol.md)
22. [Documentation and progress tracking system](./21_documentation_and_progress_tracking.md)

## Non-negotiable design principles

1. The compiler must understand intent, not only lower syntax.
2. HTML is the primary artifact; JavaScript is loaded only when required.
3. Fine-grained updates are the default; virtual DOM diffing is not the baseline.
4. Resumability is a compiler capability, not a user ceremony.
5. Accessibility is compiler-enforced where static or semantic analysis can prove issues.
6. Every optimization has an explanation path.
7. Web Components are a native output target, but not the entire product.
8. The runtime exists to execute compiler decisions, not to compensate for compiler ignorance.

## Primary product promise

EdgeZero should make the best thing the default thing:

- static HTML where the UI is static,
- streaming HTML where data is async,
- resumable interaction where UI becomes interactive,
- lazy code loading at the event/resource boundary,
- precise binding updates where state changes,
- native form fallback where JavaScript is absent,
- standards-native component distribution where interop matters.

## Suggested repository layout

```txt
edgezero/
  packages/
    compiler/          # Rust or TypeScript compiler core
    runtime/           # Small browser runtime
    cli/               # ez/fw command surface
    language-tools/    # LSP, editor plugins, semantic inspector bridge
    wc/                # Web Component output helpers
    server/            # SSR, streaming, server actions, adapters
    devtools/          # browser extension / inspector
  examples/
    counter/
    forms/
    dashboard-streaming/
    wc-library/
  docs/
    philosophy/
    reference/
    compiler/
    runtime/
    guides/
  tests/
    compiler-fixtures/
    runtime-fixtures/
    integration/
```
