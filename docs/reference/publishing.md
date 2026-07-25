# Publish the Presolve release train

This guide is for maintainers releasing Presolve publicly. A release is one
versioned compatibility train, not a set of independently published packages.

## Release contents

- crates.io: `presolve-parser`, `presolve-compiler`, and `presolve-cli`,
  published in dependency order so the public compiler has resolvable registry
  dependencies.
- npm: `@presolve/core`, `create-presolve`, `@presolve/cli`, supported platform CLI
  packages, and any public runtime/tooling/application packages.
- Visual Studio Marketplace: `presolve-vscode` under the `fierstdev` publisher.
- GitHub: a matching annotated release tag and release notes.
- Cloudflare: the separately operated Presolve documentation site, deployed
  only after its installed-package smoke test passes.

## Required release sequence

1. Choose one prerelease version and update every publishable Cargo and npm
   manifest, package metadata, and compatibility assertion. Map
   `MAJOR.MINOR.PATCH-alpha.N` to the numeric Visual Studio Marketplace
   prerelease version `MAJOR.MINOR.(PATCH + N)` because the Marketplace does
   not accept SemVer prerelease suffixes. Presolve `0.1.0-alpha.7` therefore
   publishes as Marketplace prerelease `0.1.7`.
2. Run the full Rust, package, browser, artifact, formatter, and release-check
   matrix from a clean checkout.
3. Pack every npm package, install the tarballs in a fresh external directory,
   run `pnpm create presolve`, install, open/check it with the supported
   TypeScript configuration, then build it.
4. Publish platform-specific CLI packages before `@presolve/cli`; publish the
   framework, scaffold, and supported tooling packages at the exact same
   version and alpha tag.
5. Publish `presolve-parser`, then `presolve-compiler`, then `presolve-cli`,
   using a registry token with only the permissions required for those crates.
6. Package and publish `presolve-vscode`; install the produced VSIX in a clean
   VS Code profile and validate a newly generated project.
7. Create the GitHub release, then build and deploy the documentation site via
   its Cloudflare workflow.

Never publish from ordinary CI. Use protected release environments and scoped
credentials: npm token, crates.io token, Visual Studio Marketplace token, and
Cloudflare deployment credentials. The release workflow must verify package
contents, provenance, version equality, and no generated artifacts or secrets
before any registry upload.

The initial npm release requires an npm organization named `presolve`; that
organization owns the `@presolve` scope. Its GitHub Actions granular access
token must enable bypass 2FA, grant read/write access to all packages and
scopes the npm account can access, and grant at least read-only access to the
`presolve` organization. Organization permission alone does not grant package
publishing permission. The protected npm environment verifies the token's npm
identity, organization membership, and scope access before any immutable crate
is published.
