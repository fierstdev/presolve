# Name Decision

## Recommendation

Use **EdgeZero** as the working product name and **EdgeZero.dev** as the primary domain.

Use **Blokd** only as a possible sub-brand for a visual editor, component block registry, or experimental package name. Do not use Blokd as the primary framework name unless trademark/search diligence reveals a hard blocker for EdgeZero.

## Why EdgeZero fits the strategy

EdgeZero maps cleanly onto the product thesis:

- **Edge** implies deployment locality, streaming, server/client splitting, and modern infrastructure.
- **Zero** implies zero wasted JavaScript, zero mandatory hydration, zero manual memoization, and zero inaccessible-by-default UI.
- The name supports a serious developer-tooling posture.
- It can stretch from framework to compiler to platform: EdgeZero Compiler, EdgeZero Runtime, EdgeZero Inspector, EdgeZero Cloud adapters.

The name should not promise literal “zero JavaScript.” The safer public phrase is:

> Zero wasted JavaScript.

## Why Blokd is weaker as the primary name

Blokd has some useful associations: blocks, components, composable UI, and possibly “blocked from shipping waste.” But as a primary framework name it has several problems:

- It reads like “blocked,” a negative developer emotion.
- The dropped vowel makes it feel more consumer/web3/gaming than infrastructure-grade.
- It does not naturally communicate HTML-first delivery, edge execution, resumability, compiler intelligence, or standards-native output.
- It is more likely to be misheard, misspelled, or treated as a toy brand.

Blokd could still work for:

- a component playground,
- visual UI builder,
- block registry,
- content/design-system marketplace,
- internal codename.

## Naming risks

A web search found existing use of **Edge Zero** by an energy/grid-monitoring company and an EdgeZero mobile app associated with that ecosystem. Owning EdgeZero.dev does not by itself clear trademark, package-name, or market-confusion risk.

Before public launch, perform:

1. Trademark search in the United States, EU, UK, Canada, Australia, and intended launch markets.
2. npm package-name availability checks for `edgezero`, `@edgezero/*`, `ez`, and possible CLI names.
3. GitHub organization availability checks.
4. Social handle checks.
5. Legal review for software/developer-tool classes.
6. Search for confusingly similar developer tools, CDN products, edge platforms, and web frameworks.

## Brand architecture

Recommended structure:

```txt
EdgeZero.dev        public site
EdgeZero Compiler   core product
EdgeZero Runtime    small browser/server runtime
EdgeZero Inspector  semantic DevTools surface
EdgeZero Forms      optional first-party form layer
EdgeZero WC         Web Component output target
EdgeZero Kit        full-stack application layer
```

## CLI naming

Avoid `fw` publicly. It is too generic and hard to search.

Candidate CLIs:

- `edgezero`
- `ez`
- `ezero`
- `edgez`

Preferred:

```bash
npx edgezero dev
npx edgezero explain src/components/Counter.tsx
```

Use `ez` only if package availability and naming clarity are acceptable.

## Tagline candidates

Primary:

> HTML first. JavaScript when needed. Compiler by default.

Developer-oriented:

> Write components. Ship intent.

Performance-oriented:

> Zero wasted JavaScript.

Enterprise-oriented:

> A compiler-first web framework for fast, accessible, standards-native interfaces.

Most precise:

> A web authoring compiler with a framework surface.

## Public category language

Do not lead with:

> Web Components + TSX + signals.

Lead with:

> A compiler-centered web authoring system that turns ordinary components into HTML-first, resumable, standards-native interfaces.
