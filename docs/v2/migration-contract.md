# Migration command contract

`presolve migrate` is a schema v1, report-only command. Its JSON product embeds
the compiler-owned semantic-capability registry, lists automatic codemods, and
states its source-rewrite policy. The human form is the same canonical
capability migration guidance with the codemod boundary made explicit.

The initial report has an empty `automaticCodemods` list and policy
`report-only-no-source-rewrites`. This is intentional: no compiler-owned
source-transform product currently defines safe before/after syntax semantics.
The command must not use text replacement, AST heuristics, or a Vite adapter to
invent translations. A future codemod requires a versioned compiler transform,
source-location proof, before/after fixtures, and an explicit contract
amendment.
