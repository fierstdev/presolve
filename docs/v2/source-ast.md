# General source AST

The parser now exposes `ParsedSourceAst` as the primary source-faithful syntax
product. It retains the original source text, a complete TypeScript-inclusive
OXC ESTree JSON representation, and the full program span.

The OXC parser remains the syntax frontend. Existing component, form, effect,
and JSX facts remain derived parser views for alpha compatibility; they are not
an alternate syntax authority. Parse recovery diagnostics stay on `ParsedFile`
and are carried beside the complete syntax product.

The ESTree representation includes TypeScript fields and TSX nodes, preserving
the general expressions, statements, declarations, imports, exports, class
members, type syntax, and attributes required by later analysis and tooling.
Later normalization must consume this source product together with TypeScript
semantic-authority queries.
