# Phase N N4-B JSX-to-HTML attribute aliases

N4-B admits exactly two JSX spellings on compiler-supported intrinsic elements:

| Authored spelling | Canonical compiler attribute |
| --- | --- |
| `className` | `class` |
| `htmlFor` | `for` |

Normalization happens in the compiler's template lowering step before template
semantic analysis, DOM binding type contracts, template manifests, static HTML,
ordinary-template runtime artifacts, and generated browser updates. The
framework does not transform JSX and the runtime carries no alias table.

Both aliases accept only the existing bounded static value or direct
compiler-supported string binding. After lowering, `class` and `for` use the
same string attribute contract and State dependency/resume path as any other
admitted attribute binding. The generated runtime therefore calls
`setAttribute("class", value)` or `setAttribute("for", value)`, never the JSX
spelling.

Spreads, style objects, arbitrary class composition helpers, selector-based
lookup, dynamic property names, and every other JSX/DOM alias remain outside
this slice.
