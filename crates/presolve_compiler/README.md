# presolve-compiler

The Rust compiler for Presolve TypeScript and TSX applications. It owns the
semantic model and produces application artifacts consumed by the CLI and
tooling packages.

Most web applications should use `pnpm create presolve` rather than this crate
directly. The crate is published for embedding and tool integration.
