# Releasing Presolve

Presolve alpha releases are lockstep releases. Every public npm package,
platform CLI package, VS Code extension, and Rust crate must carry the same
version and be validated together.

Before publishing `0.1.0-alpha.1`:

1. run `pnpm release:check` and the relevant Rust/browser verification matrix;
2. pack and install the public framework, CLI, scaffold, tooling, and VS Code
   packages from their tarballs in a clean directory;
3. verify `pnpm create presolve`, `pnpm install`, VS Code TypeScript checking,
   and `pnpm build` in that installed project;
4. build and publish all supported platform CLI packages before publishing
   `@presolve/cli`;
5. package and publish `presolve-vscode` from its extension manifest;
6. build `examples/presolve-site` and run Cloudflare deployment preparation as
   a local dogfood proof. Production-site deployment is operated separately by
   the site owner.

Normal CI never publishes. The `Publish alpha` workflow runs only for a tag
matching `v0.1.0-alpha.*` or a manually supplied alpha version, verifies that
every public package and the Rust workspace use that exact version, and then
publishes the native CLI packages before the CLI wrapper and scaffold.

Before an operator runs it, create the `npm` GitHub environment with an
`NPM_TOKEN` that can publish the `presolve`, `create-presolve`, and
`@presolve/*` package names. Create the `vscode-marketplace` environment with
`VSCE_PAT` for the `fierstdev` publisher. The workflow does not deploy a
website or contact Cloudflare; the site owner separately owns Cloudflare
authentication and deployment.
